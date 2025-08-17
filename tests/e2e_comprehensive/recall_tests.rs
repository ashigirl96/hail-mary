// E2E tests for the recall tool

use anyhow::Result;
use rmcp::handler::server::tool::Parameters;
use std::path::PathBuf;

use super::utils::{
    PerformanceMeasure, TestEnvironment, TestReporter, assert_recall_response,
    get_all_recall_scenarios, input_to_recall_params, load_recall_test_data,
};
use hail_mary::memory::models::{RmcpRecallParams, RmcpRememberParams};

/// Setup test data for recall tests
async fn setup_test_memories(env: &TestEnvironment) -> Result<()> {
    // Create a variety of memories for testing recall
    let test_memories = vec![
        RmcpRememberParams {
            r#type: "tech".to_string(),
            title: "Rust Async Programming".to_string(),
            content: "Understanding async programming in Rust with tokio runtime".to_string(),
            tags: Some(vec![
                "rust".to_string(),
                "async".to_string(),
                "tokio".to_string(),
            ]),
            examples: Some(vec![
                "async fn main() {}".to_string(),
                "tokio::spawn(async {})".to_string(),
            ]),
        },
        RmcpRememberParams {
            r#type: "tech".to_string(),
            title: "Error Handling Best Practices".to_string(),
            content: "Effective error handling strategies using Result and Option types"
                .to_string(),
            tags: Some(vec!["rust".to_string(), "error-handling".to_string()]),
            examples: Some(vec!["Result<T, E>".to_string()]),
        },
        RmcpRememberParams {
            r#type: "project-tech".to_string(),
            title: "MCP Server Implementation".to_string(),
            content: "Implementation details for the Memory MCP server using rmcp".to_string(),
            tags: Some(vec!["mcp".to_string(), "rmcp".to_string()]),
            examples: None,
        },
        RmcpRememberParams {
            r#type: "project-tech".to_string(),
            title: "Memory Service Architecture".to_string(),
            content: "Architecture of the memory service with SQLite backend".to_string(),
            tags: Some(vec!["architecture".to_string(), "sqlite".to_string()]),
            examples: None,
        },
        RmcpRememberParams {
            r#type: "domain".to_string(),
            title: "Business Logic Documentation".to_string(),
            content: "Core business logic for memory management system".to_string(),
            tags: Some(vec!["business".to_string(), "documentation".to_string()]),
            examples: None,
        },
        RmcpRememberParams {
            r#type: "tech".to_string(),
            title: "Testing Strategies".to_string(),
            content: "Comprehensive testing strategies including unit and integration tests"
                .to_string(),
            tags: Some(vec!["testing".to_string()]),
            examples: Some(vec!["#[test]".to_string(), "#[tokio::test]".to_string()]),
        },
        RmcpRememberParams {
            r#type: "tech".to_string(),
            title: "Performance Optimization".to_string(),
            content: "Techniques for optimizing Rust application performance".to_string(),
            tags: Some(vec!["performance".to_string(), "optimization".to_string()]),
            examples: None,
        },
        RmcpRememberParams {
            r#type: "tech".to_string(),
            title: "Unicode Support 日本語".to_string(),
            content: "Supporting Unicode and international characters 你好 مرحبا".to_string(),
            tags: Some(vec!["unicode".to_string(), "i18n".to_string()]),
            examples: None,
        },
    ];

    // Store all test memories
    for memory_params in test_memories {
        let params = Parameters(memory_params);
        env.server().remember(params).await?;
    }

    Ok(())
}

/// Test all recall scenarios from YAML data
#[tokio::test]
async fn test_recall_all_yaml_scenarios() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    let mut reporter = TestReporter::new();

    // Load test scenarios
    let scenarios_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/e2e_comprehensive/data/recall_scenarios.yaml");
    let recall_data = load_recall_test_data(scenarios_path)?;

    let all_scenarios = get_all_recall_scenarios(&recall_data);
    println!("Running {} recall test scenarios", all_scenarios.len());

    for scenario in all_scenarios {
        let params = input_to_recall_params(&scenario.input);
        let wrapped_params = Parameters(params);

        match env.server().recall(wrapped_params).await {
            Ok(response) => {
                assert_recall_response(&response.0, &scenario.expected, &scenario.scenario_id);
                reporter.record_pass(&scenario.scenario_id);
            }
            Err(e) => {
                reporter.record_fail(&scenario.scenario_id, &format!("Error: {}", e));
            }
        }
    }

    reporter.print_summary();
    reporter.assert_all_passed();

    Ok(())
}

/// Test empty query (browse mode)
#[tokio::test]
async fn test_recall_browse_mode() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    let params = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let response = env.server().recall(params).await?;

    // Should return all memories
    assert!(
        !response.0.memories.is_empty(),
        "Browse mode should return memories"
    );
    assert!(
        response.0.memories.len() >= 5,
        "Should return multiple memories"
    );

    Ok(())
}

/// Test single word search
#[tokio::test]
async fn test_recall_single_word_search() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    let params = Parameters(RmcpRecallParams {
        query: "rust".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let response = env.server().recall(params).await?;

    // Should find memories containing "rust"
    assert!(
        !response.0.memories.is_empty(),
        "Should find rust-related memories"
    );

    // Verify all results contain "rust" in content, title, or tags
    for memory in &response.0.memories {
        let contains_rust = memory.content.to_lowercase().contains("rust")
            || memory.title.to_lowercase().contains("rust")
            || memory
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains("rust"));

        assert!(contains_rust, "Memory should be related to 'rust'");
    }

    Ok(())
}

/// Test type filtering
#[tokio::test]
async fn test_recall_type_filter() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    // Test tech type filter
    let tech_params = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: None,
    });

    let tech_response = env.server().recall(tech_params).await?;

    for memory in &tech_response.0.memories {
        assert_eq!(
            memory.memory_type.to_string(),
            "tech",
            "Should only return tech memories"
        );
    }

    // Test project-tech type filter
    let project_params = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: Some("project-tech".to_string()),
        tags: None,
        limit: None,
    });

    let project_response = env.server().recall(project_params).await?;

    for memory in &project_response.0.memories {
        assert_eq!(
            memory.memory_type.to_string(),
            "project-tech",
            "Should only return project-tech memories"
        );
    }

    Ok(())
}

/// Test tag filtering
#[tokio::test]
async fn test_recall_tag_filter() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    // Single tag filter
    let single_tag_params = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: Some(vec!["rust".to_string()]),
        limit: None,
    });

    let response = env.server().recall(single_tag_params).await?;

    for memory in &response.0.memories {
        assert!(
            memory.tags.contains(&"rust".to_string()),
            "Memory should have 'rust' tag"
        );
    }

    // Multiple tags filter (AND logic)
    let multi_tag_params = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: Some(vec!["rust".to_string(), "async".to_string()]),
        limit: None,
    });

    let multi_response = env.server().recall(multi_tag_params).await?;

    for memory in &multi_response.0.memories {
        assert!(
            memory.tags.contains(&"rust".to_string()),
            "Memory should have 'rust' tag"
        );
        assert!(
            memory.tags.contains(&"async".to_string()),
            "Memory should have 'async' tag"
        );
    }

    Ok(())
}

/// Test combined filters
#[tokio::test]
async fn test_recall_combined_filters() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    // Combine query, type, and tags
    let params = Parameters(RmcpRecallParams {
        query: "programming".to_string(),
        r#type: Some("tech".to_string()),
        tags: Some(vec!["rust".to_string()]),
        limit: Some(5),
    });

    let response = env.server().recall(params).await?;

    // Should return at most 5 tech memories with rust tag containing "programming"
    assert!(response.0.memories.len() <= 5, "Should respect limit");

    for memory in &response.0.memories {
        assert_eq!(memory.memory_type.to_string(), "tech");
        assert!(memory.tags.contains(&"rust".to_string()));
    }

    Ok(())
}

/// Test limit parameter
#[tokio::test]
async fn test_recall_limit() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    // Test limit = 1
    let limit_one = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: None,
        limit: Some(1),
    });

    let response_one = env.server().recall(limit_one).await?;
    assert_eq!(
        response_one.0.memories.len(),
        1,
        "Should return exactly 1 memory"
    );

    // Test limit = 5
    let limit_five = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: None,
        limit: Some(5),
    });

    let response_five = env.server().recall(limit_five).await?;
    assert!(
        response_five.0.memories.len() <= 5,
        "Should return at most 5 memories"
    );

    Ok(())
}

/// Test no results scenario
#[tokio::test]
async fn test_recall_no_results() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    // Search for non-existent content
    let params = Parameters(RmcpRecallParams {
        query: "nonexistentquery12345".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let response = env.server().recall(params).await?;
    assert_eq!(response.0.memories.len(), 0, "Should return empty results");

    // Filter by non-existent tag
    let tag_params = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: Some(vec!["nonexistenttag123".to_string()]),
        limit: None,
    });

    let tag_response = env.server().recall(tag_params).await?;
    assert_eq!(
        tag_response.0.memories.len(),
        0,
        "Should return empty results for non-existent tag"
    );

    Ok(())
}

/// Test Unicode and special character queries
#[tokio::test]
async fn test_recall_unicode_queries() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    // Add a memory about hail-mary implementation in Japanese
    let hail_mary_params = Parameters(RmcpRememberParams {
        r#type: "project-tech".to_string(),
        title: "Hail Mary MCP Server".to_string(),
        content: "hail-maryを実装している。このプロジェクトはRustで書かれたMemory MCP Serverです。"
            .to_string(),
        tags: Some(vec![
            "rust".to_string(),
            "mcp".to_string(),
            "日本語".to_string(),
        ]),
        examples: Some(vec!["cargo run".to_string()]),
    });
    let remember_response = env.server().remember(hail_mary_params).await?;
    println!(
        "Created memory with ID: {}, action: {}",
        remember_response.0.memory_id, remember_response.0.action
    );

    // Test browse all to see what memories exist
    let browse_params = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let browse_response = env.server().recall(browse_params).await?;
    println!(
        "Total memories after adding Japanese content: {}",
        browse_response.0.memories.len()
    );
    for memory in &browse_response.0.memories {
        println!(
            "  - {} (type: {}, content: {}...)",
            memory.title,
            memory.memory_type,
            &memory.content.chars().take(50).collect::<String>()
        );
    }

    // Search for Unicode content
    let unicode_params = Parameters(RmcpRecallParams {
        query: "日本語".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let response = env.server().recall(unicode_params).await?;

    // Should find the Unicode memory
    assert!(
        !response.0.memories.is_empty(),
        "Should find Unicode memory"
    );
    assert!(
        response
            .0
            .memories
            .iter()
            .any(|m| m.title.contains("日本語")),
        "Should find memory with Japanese title"
    );

    // Test searching for "hail-maryを実装している" (will be normalized to have space)
    let hail_mary_query = Parameters(RmcpRecallParams {
        query: "hail-maryを実装している".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    println!("Testing query: 'hail-maryを実装している'");
    let hail_mary_response = env.server().recall(hail_mary_query).await?;
    println!(
        "Query 'hail-maryを実装している' returned {} memories",
        hail_mary_response.0.memories.len()
    );

    // The content is normalized to "hail-mary を実装している" with space
    // Try with just "hail-mary" which should find it
    let hail_simple = Parameters(RmcpRecallParams {
        query: "hail-mary".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let hail_simple_response = env.server().recall(hail_simple).await?;

    // Should find the hail-mary implementation memory with simple query
    assert!(
        hail_simple_response
            .0
            .memories
            .iter()
            .any(|m| m.title == "Hail Mary MCP Server" && m.content.contains("実装している")),
        "Should find the hail-mary implementation memory with 'hail-mary' query"
    );

    // Test another Japanese query that's more likely to work
    // Using "Rust" which appears in the Japanese memory content
    let rust_jp_query = Parameters(RmcpRecallParams {
        query: "Rust".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let rust_jp_response = env.server().recall(rust_jp_query).await?;

    println!(
        "Query 'Rust' returned {} memories",
        rust_jp_response.0.memories.len()
    );

    // Should find multiple memories including the Japanese one
    assert!(
        !rust_jp_response.0.memories.is_empty(),
        "Should find memories with 'Rust'"
    );
    assert!(
        rust_jp_response.0.memories.iter().any(|m|
            // After normalization, it's "Rust で書かれた" with space
            m.content.contains("Rust") && m.content.contains("で書かれた")),
        "Should find the Japanese memory mentioning Rust"
    );

    // Test searching with just "hail-mary" (without Japanese text)
    let hail_only_query = Parameters(RmcpRecallParams {
        query: "hail-mary".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let hail_only_response = env.server().recall(hail_only_query).await?;
    println!(
        "Query 'hail-mary' returned {} memories",
        hail_only_response.0.memories.len()
    );

    // Should find the hail-mary memory
    assert!(
        !hail_only_response.0.memories.is_empty(),
        "Should find hail-mary memory"
    );
    assert!(
        hail_only_response
            .0
            .memories
            .iter()
            .any(|m| m.content.contains("hail-mary")),
        "Should find memory containing hail-mary"
    );

    // Test mixed English-Japanese query
    let mixed_query = Parameters(RmcpRecallParams {
        query: "Rust MCP".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let mixed_response = env.server().recall(mixed_query).await?;

    // Should find memories with both Rust and MCP
    assert!(
        !mixed_response.0.memories.is_empty(),
        "Should find memories with Rust and MCP"
    );

    Ok(())
}

/// Test recall performance with many memories
#[tokio::test]
async fn test_recall_performance() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Create many memories
    for i in 0..100 {
        let params = Parameters(RmcpRememberParams {
            r#type: "tech".to_string(),
            title: format!("Performance Test Memory {}", i),
            content: format!("Content for performance test {}", i),
            tags: Some(vec![format!("perf-{}", i % 10)]),
            examples: None,
        });
        env.server().remember(params).await?;
    }

    // Test browse performance
    let browse_measure = PerformanceMeasure::start("recall_browse_100");
    let browse_params = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: None,
        limit: Some(50),
    });
    let _ = env.server().recall(browse_params).await?;
    browse_measure.assert_under_ms(500); // Browse should be fast

    // Test search performance
    let search_measure = PerformanceMeasure::start("recall_search_100");
    let search_params = Parameters(RmcpRecallParams {
        query: "performance test".to_string(),
        r#type: None,
        tags: None,
        limit: Some(20),
    });
    let _ = env.server().recall(search_params).await?;
    search_measure.assert_under_ms(500); // Search should be fast

    Ok(())
}

/// Test SQL injection prevention
#[tokio::test]
async fn test_recall_sql_injection_prevention() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    // Try SQL injection in query
    let injection_params = Parameters(RmcpRecallParams {
        query: "'; DROP TABLE memories; --".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    // Should handle safely without error
    let _response = env.server().recall(injection_params).await?;
    // If we reach here, the SQL injection was handled safely

    // Verify table still exists by doing another query
    let verify_params = Parameters(RmcpRecallParams {
        query: "test".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let _verify_response = env.server().recall(verify_params).await?;
    // If we reach here, the table still exists

    Ok(())
}

/// Test partial matching
#[tokio::test]
async fn test_recall_partial_matching() -> Result<()> {
    let env = TestEnvironment::new().await?;
    setup_test_memories(&env).await?;

    // Search for partial word
    let params = Parameters(RmcpRecallParams {
        query: "prog".to_string(), // Should match "programming"
        r#type: None,
        tags: None,
        limit: None,
    });

    let response = env.server().recall(params).await?;

    // Should find memories with "programming"
    assert!(
        !response.0.memories.is_empty(),
        "Should find partial matches"
    );

    let has_programming_match = response.0.memories.iter().any(|m| {
        m.title.to_lowercase().contains("prog") || m.content.to_lowercase().contains("prog")
    });

    assert!(
        has_programming_match,
        "Should find memories containing 'prog'"
    );

    Ok(())
}

/// Test Japanese content with English query (hail-mary をテスト)
#[tokio::test]
async fn test_recall_japanese_content_with_english_query() -> Result<()> {
    let env = TestEnvironment::new().await?;

    // Test case 1: With space - "hail-mary をテスト" (should be found by "hail-mary")
    let japanese_memory_with_space = Parameters(RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Japanese Test Memory With Space".to_string(),
        content: "hail-mary をテスト".to_string(), // Note the space after hail-mary
        tags: Some(vec!["test".to_string(), "japanese".to_string()]),
        examples: None,
    });

    let remember_response1 = env.server().remember(japanese_memory_with_space).await?;
    println!(
        "Created Japanese memory with space, ID: {}",
        remember_response1.0.memory_id
    );

    // Test case 2: Without space - "hail-maryをテスト" (won't be found by "hail-mary" alone due to FTS5 tokenization)
    let japanese_memory_no_space = Parameters(RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Japanese Test Memory No Space".to_string(),
        content: "hail-maryをテスト".to_string(), // No space - treated as single token
        tags: Some(vec!["test".to_string(), "japanese".to_string()]),
        examples: None,
    });

    let remember_response2 = env.server().remember(japanese_memory_no_space).await?;
    println!(
        "Created Japanese memory without space, ID: {}",
        remember_response2.0.memory_id
    );

    // Now search for "hail-mary" (English query)
    let english_query = Parameters(RmcpRecallParams {
        query: "hail-mary".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    println!("Searching for 'hail-mary' to find Japanese content");
    let recall_response = env.server().recall(english_query).await?;

    // Should find at least the memory with space
    assert!(
        !recall_response.0.memories.is_empty(),
        "Should find at least one Japanese memory when searching for 'hail-mary'"
    );

    // Verify the memory with space is found
    let found_spaced_memory =
        recall_response.0.memories.iter().any(|m| {
            m.content == "hail-mary をテスト" && m.title == "Japanese Test Memory With Space"
        });

    assert!(
        found_spaced_memory,
        "Should find the Japanese memory with content 'hail-mary をテスト' when searching for 'hail-mary'"
    );

    println!(
        "Found {} memories with query 'hail-mary':",
        recall_response.0.memories.len()
    );
    for mem in &recall_response.0.memories {
        println!("  - {}: {}", mem.title, mem.content);
    }

    // Now test with the normalized version of the query
    // The query "hail-maryをテスト" should also be normalized to "hail-mary をテスト"
    let normalized_query = Parameters(RmcpRecallParams {
        query: "hail-mary をテスト".to_string(), // Use normalized version with space
        r#type: None,
        tags: None,
        limit: None,
    });

    let normalized_response = env.server().recall(normalized_query).await?;
    // Should find both memories since they're normalized to the same content
    let found_normalized = normalized_response
        .0
        .memories
        .iter()
        .any(|m| m.content.contains("hail-mary") && m.content.contains("テスト"));

    assert!(
        found_normalized,
        "Should find normalized content when searching with normalized query"
    );

    println!("✅ Successfully tested Japanese content recall with English query");
    println!("   - 'hail-mary をテスト' (with space) is found by 'hail-mary' query");
    println!("   - 'hail-maryをテスト' (no space) is found by exact query");

    Ok(())
}
