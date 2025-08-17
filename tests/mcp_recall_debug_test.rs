use hail_mary::mcp::server::MemoryMcpServer;
use hail_mary::memory::models::RmcpRecallParams;
use rmcp::handler::server::tool::Parameters;

#[tokio::test]
async fn test_mcp_recall_debug() {
    // Test against the real database
    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("hail-mary")
        .join("memory.db");

    if !db_path.exists() {
        println!("Skipping test - no existing database found");
        return;
    }

    let server = MemoryMcpServer::new(&db_path).unwrap();

    // Test the exact MCP call: query "MCP" with type "tech"
    let recall_params = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(10),
    };

    println!("=== MCP Server Recall Debug Test ===");
    println!("Input params:");
    println!("  query: {:?}", recall_params.query);
    println!("  type: {:?}", recall_params.r#type);
    println!("  tags: {:?}", recall_params.tags);
    println!("  limit: {:?}", recall_params.limit);

    let recall_result = server.recall(Parameters(recall_params)).await;

    match recall_result {
        Ok(response) => {
            println!(
                "SUCCESS - MCP recall returned {} memories",
                response.0.memories.len()
            );
            println!("Total count: {}", response.0.total_count);

            for (i, memory) in response.0.memories.iter().enumerate() {
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
        }
        Err(error) => {
            println!("ERROR - MCP recall failed: {:?}", error);
            panic!("MCP recall should succeed but failed with: {:?}", error);
        }
    }
}

#[tokio::test]
async fn test_mcp_recall_param_conversion() {
    // Test the parameter conversion in isolation
    use hail_mary::memory::models::{MemoryType, RecallParams};

    let rmcp_params = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(10),
    };

    println!("=== Parameter Conversion Test ===");
    println!("RMCP params:");
    println!("  query: {:?}", rmcp_params.query);
    println!("  r#type: {:?}", rmcp_params.r#type);

    let recall_params: RecallParams = rmcp_params.into();

    println!("Converted RecallParams:");
    println!("  query: {:?}", recall_params.query);
    println!("  memory_type: {:?}", recall_params.memory_type);
    println!("  tags: {:?}", recall_params.tags);
    println!("  limit: {:?}", recall_params.limit);

    // Verify conversion
    assert_eq!(recall_params.query, "MCP");
    assert_eq!(recall_params.memory_type, Some(MemoryType::Tech));
    assert_eq!(recall_params.tags, None);
    assert_eq!(recall_params.limit, Some(10));

    println!("✅ Parameter conversion is correct");
}

#[tokio::test]
async fn test_mcp_vs_direct_comparison() {
    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("hail-mary")
        .join("memory.db");

    if !db_path.exists() {
        println!("Skipping comparison test - no existing database found");
        return;
    }

    // Test 1: Direct service call
    use hail_mary::memory::{
        models::{MemoryType, RecallParams},
        repository::SqliteMemoryRepository,
        service::MemoryService,
    };

    let repository = SqliteMemoryRepository::new(&db_path).unwrap();
    let mut service = MemoryService::new(repository);

    let direct_params = RecallParams {
        query: "MCP".to_string(),
        memory_type: Some(MemoryType::Tech),
        tags: None,
        limit: Some(10),
        invalid_type: false,
    };

    let direct_result = service.recall(direct_params).await;
    assert!(direct_result.is_ok(), "Direct service call should succeed");
    let direct_response = direct_result.unwrap();

    println!("=== Direct vs MCP Comparison ===");
    println!(
        "Direct service call: {} memories",
        direct_response.memories.len()
    );

    // Test 2: MCP server call
    let server = MemoryMcpServer::new(&db_path).unwrap();

    let mcp_params = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(10),
    };

    let mcp_result = server.recall(Parameters(mcp_params)).await;
    assert!(mcp_result.is_ok(), "MCP server call should succeed");
    let mcp_response = mcp_result.unwrap();

    println!(
        "MCP server call: {} memories",
        mcp_response.0.memories.len()
    );

    // Compare results
    assert_eq!(
        direct_response.memories.len(),
        mcp_response.0.memories.len(),
        "Direct and MCP calls should return same number of memories"
    );

    println!("✅ Direct and MCP results match!");

    // Print both results for comparison
    println!("\nDirect result memories:");
    for memory in &direct_response.memories {
        println!("  - {} ({})", memory.title, memory.memory_type);
    }

    println!("\nMCP result memories:");
    for memory in &mcp_response.0.memories {
        println!("  - {} ({})", memory.title, memory.memory_type);
    }
}

#[tokio::test]
async fn test_mcp_recall_different_params() {
    let db_path = dirs::data_local_dir()
        .unwrap()
        .join("hail-mary")
        .join("memory.db");

    if !db_path.exists() {
        println!("Skipping test - no existing database found");
        return;
    }

    let server = MemoryMcpServer::new(&db_path).unwrap();

    println!("=== Testing Different MCP Recall Parameters ===");

    // Test 1: Query without type filter
    let params_no_type = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: None,
        tags: None,
        limit: Some(10),
    };

    let result_no_type = server.recall(Parameters(params_no_type)).await;
    assert!(result_no_type.is_ok(), "No-type recall should succeed");
    let response_no_type = result_no_type.unwrap();
    println!(
        "No type filter: {} memories",
        response_no_type.0.memories.len()
    );

    // Test 2: Query with tech type filter
    let params_with_tech = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(10),
    };

    let result_with_tech = server.recall(Parameters(params_with_tech)).await;
    assert!(result_with_tech.is_ok(), "Tech-type recall should succeed");
    let response_with_tech = result_with_tech.unwrap();
    println!(
        "With tech type: {} memories",
        response_with_tech.0.memories.len()
    );

    // Test 3: Query with invalid type filter
    let params_invalid_type = RmcpRecallParams {
        query: "MCP".to_string(),
        r#type: Some("invalid".to_string()),
        tags: None,
        limit: Some(10),
    };

    let result_invalid_type = server.recall(Parameters(params_invalid_type)).await;
    assert!(
        result_invalid_type.is_ok(),
        "Invalid-type recall should succeed"
    );
    let response_invalid_type = result_invalid_type.unwrap();
    println!(
        "With invalid type: {} memories",
        response_invalid_type.0.memories.len()
    );

    // Test 4: Empty query with tech type
    let params_empty_query = RmcpRecallParams {
        query: "".to_string(),
        r#type: Some("tech".to_string()),
        tags: None,
        limit: Some(10),
    };

    let result_empty_query = server.recall(Parameters(params_empty_query)).await;
    assert!(
        result_empty_query.is_ok(),
        "Empty query recall should succeed"
    );
    let response_empty_query = result_empty_query.unwrap();
    println!(
        "Empty query with tech type: {} memories",
        response_empty_query.0.memories.len()
    );

    println!("✅ All parameter variations work correctly");
}
