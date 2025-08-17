use hail_mary::mcp::server::MemoryMcpServer;
use hail_mary::memory::models::{RmcpRecallParams, RmcpRememberParams};
use rmcp::handler::server::tool::Parameters;
use tempfile::tempdir;

#[tokio::test]
async fn test_mcp_remember_and_recall() {
    // Setup: Create temporary database
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_mcp.db");
    let server = MemoryMcpServer::new(&db_path).unwrap();

    // Test 1: Remember a memory
    let remember_params = RmcpRememberParams {
        r#type: "tech".to_string(),
        topic: "MCP Test Memory".to_string(),
        content: "This is a test memory for MCP server functionality".to_string(),
        tags: Some(vec![
            "mcp".to_string(),
            "test".to_string(),
            "tech".to_string(),
        ]),
        examples: Some(vec!["Example usage".to_string()]),
        source: Some("Test source".to_string()),
    };

    let remember_result = server.remember(Parameters(remember_params)).await;
    assert!(remember_result.is_ok(), "Remember should succeed");

    let remember_response = remember_result.unwrap();
    println!(
        "Remember response: memory_id={}, action={}",
        remember_response.0.memory_id, remember_response.0.action
    );

    // Test 2: Recall with query only (no type filter)
    let recall_params_no_type = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: None,
        tags: None,
        limit: Some(10),
    };

    let recall_result_no_type = server.recall(Parameters(recall_params_no_type)).await;
    assert!(
        recall_result_no_type.is_ok(),
        "Recall without type should succeed"
    );

    let recall_response_no_type = recall_result_no_type.unwrap();
    println!(
        "Recall without type: found {} memories",
        recall_response_no_type.0.memories.len()
    );
    assert!(
        !recall_response_no_type.0.memories.is_empty(),
        "Should find memories with MCP query"
    );

    // Test 3: Recall with query AND type filter
    let recall_params_with_type = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(10),
    };

    let recall_result_with_type = server.recall(Parameters(recall_params_with_type)).await;
    assert!(
        recall_result_with_type.is_ok(),
        "Recall with type should succeed"
    );

    let recall_response_with_type = recall_result_with_type.unwrap();
    println!(
        "Recall with type: found {} memories",
        recall_response_with_type.0.memories.len()
    );
    assert!(
        !recall_response_with_type.0.memories.is_empty(),
        "Should find tech memories with MCP query"
    );

    // Test 4: Verify the returned memory matches what we stored
    let found_memory = &recall_response_with_type.0.memories[0];
    assert_eq!(found_memory.topic, "MCP Test Memory");
    assert_eq!(
        found_memory.content,
        "This is a test memory for MCP server functionality"
    );
    assert!(found_memory.tags.contains(&"mcp".to_string()));
    assert!(found_memory.tags.contains(&"test".to_string()));

    // Test 5: Test with wrong type - should return empty
    let recall_params_wrong_type = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("domain".to_string()), // Wrong type
        tags: None,
        limit: Some(10),
    };

    let recall_result_wrong_type = server.recall(Parameters(recall_params_wrong_type)).await;
    assert!(
        recall_result_wrong_type.is_ok(),
        "Recall with wrong type should succeed"
    );

    let recall_response_wrong_type = recall_result_wrong_type.unwrap();
    println!(
        "Recall with wrong type: found {} memories",
        recall_response_wrong_type.0.memories.len()
    );
    assert!(
        recall_response_wrong_type.0.memories.is_empty(),
        "Should find no memories with wrong type"
    );
}

#[tokio::test]
async fn test_mcp_recall_existing_data() {
    // Test against existing database with real data
    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("hail-mary")
        .join("memory.db");

    // Only run if database exists
    if !db_path.exists() {
        println!("Skipping test - no existing database found");
        return;
    }

    let server = MemoryMcpServer::new(&db_path).unwrap();

    // Test 1: Query existing MCP data without type filter
    let recall_params_no_type = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: None,
        tags: None,
        limit: Some(10),
    };

    let recall_result_no_type = server.recall(Parameters(recall_params_no_type)).await;
    assert!(
        recall_result_no_type.is_ok(),
        "Recall without type should succeed"
    );

    let recall_response_no_type = recall_result_no_type.unwrap();
    println!(
        "Existing data - Recall without type: found {} memories",
        recall_response_no_type.0.memories.len()
    );

    for memory in &recall_response_no_type.0.memories {
        let content_preview = if memory.content.len() > 50 {
            format!(
                "{}...",
                &memory.content.chars().take(47).collect::<String>()
            )
        } else {
            memory.content.clone()
        };
        println!(
            "  - {} ({}): {}",
            memory.topic, memory.memory_type, content_preview
        );
    }

    // Test 2: Query existing MCP data WITH tech type filter
    let recall_params_with_type = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(10),
    };

    let recall_result_with_type = server.recall(Parameters(recall_params_with_type)).await;
    assert!(
        recall_result_with_type.is_ok(),
        "Recall with type should succeed"
    );

    let recall_response_with_type = recall_result_with_type.unwrap();
    println!(
        "Existing data - Recall with tech type: found {} memories",
        recall_response_with_type.0.memories.len()
    );

    for memory in &recall_response_with_type.0.memories {
        let content_preview = if memory.content.len() > 50 {
            format!(
                "{}...",
                &memory.content.chars().take(47).collect::<String>()
            )
        } else {
            memory.content.clone()
        };
        println!(
            "  - {} ({}): {}",
            memory.topic, memory.memory_type, content_preview
        );
    }

    // Test 3: Compare results
    println!(
        "Without type filter: {} results",
        recall_response_no_type.0.memories.len()
    );
    println!(
        "With tech type filter: {} results",
        recall_response_with_type.0.memories.len()
    );

    // All tech memories from no-type should be included in with-type results
    let tech_memories_from_no_type: Vec<_> = recall_response_no_type
        .0
        .memories
        .iter()
        .filter(|m| m.memory_type.to_string() == "tech")
        .collect();

    println!(
        "Tech memories from no-type filter: {}",
        tech_memories_from_no_type.len()
    );
    assert_eq!(
        tech_memories_from_no_type.len(),
        recall_response_with_type.0.memories.len(),
        "Tech type filter should return same number as tech memories from unfiltered search"
    );
}

#[tokio::test]
async fn test_debug_type_conversion() {
    use hail_mary::memory::models::{MemoryType, RecallParams};

    // Test the conversion from RmcpRecallParams to RecallParams
    let rmcp_params = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(10),
    };

    let recall_params: RecallParams = rmcp_params.into();

    println!("Original type: {:?}", "tech");
    println!("Converted memory_type: {:?}", recall_params.memory_type);
    println!("Expected: {:?}", Some(MemoryType::Tech));

    assert_eq!(recall_params.query, "MCP");
    assert_eq!(recall_params.memory_type, Some(MemoryType::Tech));
    assert_eq!(recall_params.limit, Some(10));
}

