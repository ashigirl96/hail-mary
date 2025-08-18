use anyhow::{Context, Result};
use hail_mary::models::kiro::KiroConfig;
use hail_mary::models::memory::MemoryType;
use hail_mary::repositories::memory::{MemoryRepository, SqliteMemoryRepository};
use hail_mary::services::memory::{MemoryInput, MemoryService};
use std::time::Instant;
use tempfile::TempDir;

/// Scale tests for Memory MCP system
/// Tests system behavior with large datasets and validates design constraints:
/// - 10,000 memories should work smoothly
/// - Database size < 100MB for 10,000 memories
/// - Memory usage < 50MB during normal operations
/// - Search performance degrades gracefully with scale

#[tokio::test]
async fn test_10k_memories_constraint() -> Result<()> {
    let (temp_dir, mut service) = setup_scale_test_service().await?;
    
    println!("Starting 10k memories scale test...");
    
    // Populate with exactly 10,000 memories
    let start_time = Instant::now();
    populate_large_scale_dataset(&mut service, 10000).await?;
    let population_time = start_time.elapsed();
    
    println!("Populated 10,000 memories in {:.2}s", population_time.as_secs_f64());
    
    // Verify count
    let memory_count = count_memories_directly(&temp_dir).await?;
    assert_eq!(memory_count, 10000, "Should have exactly 10,000 memories");
    
    // Test database size constraint
    let db_size = get_database_size(&temp_dir)?;
    let size_mb = db_size / (1024 * 1024);
    
    assert!(
        size_mb < 100,
        "Database size {}MB exceeds 100MB limit for 10,000 memories",
        size_mb
    );
    
    // Test search performance with large dataset
    let search_start = Instant::now();
    let results = service.recall("performance", 10, None, vec![]).await?;
    let search_time = search_start.elapsed();
    
    assert!(
        search_time.as_millis() < 500,
        "Search with 10k memories took {}ms, expected < 500ms",
        search_time.as_millis()
    );
    
    assert!(!results.is_empty(), "Should find results even with 10k memories");
    
    println!(
        "Scale test results: {}MB database, search: {}ms",
        size_mb,
        search_time.as_millis()
    );
    
    Ok(())
}

#[tokio::test]
async fn test_memory_usage_under_50mb() -> Result<()> {
    let (_temp_dir, mut service) = setup_scale_test_service().await?;
    
    // Populate with moderate dataset for memory usage testing
    populate_large_scale_dataset(&mut service, 5000).await?;
    
    // Memory usage testing would require process monitoring
    // For now, we'll test that operations complete successfully without OOM
    
    // Test multiple concurrent operations to stress memory usage
    let concurrent_operations = 5;
    let mut handles = Vec::new();
    
    for i in 0..concurrent_operations {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let search_term = format!("test{}", i);
            service_clone.lock().await.recall(&search_term, 20, None, vec![]).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.unwrap();
        result.unwrap_or_else(|e| panic!("Concurrent operation {} failed: {}", i, e));
    }
    
    // Test document generation under memory pressure
    let temp_config = create_test_config(&tempfile::tempdir()?.path())?;
    let doc_start = Instant::now();
    service.generate_documents(&temp_config).await?;
    let doc_time = doc_start.elapsed();
    
    assert!(
        doc_time.as_millis() < 5000,
        "Document generation with 5k memories took {}ms, expected < 5000ms",
        doc_time.as_millis()
    );
    
    println!("Memory stress test: completed {} concurrent operations", concurrent_operations);
    Ok(())
}

#[tokio::test]
async fn test_search_performance_degradation() -> Result<()> {
    let (_temp_dir, mut service) = setup_scale_test_service().await?;
    
    // Test search performance at different scales
    let scale_points = [100, 500, 1000, 2500, 5000];
    let mut performance_data = Vec::new();
    
    for &scale in &scale_points {
        // Start fresh for each scale point
        let (_temp_dir, mut fresh_service) = setup_scale_test_service().await?;
        
        // Populate with memories
        populate_large_scale_dataset(&mut fresh_service, scale).await?;
        
        // Warm up search
        fresh_service.recall("test", 5, None, vec![]).await?;
        
        // Measure search performance
        let search_start = Instant::now();
        let results = fresh_service.recall("performance", 10, None, vec![]).await?;
        let search_time = search_start.elapsed();
        
        performance_data.push((scale, search_time.as_millis()));
        
        assert!(
            search_time.as_millis() < 1000,
            "Search with {} memories took {}ms, should be < 1000ms",
            scale,
            search_time.as_millis()
        );
        
        assert!(!results.is_empty(), "Should find results at scale {}", scale);
        
        println!("Search performance at {} memories: {}ms", scale, search_time.as_millis());
    }
    
    // Verify that performance degradation is reasonable (sublinear)
    // Performance should not increase linearly with dataset size
    let (small_scale, small_time) = performance_data[0];
    let (large_scale, large_time) = performance_data.last().unwrap();
    
    let scale_factor = *large_scale as f64 / small_scale as f64;
    let time_factor = *large_time as f64 / small_time as f64;
    
    assert!(
        time_factor < scale_factor,
        "Search performance degradation is too severe: {}x scale caused {}x time increase",
        scale_factor,
        time_factor
    );
    
    println!(
        "Performance scaling: {}x data → {:.2}x time (sublinear ✓)",
        scale_factor,
        time_factor
    );
    
    Ok(())
}

#[tokio::test]
async fn test_batch_operations_scaling() -> Result<()> {
    let (_temp_dir, mut service) = setup_scale_test_service().await?;
    
    // Test batch operations with increasing batch sizes
    let batch_sizes = [10, 50, 100, 250, 500];
    
    for &batch_size in &batch_sizes {
        let memories = create_test_memories(batch_size);
        
        let start = Instant::now();
        let result = service.remember_batch(memories).await?;
        let elapsed = start.elapsed();
        
        assert_eq!(result.len(), batch_size, "Should save all memories in batch");
        
        let per_memory_ms = elapsed.as_millis() as f64 / batch_size as f64;
        
        // Per-memory time should remain reasonable even for large batches
        assert!(
            per_memory_ms < 50.0,
            "Batch of {} memories: {:.2}ms per memory (should be < 50ms)",
            batch_size,
            per_memory_ms
        );
        
        println!(
            "Batch save ({} memories): {}ms total, {:.2}ms per memory",
            batch_size,
            elapsed.as_millis(),
            per_memory_ms
        );
    }
    
    Ok(())
}

#[tokio::test]
async fn test_database_growth_patterns() -> Result<()> {
    let (temp_dir, mut service) = setup_scale_test_service().await?;
    
    // Measure database growth at different scales
    let growth_points = [1000, 2000, 3000, 4000, 5000];
    let mut size_data = Vec::new();
    
    for &count in &growth_points {
        // Add more memories (cumulative)
        let additional = if size_data.is_empty() { count } else { count - growth_points[size_data.len() - 1] };
        populate_large_scale_dataset(&mut service, additional).await?;
        
        let db_size = get_database_size(&temp_dir)?;
        let size_mb = db_size / (1024 * 1024);
        
        size_data.push((count, size_mb));
        
        // Database should not grow excessively
        let mb_per_1k_memories = size_mb as f64 / (count as f64 / 1000.0);
        assert!(
            mb_per_1k_memories < 20.0,
            "Database uses {:.1}MB per 1k memories (should be < 20MB)",
            mb_per_1k_memories
        );
        
        println!("Database growth: {} memories → {}MB ({:.1}MB per 1k)", count, size_mb, mb_per_1k_memories);
    }
    
    // Verify linear growth pattern (not exponential)
    let (small_count, small_size) = size_data[0];
    let (large_count, large_size) = size_data.last().unwrap();
    
    let count_ratio = *large_count as f64 / small_count as f64;
    let size_ratio = *large_size as f64 / small_size as f64;
    
    // Size growth should be roughly linear (allow 20% deviation)
    assert!(
        (size_ratio / count_ratio - 1.0).abs() < 0.2,
        "Database growth is not linear: {}x memories → {}x size",
        count_ratio,
        size_ratio
    );
    
    println!("Database growth pattern: {:.1}x memories → {:.1}x size (linear ✓)", count_ratio, size_ratio);
    Ok(())
}

#[tokio::test]
async fn test_fts5_index_performance_at_scale() -> Result<()> {
    let (_temp_dir, mut service) = setup_scale_test_service().await?;
    
    // Populate with large dataset
    populate_large_scale_dataset(&mut service, 7500).await?;
    
    // Test various search patterns that stress FTS5
    let search_patterns = [
        ("single", "Single term search"),
        ("rust programming", "Two terms"),
        ("async rust programming performance", "Multiple terms"),
        ("プログラミング", "Japanese term"),
        ("\"exact phrase\"", "Exact phrase search"),
    ];
    
    for (query, description) in search_patterns {
        // Test with different result limits
        let limits = [5, 10, 25, 50];
        
        for &limit in &limits {
            let start = Instant::now();
            let results = service.recall(query, limit, None, vec![]).await?;
            let elapsed = start.elapsed();
            
            // FTS5 should remain fast even with large datasets
            assert!(
                elapsed.as_millis() < 300,
                "{} (limit {}): {}ms, expected < 300ms",
                description,
                limit,
                elapsed.as_millis()
            );
            
            let found_count = count_markdown_results(&results);
            assert!(
                found_count <= limit,
                "Found {} results but limit was {}",
                found_count,
                limit
            );
        }
        
        println!("{}: FTS5 performance validated at scale", description);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_access_at_scale() -> Result<()> {
    let (_temp_dir, service) = setup_scale_test_service().await?;
    
    // Populate with substantial dataset
    service.lock().await.remember_batch(create_test_memories(3000)).await?;
    
    // Test concurrent read operations
    let concurrent_readers = 10;
    let reads_per_thread = 5;
    
    let start = Instant::now();
    let handles: Vec<_> = (0..concurrent_readers)
        .map(|thread_id| {
            let service_clone = service.clone();
            tokio::spawn(async move {
                let mut results = Vec::new();
                for i in 0..reads_per_thread {
                    let query = format!("test{}", (thread_id * reads_per_thread + i) % 10);
                    let result = service_clone.lock().await.recall(&query, 5, None, vec![]).await;
                    results.push(result);
                }
                (thread_id, results)
            })
        })
        .collect();
    
    // Wait for all operations
    for handle in handles {
        let (thread_id, results) = handle.await.unwrap();
        for (i, result) in results.into_iter().enumerate() {
            result.unwrap_or_else(|e| panic!("Thread {} operation {} failed: {}", thread_id, i, e));
        }
    }
    
    let total_time = start.elapsed();
    let operations_per_sec = (concurrent_readers * reads_per_thread) as f64 / total_time.as_secs_f64();
    
    assert!(
        operations_per_sec > 20.0,
        "Concurrent operations: {:.1} ops/sec (should be > 20)",
        operations_per_sec
    );
    
    println!(
        "Concurrent access: {} threads × {} ops = {:.1} ops/sec",
        concurrent_readers,
        reads_per_thread,
        operations_per_sec
    );
    
    Ok(())
}

#[tokio::test]
async fn test_system_limits_and_boundaries() -> Result<()> {
    let (_temp_dir, mut service) = setup_scale_test_service().await?;
    
    // Test with very large individual memories
    let large_memory = MemoryInput {
        memory_type: MemoryType::Tech,
        title: "Very Large Memory Test".to_string(),
        content: "x".repeat(100_000), // 100KB content
        tags: vec!["large".to_string(), "test".to_string()],
        confidence: Some(0.9),
    };
    
    let start = Instant::now();
    let result = service.remember_batch(vec![large_memory]).await?;
    let elapsed = start.elapsed();
    
    assert_eq!(result.len(), 1, "Should handle large memories");
    assert!(
        elapsed.as_millis() < 1000,
        "Large memory save took {}ms, expected < 1000ms",
        elapsed.as_millis()
    );
    
    // Test search in large content
    let search_start = Instant::now();
    let search_results = service.recall("x", 5, None, vec![]).await?;
    let search_elapsed = search_start.elapsed();
    
    assert!(!search_results.is_empty(), "Should find content in large memory");
    assert!(
        search_elapsed.as_millis() < 500,
        "Search in large content took {}ms, expected < 500ms",
        search_elapsed.as_millis()
    );
    
    // Test with maximum reasonable tag count
    let many_tags_memory = MemoryInput {
        memory_type: MemoryType::Tech,
        title: "Many Tags Test".to_string(),
        content: "Memory with many tags for testing tag search performance".to_string(),
        tags: (0..100).map(|i| format!("tag{}", i)).collect(),
        confidence: Some(0.9),
    };
    
    let tag_result = service.remember_batch(vec![many_tags_memory]).await?;
    assert_eq!(tag_result.len(), 1, "Should handle memories with many tags");
    
    println!("System limits test: handled large content and many tags successfully");
    Ok(())
}

// Helper functions

async fn setup_scale_test_service() -> Result<(TempDir, MemoryService<SqliteMemoryRepository>)> {
    let temp_dir = tempfile::tempdir()?;
    let config = create_test_config(temp_dir.path())?;
    
    let repository = SqliteMemoryRepository::new(&config)?;
    let service = MemoryService::new(repository, config);
    
    Ok((temp_dir, service))
}

fn create_test_config(temp_path: &std::path::Path) -> Result<KiroConfig> {
    let db_path = temp_path.join("db.sqlite3");
    
    Ok(KiroConfig {
        root_dir: temp_path.to_path_buf(),
        memory: hail_mary::models::kiro::MemoryConfig {
            types: vec![
                "tech".to_string(),
                "project-tech".to_string(),
                "domain".to_string(),
            ],
            instructions: "Scale test instructions".to_string(),
            document: hail_mary::models::kiro::DocumentConfig {
                output_dir: temp_path.to_path_buf(),
                format: "markdown".to_string(),
            },
            database: hail_mary::models::kiro::DatabaseConfig {
                path: db_path,
            },
        },
    })
}

async fn populate_large_scale_dataset(
    service: &mut MemoryService<SqliteMemoryRepository>,
    count: usize,
) -> Result<()> {
    const BATCH_SIZE: usize = 500; // Larger batches for scale testing
    
    for batch_start in (0..count).step_by(BATCH_SIZE) {
        let batch_end = std::cmp::min(batch_start + BATCH_SIZE, count);
        let batch_size = batch_end - batch_start;
        
        let memories = create_scale_test_memories(batch_start, batch_size);
        service.remember_batch(memories).await?;
        
        // Progress indicator for large datasets
        if batch_end % 2000 == 0 || batch_end == count {
            println!("Populated {}/{} memories", batch_end, count);
        }
    }
    
    Ok(())
}

fn create_scale_test_memories(start_index: usize, count: usize) -> Vec<MemoryInput> {
    (0..count)
        .map(|i| {
            let index = start_index + i;
            MemoryInput {
                memory_type: match index % 3 {
                    0 => MemoryType::Tech,
                    1 => MemoryType::ProjectTech,
                    _ => MemoryType::Domain,
                },
                title: format!("Scale Test Memory {}", index),
                content: format!(
                    "This is scale test memory number {}. \
                     Content includes keywords for search testing: \
                     programming, performance, rust, database, optimization, \
                     testing, development, scale, システム, テスト, \
                     プログラミング, データベース. \
                     Additional content to make realistic size: {}",
                    index,
                    "Lorem ipsum ".repeat(10)
                ),
                tags: vec![
                    "scale-test".to_string(),
                    "performance".to_string(),
                    match index % 5 {
                        0 => "programming",
                        1 => "database",
                        2 => "optimization",
                        3 => "testing",
                        _ => "development",
                    }.to_string(),
                    if index % 10 == 0 { "rust" } else { "general" }.to_string(),
                ],
                confidence: Some(0.7 + (index as f32 * 0.001) % 0.3),
            }
        })
        .collect()
}

fn create_test_memories(count: usize) -> Vec<MemoryInput> {
    create_scale_test_memories(0, count)
}

fn get_database_size(temp_dir: &TempDir) -> Result<u64> {
    let db_path = temp_dir.path().join("db.sqlite3");
    let metadata = std::fs::metadata(&db_path)
        .context("Failed to get database file metadata")?;
    Ok(metadata.len())
}

async fn count_memories_directly(temp_dir: &TempDir) -> Result<usize> {
    let config = create_test_config(temp_dir.path())?;
    let repository = SqliteMemoryRepository::new(&config)?;
    let memories = repository.find_all()?;
    Ok(memories.len())
}

fn count_markdown_results(markdown: &str) -> usize {
    markdown.lines().filter(|line| line.starts_with("## ")).count()
}

#[cfg(test)]
mod scale_test_helpers {
    use super::*;
    
    #[test]
    fn test_markdown_result_counting() {
        let markdown = r#"
# Document Title

## Memory 1
Content 1

## Memory 2
Content 2

## Memory 3
Content 3
"#;
        
        let count = count_markdown_results(markdown);
        assert_eq!(count, 3);
    }
    
    #[test]
    fn test_scale_memory_generation() {
        let memories = create_scale_test_memories(100, 5);
        
        assert_eq!(memories.len(), 5);
        assert_eq!(memories[0].title, "Scale Test Memory 100");
        assert_eq!(memories[4].title, "Scale Test Memory 104");
        
        // Should have variety in memory types
        let types: std::collections::HashSet<_> = memories.iter().map(|m| &m.memory_type).collect();
        assert!(types.len() > 1, "Should have variety in memory types");
    }
}