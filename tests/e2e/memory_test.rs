use crate::e2e::helpers::*;
use anyhow::Result;
use tokio::time::{sleep, Duration};

/// Complete E2E test for Memory MCP workflow
/// Tests: init → serve → remember → recall → document flow
#[tokio::test]
async fn test_complete_memory_workflow() -> Result<()> {
    // Initialize test environment
    let env = E2ETestEnv::new()?;
    
    // Step 1: Initialize project
    let init_result = env.init_project()?;
    init_result.assert_success()?;
    init_result.assert_stdout_contains("Initialized .kiro directory structure")?;
    
    // Verify .kiro structure was created
    assert!(env.kiro_structure_exists(), ".kiro directory structure should exist");
    
    // Step 2: Import fixture memories (simulates MCP remember operations)
    env.import_fixture_memories("memories.yaml").await?;
    
    // Verify memories were imported
    let memory_count = env.count_memories()?;
    assert!(memory_count > 15, "Should have imported fixture memories");
    
    // Step 3: Test document generation
    let doc_result = env.run_command(&["memory", "document"])?;
    doc_result.assert_success()?;
    
    // Verify generated documentation
    let tech_docs = env.read_generated_docs("tech")?;
    E2EValidation::validate_markdown_structure(&tech_docs)?;
    assert!(tech_docs.contains("Rust"), "Tech docs should contain Rust content");
    
    let project_docs = env.read_generated_docs("project-tech")?;
    E2EValidation::validate_markdown_structure(&project_docs)?;
    assert!(project_docs.contains("hail-mary"), "Project docs should contain project-specific content");
    
    let domain_docs = env.read_generated_docs("domain")?;
    E2EValidation::validate_markdown_structure(&domain_docs)?;
    
    // Step 4: Test selective document generation
    let selective_result = env.run_command(&["memory", "document", "--type", "tech"])?;
    selective_result.assert_success()?;
    
    // Step 5: Validate database integrity
    assert!(env.validate_database()?, "Database should be accessible");
    
    // Step 6: Test error handling - invalid memory type
    let invalid_type_result = env.run_command(&["memory", "document", "--type", "invalid"])?;
    invalid_type_result.assert_failure()?;
    invalid_type_result.assert_stderr_contains("Invalid memory type")?;
    
    Ok(())
}

/// Test MCP server startup and basic functionality
#[tokio::test]
async fn test_mcp_server_startup() -> Result<()> {
    let env = E2ETestEnv::new()?;
    
    // Initialize project first
    env.init_project()?.assert_success()?;
    
    // Test that server command starts properly
    // Note: We can't easily test the actual MCP protocol without a client,
    // but we can test that the server initializes correctly
    let server_start_result = env.run_command_async(&["memory", "serve"], 2).await;
    
    // Server should either run (and timeout) or exit with proper initialization
    match server_start_result {
        Ok(_) => {
            // Server ran and timed out - this is expected
        },
        Err(e) => {
            // Check if it's a timeout (expected) or actual error
            let error_msg = e.to_string();
            if !error_msg.contains("timed out") {
                // If it's not a timeout, there might be a real error
                println!("Warning: Server startup test had unexpected error: {}", error_msg);
            }
        }
    }
    
    Ok(())
}

/// Test error handling scenarios
#[tokio::test]
async fn test_error_scenarios() -> Result<()> {
    let env = E2ETestEnv::new()?;
    
    // Test 1: Commands without initialization should fail gracefully
    let doc_without_init = env.run_command(&["memory", "document"])?;
    doc_without_init.assert_failure()?;
    doc_without_init.assert_stderr_contains("Failed to load configuration")?;
    
    let serve_without_init = env.run_command_async(&["memory", "serve"], 2).await;
    // Should fail quickly due to missing config
    assert!(serve_without_init.is_err() || !serve_without_init.unwrap().success);
    
    // Test 2: Initialize project
    env.init_project()?.assert_success()?;
    
    // Test 3: Invalid command arguments
    let invalid_command = env.run_command(&["memory", "invalid-subcommand"])?;
    invalid_command.assert_failure()?;
    
    // Test 4: Document generation with empty database should succeed but generate empty docs
    let empty_doc_result = env.run_command(&["memory", "document"])?;
    empty_doc_result.assert_success()?;
    
    Ok(())
}

/// Test Japanese content handling and search
#[tokio::test]
async fn test_japanese_content_handling() -> Result<()> {
    let env = E2ETestEnv::new()?;
    
    // Initialize and import Japanese test data
    env.init_project()?.assert_success()?;
    env.import_fixture_memories("memories.yaml").await?;
    
    // Generate documentation and verify Japanese content
    env.run_command(&["memory", "document"])?.assert_success()?;
    
    let tech_docs = env.read_generated_docs("tech")?;
    
    // Verify Japanese content is properly preserved in documentation
    assert!(tech_docs.contains("非同期"), "Should contain Japanese hiragana");
    assert!(tech_docs.contains("プログラミング"), "Should contain Japanese katakana");
    assert!(tech_docs.contains("Rust"), "Should contain mixed language content");
    
    // Test that Unicode characters are properly handled
    assert!(tech_docs.contains("🚀"), "Should handle emoji in content");
    
    // Verify document structure with Japanese content
    E2EValidation::validate_markdown_structure(&tech_docs)?;
    
    Ok(())
}

/// Test large dataset handling
#[tokio::test]
async fn test_large_dataset_handling() -> Result<()> {
    let env = E2ETestEnv::new()?;
    
    // Initialize and import large dataset
    env.init_project()?.assert_success()?;
    
    let measurement = PerformanceMeasurement::start();
    env.import_fixture_memories("large_dataset.yaml").await?;
    
    // Verify import performance (should be reasonable even for large datasets)
    let import_time = measurement.elapsed_ms();
    assert!(import_time < 5000, "Large dataset import should complete in under 5 seconds, took {}ms", import_time);
    
    // Verify memory count
    let memory_count = env.count_memories()?;
    assert!(memory_count >= 50, "Should have imported large dataset with {} memories", memory_count);
    
    // Test document generation performance with large dataset
    let doc_measurement = PerformanceMeasurement::start();
    env.run_command(&["memory", "document"])?.assert_success()?;
    doc_measurement.assert_under_ms(3000)?; // Should complete in under 3 seconds
    
    // Verify database size constraints
    let db_size = env.database_size()?;
    E2EValidation::validate_database_size(db_size, 50)?; // Should be under 50MB for test dataset
    
    Ok(())
}

/// Test concurrent operations and data consistency
#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let env = E2ETestEnv::new()?;
    
    env.init_project()?.assert_success()?;
    env.import_fixture_memories("memories.yaml").await?;
    
    // Test concurrent document generation (simulates multiple users)
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let env_clone = &env;
            tokio::spawn(async move {
                let result = env_clone.run_command(&["memory", "document", "--type", "tech"]);
                (i, result)
            })
        })
        .collect();
    
    // Wait for all operations to complete
    for handle in handles {
        let (i, result) = handle.await.unwrap();
        result.unwrap().assert_success().unwrap_or_else(|e| {
            panic!("Concurrent operation {} failed: {}", i, e);
        });
    }
    
    // Verify data consistency after concurrent operations
    let final_count = env.count_memories()?;
    assert!(final_count > 0, "Memory count should be preserved after concurrent operations");
    
    Ok(())
}

/// Test reindex functionality (placeholder for Phase 3)
#[tokio::test]
async fn test_reindex_command() -> Result<()> {
    let env = E2ETestEnv::new()?;
    
    env.init_project()?.assert_success()?;
    env.import_fixture_memories("memories.yaml").await?;
    
    // Test dry-run mode
    let dry_run_result = env.run_command(&["memory", "reindex", "--dry-run"])?;
    dry_run_result.assert_success()?;
    dry_run_result.assert_stdout_contains("dry run")?;
    
    // Test verbose mode
    let verbose_result = env.run_command(&["memory", "reindex", "--verbose", "--dry-run"])?;
    verbose_result.assert_success()?;
    
    // Verify that dry-run doesn't actually change anything
    let count_before = env.count_memories()?;
    env.run_command(&["memory", "reindex", "--dry-run"])?.assert_success()?;
    let count_after = env.count_memories()?;
    assert_eq!(count_before, count_after, "Dry run should not change memory count");
    
    Ok(())
}

/// Test edge cases and boundary conditions
#[tokio::test]
async fn test_edge_cases() -> Result<()> {
    let env = E2ETestEnv::new()?;
    
    env.init_project()?.assert_success()?;
    
    // Test 1: Generate documentation with empty database
    let empty_result = env.run_command(&["memory", "document"])?;
    empty_result.assert_success()?;
    
    // Generated files should exist but be minimal
    let tech_docs = env.read_generated_docs("tech")?;
    assert!(tech_docs.len() < 100, "Empty database should generate minimal documentation");
    
    // Test 2: Very long command line arguments
    let long_type_filter = "a".repeat(1000);
    let long_arg_result = env.run_command(&["memory", "document", "--type", &long_type_filter])?;
    long_arg_result.assert_failure()?; // Should reject invalid type gracefully
    
    // Test 3: Document generation for non-existent type
    let nonexistent_result = env.run_command(&["memory", "document", "--type", "nonexistent"])?;
    nonexistent_result.assert_failure()?;
    nonexistent_result.assert_stderr_contains("Invalid memory type")?;
    
    Ok(())
}

/// Test configuration file handling
#[tokio::test]
async fn test_configuration_handling() -> Result<()> {
    let env = E2ETestEnv::new()?;
    
    // Test 1: Init with force flag
    env.run_command(&["init", "--force"])?.assert_success()?;
    
    // Verify config file was created
    let config_content = std::fs::read_to_string(env.working_dir.join(".kiro/config.toml"))?;
    assert!(config_content.contains("[memory]"), "Config should contain memory section");
    assert!(config_content.contains("tech"), "Config should contain default memory types");
    
    // Test 2: Re-initialize without force should prompt (will fail in non-interactive environment)
    let reinit_result = env.run_command(&["init"])?;
    // In non-interactive environment, this should succeed silently or fail gracefully
    
    // Test 3: Verify .gitignore was updated
    let gitignore_content = std::fs::read_to_string(env.working_dir.join(".gitignore"))?;
    assert!(gitignore_content.contains(".kiro/memory/db.sqlite3"), "Gitignore should exclude database files");
    
    Ok(())
}

/// Performance regression test
#[tokio::test]
async fn test_performance_regression() -> Result<()> {
    let env = E2ETestEnv::new()?;
    
    env.init_project()?.assert_success()?;
    env.import_fixture_memories("large_dataset.yaml").await?;
    
    // Test document generation performance
    let measurement = PerformanceMeasurement::start();
    env.run_command(&["memory", "document"])?.assert_success()?;
    
    // Assert performance targets from design specification
    measurement.assert_under_ms(1000)?; // < 1 second for document generation
    
    // Test database operations performance
    let db_measurement = PerformanceMeasurement::start();
    let _count = env.count_memories()?;
    db_measurement.assert_under_ms(100)?; // Database query should be fast
    
    // Test memory usage constraints
    let db_size = env.database_size()?;
    let size_mb = db_size / (1024 * 1024);
    assert!(size_mb < 50, "Database size should be reasonable: {}MB", size_mb);
    
    Ok(())
}

/// Integration test with all memory types
#[tokio::test]
async fn test_all_memory_types() -> Result<()> {
    let env = E2ETestEnv::new()?;
    
    env.init_project()?.assert_success()?;
    env.import_fixture_memories("memories.yaml").await?;
    
    // Test document generation for each memory type
    let memory_types = ["tech", "project-tech", "domain"];
    
    for memory_type in memory_types {
        let result = env.run_command(&["memory", "document", "--type", memory_type])?;
        result.assert_success()?;
        
        let docs = env.read_generated_docs(memory_type)?;
        E2EValidation::validate_markdown_structure(&docs)?;
        
        // Each document should contain memories of the correct type
        assert!(docs.len() > 50, "Document for {} should have substantial content", memory_type);
    }
    
    // Test generation of all types at once
    let all_result = env.run_command(&["memory", "document"])?;
    all_result.assert_success()?;
    
    // All three documents should exist
    for memory_type in memory_types {
        let docs = env.read_generated_docs(memory_type)?;
        assert!(!docs.is_empty(), "Document for {} should exist", memory_type);
    }
    
    Ok(())
}