use hail_mary::memory::{
    models::{MemoryType, RecallParams},
    repository::SqliteMemoryRepository,
    service::MemoryService,
};
use tempfile::tempdir;

#[tokio::test]
async fn test_recall_function_direct() {
    // Setup: Create temporary database and service
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test_recall.db");
    let repository = SqliteMemoryRepository::new(&db_path).unwrap();
    let mut service = MemoryService::new(repository);

    // Add test data first
    let remember_params = hail_mary::memory::models::RememberParams {
        memory_type: MemoryType::Tech,
        title: "MCP Server Test".to_string(),
        content: "Testing MCP server functionality with recall function".to_string(),
        tags: Some(vec![
            "mcp".to_string(),
            "test".to_string(),
            "server".to_string(),
        ]),
        examples: Some(vec!["Example MCP usage".to_string()]),
    };

    let remember_result = service.remember(remember_params).await;
    assert!(remember_result.is_ok(), "Remember should succeed");

    // Test 1: Recall with query "MCP" and type "tech"
    let recall_params = RecallParams {
        query: "MCP".to_string(),
        memory_type: Some(MemoryType::Tech),
        tags: None,
        limit: Some(10),
    };

    let recall_result = service.recall(recall_params).await;
    assert!(recall_result.is_ok(), "Recall should succeed");

    let recall_response = recall_result.unwrap();
    println!(
        "Direct recall test - Found {} memories",
        recall_response.memories.len()
    );

    // Verify we found the memory
    assert!(
        !recall_response.memories.is_empty(),
        "Should find at least one memory"
    );
    assert_eq!(recall_response.total_count, recall_response.memories.len());

    // Check the found memory
    let found_memory = &recall_response.memories[0];
    assert_eq!(found_memory.title, "MCP Server Test");
    assert_eq!(found_memory.memory_type, MemoryType::Tech);
    assert!(found_memory.content.contains("MCP"));
    assert!(found_memory.tags.contains(&"mcp".to_string()));

    println!(
        "Found memory: {} ({})",
        found_memory.title, found_memory.memory_type
    );
    println!("Content: {}", found_memory.content);
}

#[tokio::test]
async fn test_recall_with_existing_database() {
    // Test against the real database
    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("hail-mary")
        .join("memory.db");

    // Only run if database exists
    if !db_path.exists() {
        println!(
            "Skipping test - no existing database found at {:?}",
            db_path
        );
        return;
    }

    let repository = SqliteMemoryRepository::new(&db_path).unwrap();
    let mut service = MemoryService::new(repository);

    // Test the specific case: query "MCP" with type "tech"
    let recall_params = RecallParams {
        query: "MCP".to_string(),
        memory_type: Some(MemoryType::Tech),
        tags: None,
        limit: Some(10),
    };

    println!("Testing recall with params: query='MCP', type=tech");

    let recall_result = service.recall(recall_params).await;
    assert!(
        recall_result.is_ok(),
        "Recall should succeed: {:?}",
        recall_result.err()
    );

    let recall_response = recall_result.unwrap();
    println!(
        "Real database recall test - Found {} memories",
        recall_response.memories.len()
    );
    println!("Total count: {}", recall_response.total_count);

    // Print details of found memories
    for (i, memory) in recall_response.memories.iter().enumerate() {
        println!(
            "  Memory {}: {} ({})",
            i + 1,
            memory.title,
            memory.memory_type
        );
        println!("    Tags: {:?}", memory.tags);
        let content_preview = if memory.content.len() > 100 {
            format!(
                "{}...",
                &memory.content.chars().take(97).collect::<String>()
            )
        } else {
            memory.content.clone()
        };
        println!("    Content: {}", content_preview);
    }

    // Verify all found memories are tech type
    for memory in &recall_response.memories {
        assert_eq!(
            memory.memory_type,
            MemoryType::Tech,
            "All returned memories should be tech type, but found: {}",
            memory.memory_type
        );
    }
}

#[tokio::test]
async fn test_recall_comparison_with_without_type() {
    // Test against the real database
    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("hail-mary")
        .join("memory.db");

    if !db_path.exists() {
        println!("Skipping comparison test - no existing database found");
        return;
    }

    let repository = SqliteMemoryRepository::new(&db_path).unwrap();
    let mut service = MemoryService::new(repository);

    // Test 1: Recall WITHOUT type filter
    let recall_params_no_type = RecallParams {
        query: "MCP".to_string(),
        memory_type: None,
        tags: None,
        limit: Some(10),
    };

    let recall_result_no_type = service.recall(recall_params_no_type).await;
    assert!(
        recall_result_no_type.is_ok(),
        "Recall without type should succeed"
    );
    let recall_response_no_type = recall_result_no_type.unwrap();

    // Test 2: Recall WITH tech type filter
    let recall_params_with_type = RecallParams {
        query: "MCP".to_string(),
        memory_type: Some(MemoryType::Tech),
        tags: None,
        limit: Some(10),
    };

    let recall_result_with_type = service.recall(recall_params_with_type).await;
    assert!(
        recall_result_with_type.is_ok(),
        "Recall with type should succeed"
    );
    let recall_response_with_type = recall_result_with_type.unwrap();

    // Compare results
    println!("Comparison test results:");
    println!(
        "  Without type filter: {} memories",
        recall_response_no_type.memories.len()
    );
    println!(
        "  With tech type filter: {} memories",
        recall_response_with_type.memories.len()
    );

    // Count tech memories from no-type search
    let tech_count_no_type = recall_response_no_type
        .memories
        .iter()
        .filter(|m| m.memory_type == MemoryType::Tech)
        .count();

    println!("  Tech memories in no-type search: {}", tech_count_no_type);

    // The number of tech memories from no-type should equal the with-type results
    assert_eq!(
        tech_count_no_type,
        recall_response_with_type.memories.len(),
        "Tech type filter should return same count as tech memories from unfiltered search"
    );

    // Verify all memories in with-type search are indeed tech
    for memory in &recall_response_with_type.memories {
        assert_eq!(memory.memory_type, MemoryType::Tech);
    }

    // Print memory details for debugging
    println!("\nMemories found without type filter:");
    for memory in &recall_response_no_type.memories {
        println!("  - {} ({})", memory.title, memory.memory_type);
    }

    println!("\nMemories found with tech type filter:");
    for memory in &recall_response_with_type.memories {
        println!("  - {} ({})", memory.title, memory.memory_type);
    }
}

#[tokio::test]
async fn test_recall_type_conversion() {
    // Test the MemoryType conversion
    use hail_mary::memory::models::RmcpRecallParams;

    let rmcp_params = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(10),
    };

    let recall_params: RecallParams = rmcp_params.into();

    println!("Type conversion test:");
    println!("  Original r#type: {:?}", "tech");
    println!("  Converted memory_type: {:?}", recall_params.memory_type);

    assert_eq!(recall_params.query, "MCP");
    assert_eq!(recall_params.memory_type, Some(MemoryType::Tech));
    assert_eq!(recall_params.limit, Some(10));

    // Test with invalid type
    let rmcp_params_invalid = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("invalid".to_string()),
        tags: None,
        limit: Some(10),
    };

    let recall_params_invalid: RecallParams = rmcp_params_invalid.into();
    println!(
        "  Invalid type conversion: {:?}",
        recall_params_invalid.memory_type
    );
    assert_eq!(
        recall_params_invalid.memory_type, None,
        "Invalid type should convert to None"
    );
}
