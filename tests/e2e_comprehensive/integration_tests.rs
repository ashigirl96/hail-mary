// Integration tests for full remember -> recall -> delete workflows

use anyhow::Result;
use rmcp::handler::server::tool::Parameters;

use super::utils::{PerformanceMeasure, TestEnvironment, TestReporter};
use hail_mary::memory::models::{RmcpDeleteParams, RmcpRecallParams, RmcpRememberParams};

/// Test complete workflow: remember -> recall -> update -> recall -> delete
#[tokio::test]
async fn test_complete_memory_lifecycle() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Step 1: Create a memory
    let create_params = Parameters(RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Lifecycle Test Memory".to_string(),
        content: "Original content for lifecycle test".to_string(),
        tags: Some(vec!["lifecycle".to_string(), "test".to_string()]),
        examples: Some(vec!["let x = 1;".to_string()]),
    });

    let create_response = env.server().remember(create_params).await?;
    assert_eq!(create_response.0.action, "created");
    let memory_id = create_response.0.memory_id.clone();

    // Step 2: Recall the memory
    let recall_params = Parameters(RmcpRecallParams {
        query: "Lifecycle Test Memory".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(1),
    });

    let recall_response = env.server().recall(recall_params.clone()).await?;
    assert_eq!(recall_response.0.memories.len(), 1);
    assert_eq!(recall_response.0.memories[0].id, memory_id);
    assert_eq!(
        recall_response.0.memories[0].content,
        "Original content for lifecycle test"
    );

    // Step 3: Update the memory
    let update_params = Parameters(RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Lifecycle Test Memory".to_string(), // Same title and type
        content: "Updated content for lifecycle test".to_string(),
        tags: Some(vec!["lifecycle".to_string(), "updated".to_string()]),
        examples: Some(vec!["let x = 2;".to_string(), "let y = 3;".to_string()]),
    });

    let update_response = env.server().remember(update_params).await?;
    assert_eq!(update_response.0.action, "updated");
    assert_eq!(update_response.0.memory_id, memory_id); // Should be same ID

    // Step 4: Recall updated memory
    let recall_updated = env.server().recall(recall_params).await?;
    assert_eq!(recall_updated.0.memories.len(), 1);
    assert_eq!(
        recall_updated.0.memories[0].content,
        "Updated content for lifecycle test"
    );
    assert!(
        recall_updated.0.memories[0]
            .tags
            .contains(&"updated".to_string())
    );
    assert_eq!(recall_updated.0.memories[0].examples.len(), 2);

    // Step 5: Delete the memory
    let delete_params = Parameters(RmcpDeleteParams {
        memory_id: memory_id.clone(),
    });

    let delete_response = env.server().delete_memory(delete_params).await?;
    assert!(delete_response.0.deleted);
    assert_eq!(delete_response.0.memory_id, memory_id);

    // Step 6: Verify deletion (should not find the memory)
    let recall_deleted = env
        .server()
        .recall(Parameters(RmcpRecallParams {
            query: "Lifecycle Test Memory".to_string(),
            r#type: Some("tech".to_string()),
            tags: None,
            limit: None,
        }))
        .await?;

    assert_eq!(
        recall_deleted.0.memories.len(),
        0,
        "Deleted memory should not be found"
    );

    Ok(())
}

/// Test data consistency across operations
#[tokio::test]
async fn test_data_consistency() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Create multiple related memories
    let memories = vec![
        RmcpRememberParams {
            r#type: "tech".to_string(),
            title: "Consistency Test 1".to_string(),
            content: "First memory for consistency test".to_string(),
            tags: Some(vec!["consistency".to_string(), "group1".to_string()]),
            examples: None,
        },
        RmcpRememberParams {
            r#type: "tech".to_string(),
            title: "Consistency Test 2".to_string(),
            content: "Second memory for consistency test".to_string(),
            tags: Some(vec!["consistency".to_string(), "group1".to_string()]),
            examples: None,
        },
        RmcpRememberParams {
            r#type: "project-tech".to_string(),
            title: "Consistency Test 3".to_string(),
            content: "Third memory for consistency test".to_string(),
            tags: Some(vec!["consistency".to_string(), "group2".to_string()]),
            examples: None,
        },
    ];

    let mut memory_ids = Vec::new();

    // Store all memories
    for memory_params in memories {
        let params = Parameters(memory_params);
        let response = env.server().remember(params).await?;
        memory_ids.push(response.0.memory_id);
    }

    // Verify all memories can be recalled by tag
    let tag_recall = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: Some(vec!["consistency".to_string()]),
        limit: None,
    });

    let tag_response = env.server().recall(tag_recall).await?;
    assert_eq!(
        tag_response.0.memories.len(),
        3,
        "Should find all 3 consistency memories"
    );

    // Verify filtering by type and tag
    let filtered_recall = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: Some("tech".to_string()),
        tags: Some(vec!["group1".to_string()]),
        limit: None,
    });

    let filtered_response = env.server().recall(filtered_recall).await?;
    assert_eq!(
        filtered_response.0.memories.len(),
        2,
        "Should find 2 tech memories with group1 tag"
    );

    // Delete one memory and verify others remain
    let delete_params = Parameters(RmcpDeleteParams {
        memory_id: memory_ids[0].clone(),
    });
    env.server().delete_memory(delete_params).await?;

    // Recall remaining memories
    let remaining_recall = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: Some(vec!["consistency".to_string()]),
        limit: None,
    });

    let remaining_response = env.server().recall(remaining_recall).await?;
    assert_eq!(
        remaining_response.0.memories.len(),
        2,
        "Should find 2 remaining memories"
    );

    Ok(())
}

/// Test cross-type queries
#[tokio::test]
async fn test_cross_type_queries() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Create memories of different types with similar content
    let memories = vec![
        (
            "tech",
            "Async Programming Guide",
            "Understanding async patterns in Rust",
        ),
        (
            "project-tech",
            "Project Async Implementation",
            "How we implement async in our project",
        ),
        (
            "domain",
            "Async Business Requirements",
            "Business needs for asynchronous processing",
        ),
    ];

    for (mem_type, title, content) in memories {
        let params = Parameters(RmcpRememberParams {
            r#type: mem_type.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            tags: Some(vec!["async".to_string()]),
            examples: None,
        });
        env.server().remember(params).await?;
    }

    // Query across all types
    let cross_type_query = Parameters(RmcpRecallParams {
        query: "async".to_string(),
        r#type: None, // No type filter
        tags: None,
        limit: None,
    });

    let response = env.server().recall(cross_type_query).await?;

    // Should find memories from all types
    assert!(
        response.0.memories.len() >= 3,
        "Should find async memories from all types"
    );

    // Verify we have memories from different types
    let types: Vec<String> = response
        .0
        .memories
        .iter()
        .map(|m| m.memory_type.to_string())
        .collect();

    assert!(types.contains(&"tech".to_string()));
    assert!(types.contains(&"project-tech".to_string()));
    assert!(types.contains(&"domain".to_string()));

    Ok(())
}

/// Test tag-based categorization and filtering
#[tokio::test]
async fn test_tag_categorization() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Create memories with overlapping tag sets
    let tag_sets = vec![
        vec!["rust", "backend"],
        vec!["rust", "frontend"],
        vec!["javascript", "frontend"],
        vec!["rust", "backend", "api"],
        vec!["python", "backend", "api"],
    ];

    for (i, tags) in tag_sets.iter().enumerate() {
        let params = Parameters(RmcpRememberParams {
            r#type: "tech".to_string(),
            title: format!("Tag Test Memory {}", i),
            content: format!("Content for tag test {}", i),
            tags: Some(tags.iter().map(|s| s.to_string()).collect()),
            examples: None,
        });
        env.server().remember(params).await?;
    }

    // Test single tag queries
    let rust_query = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: Some(vec!["rust".to_string()]),
        limit: None,
    });

    let rust_response = env.server().recall(rust_query).await?;
    assert_eq!(
        rust_response.0.memories.len(),
        3,
        "Should find 3 rust memories"
    );

    // Test multiple tag queries (AND logic)
    let rust_backend_query = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: Some(vec!["rust".to_string(), "backend".to_string()]),
        limit: None,
    });

    let rust_backend_response = env.server().recall(rust_backend_query).await?;
    assert_eq!(
        rust_backend_response.0.memories.len(),
        2,
        "Should find 2 rust+backend memories"
    );

    Ok(())
}

/// Test performance with realistic workload
#[tokio::test]
async fn test_realistic_workload_performance() -> Result<()> {
    let env = TestEnvironment::new().await?;
    let mut reporter = TestReporter::new();

    // Simulate realistic workload over time
    let _workload = vec![
        ("Create technical documentation", 20),
        ("Query recent memories", 30),
        ("Update existing memories", 10),
        ("Search by tags", 25),
        ("Delete old memories", 5),
    ];

    let total_measure = PerformanceMeasure::start("complete_workload");

    // Create initial dataset
    for i in 0..50 {
        let params = Parameters(RmcpRememberParams {
            r#type: match i % 3 {
                0 => "tech",
                1 => "project-tech",
                _ => "domain",
            }
            .to_string(),
            title: format!("Workload Memory {}", i),
            content: format!("Content for workload test memory {}", i),
            tags: Some(vec![
                format!("category-{}", i % 5),
                format!("priority-{}", if i % 2 == 0 { "high" } else { "low" }),
            ]),
            examples: if i % 3 == 0 {
                Some(vec![format!("example code {}", i)])
            } else {
                None
            },
        });

        if env.server().remember(params).await.is_ok() {
            reporter.record_pass(&format!("create_{}", i));
        } else {
            reporter.record_fail(&format!("create_{}", i), "Failed to create memory");
        }
    }

    // Perform various queries
    let queries = vec![
        ("", None, None),                                           // Browse all
        ("workload", None, None),                                   // Text search
        ("", Some("tech"), None),                                   // Type filter
        ("", None, Some(vec!["priority-high"])),                    // Tag filter
        ("memory", Some("project-tech"), Some(vec!["category-1"])), // Combined
    ];

    for (i, (query, type_filter, tag_filter)) in queries.into_iter().enumerate() {
        let params = Parameters(RmcpRecallParams {
            query: query.to_string(),
            r#type: type_filter.map(|s| s.to_string()),
            tags: tag_filter.map(|v| v.iter().map(|s| s.to_string()).collect()),
            limit: Some(10),
        });

        let measure = PerformanceMeasure::start(format!("query_{}", i));
        if env.server().recall(params).await.is_ok() {
            measure.assert_under_ms(200); // Each query should be fast
            reporter.record_pass(&format!("query_{}", i));
        } else {
            reporter.record_fail(&format!("query_{}", i), "Query failed");
        }
    }

    total_measure.assert_under_ms(10000); // Complete workload under 10 seconds

    reporter.print_summary();

    Ok(())
}

/// Test data integrity after concurrent operations
#[tokio::test]
async fn test_concurrent_operations_integrity() -> Result<()> {
    let env = TestEnvironment::new().await?;
    let server = env.server().clone();

    // Spawn concurrent operations
    let mut create_handles = vec![];
    let mut recall_handles = vec![];

    // Concurrent creates
    for i in 0..5 {
        let server_clone = server.clone();
        create_handles.push(tokio::spawn(async move {
            let params = Parameters(RmcpRememberParams {
                r#type: "tech".to_string(),
                title: format!("Concurrent Memory {}", i),
                content: format!("Concurrent content {}", i),
                tags: Some(vec!["concurrent".to_string()]),
                examples: None,
            });
            server_clone.remember(params).await
        }));
    }

    // Concurrent recalls
    for _ in 0..5 {
        let server_clone = server.clone();
        recall_handles.push(tokio::spawn(async move {
            let params = Parameters(RmcpRecallParams {
                query: "concurrent".to_string(),
                r#type: None,
                tags: None,
                limit: Some(10),
            });
            server_clone.recall(params).await
        }));
    }

    // Wait for all operations
    for handle in create_handles {
        handle.await??;
    }
    for handle in recall_handles {
        handle.await??;
    }

    // Verify final state
    let final_recall = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: Some(vec!["concurrent".to_string()]),
        limit: None,
    });

    let final_response = env.server().recall(final_recall).await?;
    assert_eq!(
        final_response.0.memories.len(),
        5,
        "Should have exactly 5 concurrent memories"
    );

    // Verify each memory is unique
    let titles: Vec<String> = final_response
        .0
        .memories
        .iter()
        .map(|m| m.title.clone())
        .collect();

    let unique_titles: std::collections::HashSet<_> = titles.iter().collect();
    assert_eq!(
        unique_titles.len(),
        5,
        "All memories should have unique titles"
    );

    Ok(())
}

/// Test edge cases in full workflow
#[tokio::test]
async fn test_workflow_edge_cases() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Test 1: Update non-existent memory (should create)
    let update_new = Parameters(RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Non-existent Memory".to_string(),
        content: "This should create a new memory".to_string(),
        tags: None,
        examples: None,
    });

    let response1 = env.server().remember(update_new).await?;
    assert_eq!(
        response1.0.action, "created",
        "Should create when memory doesn't exist"
    );

    // Test 2: Delete already deleted memory (should handle gracefully)
    let delete_params = Parameters(RmcpDeleteParams {
        memory_id: "non-existent-id".to_string(),
    });

    // This might return an error, but should not panic
    let delete_result = env.server().delete_memory(delete_params).await;
    assert!(delete_result.is_err() || !delete_result?.0.deleted);

    // Test 3: Recall with all filters but no matches
    let no_match_recall = Parameters(RmcpRecallParams {
        query: "impossible query xyz123".to_string(),
        r#type: Some("domain".to_string()),
        tags: Some(vec!["non-existent-tag".to_string()]),
        limit: Some(1),
    });

    let no_match_response = env.server().recall(no_match_recall).await?;
    assert_eq!(
        no_match_response.0.memories.len(),
        0,
        "Should return empty for no matches"
    );

    Ok(())
}
