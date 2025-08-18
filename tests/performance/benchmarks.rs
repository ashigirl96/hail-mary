use anyhow::{Context, Result};
use hail_mary::models::kiro::KiroConfig;
use hail_mary::models::memory::{Memory, MemoryType};
use hail_mary::repositories::memory::{MemoryRepository, SqliteMemoryRepository};
use hail_mary::services::memory::{MemoryInput, MemoryService};
use std::time::Instant;
use tempfile::TempDir;

/// Performance benchmarks for Memory MCP operations
/// Tests performance targets from design specification:
/// - remember < 50ms
/// - recall < 100ms  
/// - document generation < 1s

/// Test remember operation performance - target < 50ms
#[tokio::test]
async fn test_remember_under_50ms() -> Result<()> {
    let (_temp_dir, mut service) = setup_memory_service().await?;
    
    let memory_input = MemoryInput {
        memory_type: MemoryType::Tech,
        title: "Performance Test Memory".to_string(),
        content: "This is a test memory for performance benchmarking".to_string(),
        tags: vec!["performance".to_string(), "test".to_string()],
        confidence: Some(0.95),
    };
    
    // Warm up the system with a few operations
    for _ in 0..3 {
        service.remember_batch(vec![memory_input.clone()]).await?;
    }
    
    // Measure single memory save performance
    let start = Instant::now();
    let result = service.remember_batch(vec![memory_input]).await?;
    let elapsed = start.elapsed();
    
    assert_eq!(result.len(), 1, "Should save one memory");
    assert!(
        elapsed.as_millis() < 50,
        "Remember operation took {}ms, expected < 50ms",
        elapsed.as_millis()
    );
    
    println!("Remember performance: {}ms (target: <50ms)", elapsed.as_millis());
    Ok(())
}

/// Test recall operation performance - target < 100ms with 1000 memories
#[tokio::test]
async fn test_recall_under_100ms() -> Result<()> {
    let (_temp_dir, mut service) = setup_memory_service().await?;
    
    // Populate database with 1000 memories for realistic search conditions
    populate_large_dataset(&mut service, 1000).await?;
    
    // Warm up the search system
    for _ in 0..3 {
        service.recall("test", 10, None, vec![]).await?;
    }
    
    // Measure search performance
    let start = Instant::now();
    let result = service.recall("rust", 10, None, vec![]).await?;
    let elapsed = start.elapsed();
    
    assert!(!result.is_empty(), "Search should return results");
    assert!(
        elapsed.as_millis() < 100,
        "Recall operation took {}ms, expected < 100ms",
        elapsed.as_millis()
    );
    
    println!("Recall performance: {}ms (target: <100ms)", elapsed.as_millis());
    Ok(())
}

/// Test document generation performance - target < 1s with 1000 memories
#[tokio::test]
async fn test_document_under_1s() -> Result<()> {
    let (temp_dir, mut service) = setup_memory_service().await?;
    
    // Populate database with 1000 memories
    populate_large_dataset(&mut service, 1000).await?;
    
    // Create KiroConfig for document generation
    let config = create_test_config(temp_dir.path())?;
    
    // Warm up the system
    service.generate_documents(&config).await?;
    
    // Measure document generation performance
    let start = Instant::now();
    service.generate_documents(&config).await?;
    let elapsed = start.elapsed();
    
    assert!(
        elapsed.as_millis() < 1000,
        "Document generation took {}ms, expected < 1000ms",
        elapsed.as_millis()
    );
    
    println!("Document generation performance: {}ms (target: <1000ms)", elapsed.as_millis());
    Ok(())
}

/// Test batch save performance with various batch sizes
#[tokio::test]
async fn test_batch_save_performance() -> Result<()> {
    let (_temp_dir, mut service) = setup_memory_service().await?;
    
    let batch_sizes = [1, 10, 50, 100];
    
    for batch_size in batch_sizes {
        let memories = create_test_memories(batch_size);
        
        // Warm up
        service.remember_batch(memories.clone()).await?;
        
        // Measure batch save performance
        let start = Instant::now();
        let result = service.remember_batch(memories).await?;
        let elapsed = start.elapsed();
        
        assert_eq!(result.len(), batch_size, "Should save all memories in batch");
        
        let per_memory_ms = elapsed.as_millis() as f64 / batch_size as f64;
        assert!(
            per_memory_ms < 10.0,
            "Batch save of {} memories: {:.2}ms per memory (should be < 10ms)",
            batch_size,
            per_memory_ms
        );
        
        println!(
            "Batch save performance ({} memories): {}ms total, {:.2}ms per memory",
            batch_size,
            elapsed.as_millis(),
            per_memory_ms
        );
    }
    
    Ok(())
}

/// Test search performance with different query complexities
#[tokio::test]
async fn test_search_performance_complexity() -> Result<()> {
    let (_temp_dir, mut service) = setup_memory_service().await?;
    
    // Populate with diverse content for search testing
    populate_diverse_content(&mut service).await?;
    
    let test_queries = [
        ("rust", "Single term search"),
        ("rust async", "Two term search"),
        ("rust async programming tokio", "Multi-term search"),
        ("非同期", "Japanese single term"),
        ("Rust async プログラミング", "Mixed language search"),
    ];
    
    for (query, description) in test_queries {
        // Warm up
        service.recall(query, 10, None, vec![]).await?;
        
        // Measure search performance
        let start = Instant::now();
        let result = service.recall(query, 10, None, vec![]).await?;
        let elapsed = start.elapsed();
        
        assert!(
            elapsed.as_millis() < 150,
            "{} took {}ms, expected < 150ms",
            description,
            elapsed.as_millis()
        );
        
        println!(
            "{}: {}ms, {} results",
            description,
            elapsed.as_millis(),
            result.len()
        );
    }
    
    Ok(())
}

/// Test memory usage and database size constraints
#[tokio::test]
async fn test_memory_usage_constraints() -> Result<()> {
    let (temp_dir, mut service) = setup_memory_service().await?;
    
    // Populate with 10,000 memories to test scale limits
    populate_large_dataset(&mut service, 10000).await?;
    
    // Check database file size
    let db_path = temp_dir.path().join("db.sqlite3");
    let metadata = std::fs::metadata(&db_path)?;
    let size_mb = metadata.len() / (1024 * 1024);
    
    assert!(
        size_mb < 100,
        "Database size {}MB exceeds 100MB limit for 10,000 memories",
        size_mb
    );
    
    // Test that search performance is still acceptable with large dataset
    let start = Instant::now();
    let result = service.recall("performance", 10, None, vec![]).await?;
    let elapsed = start.elapsed();
    
    assert!(
        elapsed.as_millis() < 200,
        "Search with 10,000 memories took {}ms, expected < 200ms",
        elapsed.as_millis()
    );
    
    println!(
        "Scale test: 10,000 memories, {}MB database, search: {}ms",
        size_mb,
        elapsed.as_millis()
    );
    
    Ok(())
}

/// Test concurrent operation performance
#[tokio::test]
async fn test_concurrent_performance() -> Result<()> {
    let (_temp_dir, service) = setup_memory_service().await?;
    
    // Test concurrent recall operations
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let service = service.clone();
            tokio::spawn(async move {
                let start = Instant::now();
                let result = service.lock().await.recall("test", 5, None, vec![]).await;
                let elapsed = start.elapsed();
                (i, result, elapsed)
            })
        })
        .collect();
    
    // Wait for all operations to complete
    let mut total_time = 0u128;
    for handle in handles {
        let (i, result, elapsed) = handle.await.unwrap();
        result.unwrap_or_else(|e| panic!("Concurrent operation {} failed: {}", i, e));
        total_time += elapsed.as_millis();
        
        assert!(
            elapsed.as_millis() < 500,
            "Concurrent operation {} took {}ms, expected < 500ms",
            i,
            elapsed.as_millis()
        );
    }
    
    let avg_time = total_time / 10;
    println!("Concurrent operations: average {}ms per operation", avg_time);
    
    Ok(())
}

/// Performance regression test for reference count updates
#[tokio::test]
async fn test_reference_count_update_performance() -> Result<()> {
    let (_temp_dir, mut service) = setup_memory_service().await?;
    
    // Create some memories to search
    let memories = create_test_memories(100);
    service.remember_batch(memories).await?;
    
    // Test that reference count updates don't significantly impact search performance
    let start = Instant::now();
    let _result = service.recall("test", 10, None, vec![]).await?;
    let elapsed = start.elapsed();
    
    // Even with reference count updates, search should be fast
    assert!(
        elapsed.as_millis() < 100,
        "Search with reference count updates took {}ms, expected < 100ms",
        elapsed.as_millis()
    );
    
    println!("Search with reference updates: {}ms", elapsed.as_millis());
    Ok(())
}

// Helper functions

async fn setup_memory_service() -> Result<(TempDir, MemoryService<SqliteMemoryRepository>)> {
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
            instructions: "Performance test instructions".to_string(),
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

fn create_test_memories(count: usize) -> Vec<MemoryInput> {
    (0..count)
        .map(|i| MemoryInput {
            memory_type: match i % 3 {
                0 => MemoryType::Tech,
                1 => MemoryType::ProjectTech,
                _ => MemoryType::Domain,
            },
            title: format!("Test Memory {}", i),
            content: format!(
                "This is test memory number {} for performance testing. \
                 It contains some text about programming, rust, and performance optimization.",
                i
            ),
            tags: vec![
                "test".to_string(),
                "performance".to_string(),
                if i % 2 == 0 { "rust" } else { "programming" }.to_string(),
            ],
            confidence: Some(0.8 + (i as f32 * 0.01) % 0.2),
        })
        .collect()
}

async fn populate_large_dataset(
    service: &mut MemoryService<SqliteMemoryRepository>,
    count: usize,
) -> Result<()> {
    const BATCH_SIZE: usize = 100;
    
    for batch_start in (0..count).step_by(BATCH_SIZE) {
        let batch_end = std::cmp::min(batch_start + BATCH_SIZE, count);
        let batch_size = batch_end - batch_start;
        
        let memories = create_test_memories(batch_size);
        service.remember_batch(memories).await?;
    }
    
    Ok(())
}

async fn populate_diverse_content(
    service: &mut MemoryService<SqliteMemoryRepository>,
) -> Result<()> {
    let diverse_memories = vec![
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "Rust Async Programming".to_string(),
            content: "Rust async programming with tokio provides excellent performance for concurrent applications".to_string(),
            tags: vec!["rust".to_string(), "async".to_string(), "tokio".to_string()],
            confidence: Some(0.95),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "JavaScript非同期処理".to_string(),
            content: "JavaScriptの非同期プログラミングはPromiseとasync/awaitを使用します".to_string(),
            tags: vec!["javascript".to_string(), "非同期".to_string(), "async".to_string()],
            confidence: Some(0.88),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "Database Performance Optimization".to_string(),
            content: "Database performance can be improved through proper indexing, query optimization, and connection pooling".to_string(),
            tags: vec!["database".to_string(), "performance".to_string(), "optimization".to_string()],
            confidence: Some(0.92),
        },
        MemoryInput {
            memory_type: MemoryType::Domain,
            title: "Mixed Language Programming Documentation".to_string(),
            content: "Programming documentation often mixes English technical terms with local language explanations. For example: Rust async プログラミング provides 非同期 capabilities".to_string(),
            tags: vec!["documentation".to_string(), "mixed-language".to_string(), "programming".to_string()],
            confidence: Some(0.85),
        },
    ];
    
    service.remember_batch(diverse_memories).await?;
    Ok(())
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    
    #[test]
    fn test_memory_creation_performance() {
        let start = Instant::now();
        
        for _ in 0..1000 {
            let _memory = Memory::new(
                MemoryType::Tech,
                "Test Memory".to_string(),
                "Test content".to_string(),
            );
        }
        
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 10,
            "Memory creation took {}ms for 1000 objects, expected < 10ms",
            elapsed.as_millis()
        );
    }
    
    #[test]
    fn test_memory_type_parsing_performance() {
        let types = ["tech", "project-tech", "domain"];
        let start = Instant::now();
        
        for _ in 0..10000 {
            for type_str in &types {
                let _parsed: MemoryType = type_str.parse().unwrap();
            }
        }
        
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 50,
            "Memory type parsing took {}ms for 30,000 operations, expected < 50ms",
            elapsed.as_millis()
        );
    }
}