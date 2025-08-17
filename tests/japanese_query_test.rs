use anyhow::Result;
use hail_mary::mcp::server::MemoryMcpServer;
use hail_mary::memory::models::{RmcpRecallParams, RmcpRememberParams};
use rmcp::handler::server::tool::Parameters;
use tempfile::tempdir;

/// Test Japanese queries including "hail-maryを実装している"
#[tokio::test]
async fn test_japanese_queries() -> Result<()> {
    // Create a temporary database
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test_jp.db");
    let server = MemoryMcpServer::new(&db_path)?;

    // Add some memories in Japanese
    let memories = vec![
        (
            "Hail Mary実装",
            "hail-maryを実装している。このプロジェクトはRustで書かれています。",
        ),
        (
            "Hail Mary Documentation",
            "The hail-mary project is a Memory MCP Server implementation.",
        ),
        (
            "Rustプログラミング",
            "Rustは安全性を重視したシステムプログラミング言語です。",
        ),
        (
            "メモリ管理",
            "Rustのメモリ管理は所有権システムによって実現されています。",
        ),
        (
            "非同期処理",
            "Tokioを使った非同期プログラミングの実装方法について。",
        ),
    ];

    // Store memories
    for (title, content) in memories {
        let params = Parameters(RmcpRememberParams {
            r#type: "tech".to_string(),
            title: title.to_string(),
            content: content.to_string(),
            tags: Some(vec!["日本語".to_string(), "rust".to_string()]),
            examples: None,
        });
        server.remember(params).await?;
    }

    // Test 1: Search for "hail-maryを実装している"
    println!("\n=== Test 1: Searching for 'hail-maryを実装している' ===");
    let query1 = Parameters(RmcpRecallParams {
        query: "hail-maryを実装している".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let response1 = server.recall(query1).await?;
    println!("Found {} memories", response1.0.memories.len());

    assert!(
        !response1.0.memories.is_empty(),
        "Should find memory with 'hail-maryを実装している'"
    );
    assert!(
        response1
            .0
            .memories
            .iter()
            .any(|m| m.content.contains("hail-maryを実装している")),
        "Should find the exact Japanese phrase"
    );

    for memory in &response1.0.memories {
        let preview: String = memory.content.chars().take(50).collect();
        println!("  - {}: {}...", memory.title, preview);
    }

    // Test 2: Search for "Rust" (should find both English and Japanese memories)
    println!("\n=== Test 2: Searching for 'Rust' ===");
    let query2 = Parameters(RmcpRecallParams {
        query: "Rust".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let response2 = server.recall(query2).await?;
    println!("Found {} memories", response2.0.memories.len());

    assert!(
        response2.0.memories.len() >= 2,
        "Should find multiple memories mentioning Rust"
    );

    for memory in &response2.0.memories {
        let preview: String = memory.content.chars().take(50).collect();
        println!("  - {}: {}...", memory.title, preview);
    }

    // Test 3: Search for "メモリ管理"
    println!("\n=== Test 3: Searching for 'メモリ管理' ===");
    let query3 = Parameters(RmcpRecallParams {
        query: "メモリ管理".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let response3 = server.recall(query3).await?;
    println!("Found {} memories", response3.0.memories.len());

    assert!(
        !response3.0.memories.is_empty(),
        "Should find memory about memory management"
    );

    for memory in &response3.0.memories {
        let preview: String = memory.content.chars().take(50).collect();
        println!("  - {}: {}...", memory.title, preview);
    }

    // Test 4: Search with tag filter
    println!("\n=== Test 4: Browse with tag '日本語' ===");
    let query4 = Parameters(RmcpRecallParams {
        query: "".to_string(),
        r#type: None,
        tags: Some(vec!["日本語".to_string()]),
        limit: None,
    });

    let response4 = server.recall(query4).await?;
    println!(
        "Found {} memories with tag '日本語'",
        response4.0.memories.len()
    );

    assert_eq!(
        response4.0.memories.len(),
        5,
        "Should find all 5 memories with Japanese tag"
    );

    // Test 5: Search for "hail-mary" (with hyphen)
    println!("\n=== Test 5: Searching for 'hail-mary' (with hyphen) ===");
    let query5 = Parameters(RmcpRecallParams {
        query: "hail-mary".to_string(),
        r#type: None,
        tags: None,
        limit: None,
    });

    let response5 = server.recall(query5).await?;
    println!("Found {} memories", response5.0.memories.len());

    assert!(
        !response5.0.memories.is_empty(),
        "Should find memory with 'hail-mary'"
    );

    for memory in &response5.0.memories {
        let preview: String = memory.content.chars().take(50).collect();
        println!("  - {}: {}...", memory.title, preview);
    }

    println!("\n✅ All Japanese query tests passed!");

    Ok(())
}
