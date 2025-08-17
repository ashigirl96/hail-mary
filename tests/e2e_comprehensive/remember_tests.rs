// E2E tests for the remember tool

use anyhow::Result;
use rmcp::handler::server::tool::Parameters;
use std::path::PathBuf;

use super::utils::{
    PerformanceMeasure, TestDataGenerator, TestEnvironment, TestReporter, assert_remember_response,
    get_all_test_cases, input_to_remember_params, load_comprehensive_test_data,
};

/// Test all combinations from YAML test data
#[tokio::test]
async fn test_remember_all_yaml_combinations() -> Result<()> {
    // Initialize test environment
    let env = TestEnvironment::new().await?;
    let mut reporter = TestReporter::new();

    // Load test data
    let test_data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/e2e_comprehensive/data/comprehensive_test_data.yaml");
    let test_data = load_comprehensive_test_data(test_data_path)?;

    // Get all test cases
    let all_cases = get_all_test_cases(&test_data);
    println!("Running {} test cases from YAML data", all_cases.len());

    // Run each test case
    for test_case in all_cases {
        let params = input_to_remember_params(&test_case.input);
        let wrapped_params = Parameters(params);

        match env.server().remember(wrapped_params).await {
            Ok(response) => {
                assert_remember_response(&response.0, &test_case.expected, &test_case.test_id);
                reporter.record_pass(&test_case.test_id);
            }
            Err(e) => {
                reporter.record_fail(&test_case.test_id, &format!("Error: {}", e));
            }
        }
    }

    // Print summary and assert all passed
    reporter.print_summary();
    reporter.assert_all_passed();

    Ok(())
}

/// Test programmatically generated combinations
#[tokio::test]
async fn test_remember_generated_combinations() -> Result<()> {
    let env = TestEnvironment::new().await?;
    let mut reporter = TestReporter::new();

    // Generate test data
    let generator = TestDataGenerator::new();
    let test_cases = generator.generate_all_combinations();
    println!("Running {} generated test cases", test_cases.len());

    for test_case in test_cases {
        let params = input_to_remember_params(&test_case.input);
        let wrapped_params = Parameters(params);

        match env.server().remember(wrapped_params).await {
            Ok(response) => {
                assert_remember_response(&response.0, &test_case.expected, &test_case.test_id);
                reporter.record_pass(&test_case.test_id);
            }
            Err(e) => {
                reporter.record_fail(&test_case.test_id, &format!("Error: {}", e));
            }
        }
    }

    reporter.print_summary();
    reporter.assert_all_passed();

    Ok(())
}

/// Test duplicate detection and update behavior
#[tokio::test]
async fn test_remember_duplicate_detection() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Create first memory
    let params1 = Parameters(hail_mary::memory::models::RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Duplicate Test Title".to_string(),
        content: "Original content".to_string(),
        tags: Some(vec!["original".to_string()]),
        examples: None,
    });

    let response1 = env.server().remember(params1).await?;
    assert_eq!(response1.0.action, "created");
    let first_id = response1.0.memory_id.clone();

    // Try to create duplicate (same title and type)
    let params2 = Parameters(hail_mary::memory::models::RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Duplicate Test Title".to_string(),
        content: "Updated content".to_string(),
        tags: Some(vec!["updated".to_string()]),
        examples: None,
    });

    let response2 = env.server().remember(params2).await?;
    assert_eq!(response2.0.action, "updated");
    assert_eq!(
        response2.0.memory_id, first_id,
        "Should update the same memory"
    );

    // Verify the memory was updated by recalling it
    let recall_params = Parameters(hail_mary::memory::models::RmcpRecallParams {
        query: "Duplicate Test Title".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(1),
    });

    let recall_response = env.server().recall(recall_params).await?;
    assert_eq!(recall_response.0.memories.len(), 1);
    assert_eq!(recall_response.0.memories[0].content, "Updated content");
    assert_eq!(recall_response.0.memories[0].tags, vec!["updated"]);

    Ok(())
}

/// Test edge cases with special characters and Unicode
#[tokio::test]
async fn test_remember_special_characters() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Test Unicode content
    let unicode_params = Parameters(hail_mary::memory::models::RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Unicode Title 日本語 🚀".to_string(),
        content: "Content with emoji 😊 and Chinese 你好 and Arabic مرحبا".to_string(),
        tags: Some(vec!["unicode".to_string(), "国際化".to_string()]),
        examples: Some(vec![
            "let greeting = \"こんにちは\";".to_string(),
            "const emoji = \"🎉\";".to_string(),
        ]),
    });

    let response = env.server().remember(unicode_params).await?;
    assert_eq!(response.0.action, "created");

    // Test special characters that might cause SQL issues
    let special_params = Parameters(hail_mary::memory::models::RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Special'; DROP TABLE memories; --".to_string(),
        content: "Content with \"quotes\" and 'apostrophes' and <tags>".to_string(),
        tags: Some(vec!["@special".to_string(), "#hash".to_string()]),
        examples: None,
    });

    let response2 = env.server().remember(special_params).await?;
    assert_eq!(response2.0.action, "created");

    Ok(())
}

/// Test performance with large content
#[tokio::test]
async fn test_remember_large_content_performance() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Generate large content (10KB)
    let large_content = "x".repeat(10_000);
    let large_examples: Vec<String> = (0..10)
        .map(|i| format!("Example {} with some code content", i))
        .collect();

    let params = Parameters(hail_mary::memory::models::RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Large Content Test".to_string(),
        content: large_content,
        tags: Some(vec!["performance".to_string(), "large".to_string()]),
        examples: Some(large_examples),
    });

    // Measure performance
    let measure = PerformanceMeasure::start("remember_large_content");
    let response = env.server().remember(params).await?;
    measure.assert_under_ms(1000); // Should complete within 1 second

    assert_eq!(response.0.action, "created");

    Ok(())
}

/// Test empty and null field variations
#[tokio::test]
async fn test_remember_empty_fields() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Test with null optional fields
    let null_fields = Parameters(hail_mary::memory::models::RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Null Fields Test".to_string(),
        content: "Content with null optional fields".to_string(),
        tags: None,
        examples: None,
    });

    let response1 = env.server().remember(null_fields).await?;
    assert_eq!(response1.0.action, "created");

    // Test with empty arrays
    let empty_arrays = Parameters(hail_mary::memory::models::RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Empty Arrays Test".to_string(),
        content: "Content with empty arrays".to_string(),
        tags: Some(vec![]),
        examples: Some(vec![]),
    });

    let response2 = env.server().remember(empty_arrays).await?;
    assert_eq!(response2.0.action, "created");

    Ok(())
}

/// Test all memory type variations
#[tokio::test]
async fn test_remember_all_memory_types() -> Result<()> {
    let env = TestEnvironment::new().await?;

    let memory_types = vec!["tech", "project-tech", "domain"];

    for memory_type in memory_types {
        let params = Parameters(hail_mary::memory::models::RmcpRememberParams {
            r#type: memory_type.to_string(),
            title: format!("Test {} Memory", memory_type),
            content: format!("Content for {} type", memory_type),
            tags: Some(vec![memory_type.to_string()]),
            examples: None,
        });

        let response = env.server().remember(params).await?;
        assert_eq!(
            response.0.action, "created",
            "Failed to create memory of type {}",
            memory_type
        );
    }

    Ok(())
}

/// Test batch memory creation performance
#[tokio::test]
async fn test_remember_batch_performance() -> Result<()> {
    let env = TestEnvironment::new().await?;
    let count = 100;

    let measure = PerformanceMeasure::start(format!("remember_{}_memories", count));

    for i in 0..count {
        let params = Parameters(hail_mary::memory::models::RmcpRememberParams {
            r#type: "tech".to_string(),
            title: format!("Batch Test Memory {}", i),
            content: format!("Content for batch test {}", i),
            tags: Some(vec![format!("batch-{}", i / 10)]),
            examples: None,
        });

        let response = env.server().remember(params).await?;
        assert_eq!(response.0.action, "created");
    }

    measure.assert_under_ms(5000); // 100 memories should be created within 5 seconds

    Ok(())
}

/// Test content validation boundaries
#[tokio::test]
async fn test_remember_content_boundaries() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Test minimum content (1 character)
    let min_content = Parameters(hail_mary::memory::models::RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Min Content".to_string(),
        content: "x".to_string(),
        tags: None,
        examples: None,
    });

    let response1 = env.server().remember(min_content).await?;
    assert_eq!(response1.0.action, "created");

    // Test very long title
    let long_title = "x".repeat(500);
    let long_title_params = Parameters(hail_mary::memory::models::RmcpRememberParams {
        r#type: "tech".to_string(),
        title: long_title,
        content: "Content with very long title".to_string(),
        tags: None,
        examples: None,
    });

    let response2 = env.server().remember(long_title_params).await?;
    assert_eq!(response2.0.action, "created");

    // Test many tags
    let many_tags: Vec<String> = (0..50).map(|i| format!("tag-{}", i)).collect();
    let many_tags_params = Parameters(hail_mary::memory::models::RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Many Tags Test".to_string(),
        content: "Content with many tags".to_string(),
        tags: Some(many_tags),
        examples: None,
    });

    let response3 = env.server().remember(many_tags_params).await?;
    assert_eq!(response3.0.action, "created");

    Ok(())
}

/// Test concurrent memory creation
#[tokio::test]
async fn test_remember_concurrent_creation() -> Result<()> {
    let env = TestEnvironment::new().await?;
    let server = env.server().clone();

    // Create multiple memories concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let server_clone = server.clone();
        let handle = tokio::spawn(async move {
            let params = Parameters(hail_mary::memory::models::RmcpRememberParams {
                r#type: "tech".to_string(),
                title: format!("Concurrent Memory {}", i),
                content: format!("Concurrent content {}", i),
                tags: Some(vec!["concurrent".to_string()]),
                examples: None,
            });

            server_clone.remember(params).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut success_count = 0;
    for handle in handles {
        match handle.await? {
            Ok(_) => success_count += 1,
            Err(e) => eprintln!("Concurrent creation failed: {}", e),
        }
    }

    assert_eq!(success_count, 10, "All concurrent creations should succeed");

    Ok(())
}
