use hail_mary::mcp::server::MemoryMcpServer;
use hail_mary::memory::models::{RmcpRecallParams, RmcpRememberParams};
use rmcp::handler::server::tool::Parameters;
use tempfile::tempdir;

#[tokio::test]
async fn test_mcp_recall_with_empty_tags_array() {
    // Setup: Create temporary database
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_edge_case.db");
    let server = MemoryMcpServer::new(&db_path).unwrap();

    // Add test memory
    let remember_params = RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Empty Tags Test".to_string(),
        content: "Testing empty tags array handling in MCP server".to_string(),
        tags: Some(vec!["test".to_string(), "edge-case".to_string()]),
        examples: None,
    };

    let remember_result = server.remember(Parameters(remember_params)).await;
    assert!(remember_result.is_ok(), "Remember should succeed");

    // Test: Recall with empty tags array (the problematic case)
    let recall_params_empty_tags = RmcpRecallParams {
        query: "Empty".to_string(),
        r#type: Some("tech".to_string()),
        tags: Some(vec![]), // Empty array - this was causing the bug
        limit: Some(10),
    };

    let recall_result = server.recall(Parameters(recall_params_empty_tags)).await;
    assert!(
        recall_result.is_ok(),
        "Recall with empty tags should succeed"
    );

    let recall_response = recall_result.unwrap();
    println!(
        "Recall with empty tags array: found {} memories",
        recall_response.0.memories.len()
    );

    // Should find the memory despite empty tags array
    assert!(
        !recall_response.0.memories.is_empty(),
        "Should find memories when tags array is empty (not filtered)"
    );

    let found_memory = &recall_response.0.memories[0];
    assert_eq!(found_memory.title, "Empty Tags Test");
}

#[tokio::test]
async fn test_mcp_recall_with_no_tags_vs_empty_tags() {
    // Setup: Create temporary database
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_tags_comparison.db");
    let server = MemoryMcpServer::new(&db_path).unwrap();

    // Add test memory
    let remember_params = RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Tags Comparison Test".to_string(),
        content: "Testing tags vs no tags behavior".to_string(),
        tags: Some(vec!["test".to_string()]),
        examples: None,
    };

    let remember_result = server.remember(Parameters(remember_params)).await;
    assert!(remember_result.is_ok(), "Remember should succeed");

    // Test 1: Recall with no tags field (None)
    let recall_params_no_tags = RmcpRecallParams {
        query: "Tags".to_string(),
        r#type: Some("tech".to_string()),
        tags: None, // No tags field
        limit: Some(10),
    };

    let recall_result_no_tags = server.recall(Parameters(recall_params_no_tags)).await;
    assert!(
        recall_result_no_tags.is_ok(),
        "Recall with no tags should succeed"
    );
    let response_no_tags = recall_result_no_tags.unwrap();

    // Test 2: Recall with empty tags array
    let recall_params_empty_tags = RmcpRecallParams {
        query: "Tags".to_string(),
        r#type: Some("tech".to_string()),
        tags: Some(vec![]), // Empty tags array
        limit: Some(10),
    };

    let recall_result_empty_tags = server.recall(Parameters(recall_params_empty_tags)).await;
    assert!(
        recall_result_empty_tags.is_ok(),
        "Recall with empty tags should succeed"
    );
    let response_empty_tags = recall_result_empty_tags.unwrap();

    // Both should return the same results
    assert_eq!(
        response_no_tags.0.memories.len(),
        response_empty_tags.0.memories.len(),
        "No tags and empty tags should return same number of results"
    );

    println!("No tags: {} memories", response_no_tags.0.memories.len());
    println!(
        "Empty tags: {} memories",
        response_empty_tags.0.memories.len()
    );
}

#[tokio::test]
async fn test_mcp_recall_with_actual_tag_filter() {
    // Setup: Create temporary database
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_actual_tags.db");
    let server = MemoryMcpServer::new(&db_path).unwrap();

    // Add memories with different tags
    let remember_params1 = RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Rust Memory".to_string(),
        content: "Memory about Rust programming".to_string(),
        tags: Some(vec!["rust".to_string(), "programming".to_string()]),
        examples: None,
    };

    let remember_params2 = RmcpRememberParams {
        r#type: "tech".to_string(),
        title: "Python Memory".to_string(),
        content: "Memory about Python programming".to_string(),
        tags: Some(vec!["python".to_string(), "programming".to_string()]),
        examples: None,
    };

    server.remember(Parameters(remember_params1)).await.unwrap();
    server.remember(Parameters(remember_params2)).await.unwrap();

    // Test: Recall with actual tag filter
    let recall_params_rust_tag = RmcpRecallParams {
        query: "Memory".to_string(),
        r#type: Some("tech".to_string()),
        tags: Some(vec!["rust".to_string()]), // Actual tag filter
        limit: Some(10),
    };

    let recall_result = server.recall(Parameters(recall_params_rust_tag)).await;
    assert!(recall_result.is_ok(), "Recall with rust tag should succeed");

    let recall_response = recall_result.unwrap();
    println!(
        "Recall with rust tag: found {} memories",
        recall_response.0.memories.len()
    );

    // Should find only Rust memory
    assert_eq!(
        recall_response.0.memories.len(),
        1,
        "Should find exactly 1 rust memory"
    );
    assert_eq!(recall_response.0.memories[0].title, "Rust Memory");
}
