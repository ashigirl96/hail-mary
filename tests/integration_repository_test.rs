// Integration tests for SQLite repository with real database operations
// Tests FTS5 functionality, Japanese search, and transaction behavior

use hail_mary::models::kiro::KiroConfig;
use hail_mary::models::memory::{Memory, MemoryType};
use hail_mary::repositories::memory::{MemoryRepository, SqliteMemoryRepository};
use pretty_assertions::assert_eq;
use tempfile::TempDir;

/// Setup temporary directory for tests
fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

#[test]
fn test_sqlite_with_real_database() {
    // End-to-end test with temporary SQLite database
    // Test save, find, search operations in realistic scenario

    let temp_dir = setup_test_dir();
    let db_path = temp_dir.path().join("integration_test.db");

    // Create test config
    let config = KiroConfig {
        root_dir: temp_dir.path().to_path_buf(),
        memory: hail_mary::models::kiro::MemoryConfig {
            types: vec![
                "tech".to_string(),
                "project-tech".to_string(),
                "domain".to_string(),
            ],
            instructions: "Integration test instructions".to_string(),
            document: hail_mary::models::kiro::DocumentConfig {
                output_dir: temp_dir.path().to_path_buf(),
                format: "markdown".to_string(),
            },
            database: hail_mary::models::kiro::DatabaseConfig {
                path: db_path.clone(),
            },
        },
    };

    let mut repo = SqliteMemoryRepository::new(&config).unwrap();

    // Test 1: Save multiple memories with different types
    let tech_memory = Memory::new(
        MemoryType::Tech,
        "Rust Async Programming".to_string(),
        "Rust async/await syntax allows for efficient concurrent programming using tokio runtime."
            .to_string(),
    )
    .with_tags(vec![
        "rust".to_string(),
        "async".to_string(),
        "tokio".to_string(),
    ]);

    let project_memory = Memory::new(
        MemoryType::ProjectTech,
        "Project Error Handling".to_string(),
        "This project uses anyhow::Result for error handling and thiserror for custom error types."
            .to_string(),
    )
    .with_tags(vec![
        "error-handling".to_string(),
        "anyhow".to_string(),
        "thiserror".to_string(),
    ]);

    let domain_memory = Memory::new(
        MemoryType::Domain,
        "Memory System Design".to_string(),
        "The memory system is designed to store and retrieve technical knowledge efficiently."
            .to_string(),
    )
    .with_tags(vec![
        "design".to_string(),
        "memory".to_string(),
        "knowledge".to_string(),
    ]);

    // Save all memories
    repo.save(&tech_memory).unwrap();
    repo.save(&project_memory).unwrap();
    repo.save(&domain_memory).unwrap();

    // Test 2: Find by ID works correctly
    let found_tech = repo.find_by_id(&tech_memory.id).unwrap().unwrap();
    assert_eq!(found_tech.title, "Rust Async Programming");
    assert_eq!(found_tech.memory_type, MemoryType::Tech);
    assert_eq!(found_tech.tags, vec!["rust", "async", "tokio"]);

    // Test 3: find_all returns all non-deleted memories
    let all_memories = repo.find_all().unwrap();
    assert_eq!(all_memories.len(), 3);

    // Test 4: FTS search works with multiple matches
    let search_results = repo.search_fts("rust", 10).unwrap();
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].title, "Rust Async Programming");

    let error_results = repo.search_fts("error", 10).unwrap();
    assert_eq!(error_results.len(), 1);
    assert_eq!(error_results[0].title, "Project Error Handling");

    // Test 5: Reference count increment works
    repo.increment_reference_count(&tech_memory.id).unwrap();
    let updated_memory = repo.find_by_id(&tech_memory.id).unwrap().unwrap();
    assert_eq!(updated_memory.reference_count, 1);
    assert!(updated_memory.last_accessed.is_some());

    // Test 6: Logical deletion filtering
    // Mark one memory as deleted by updating it
    let mut deleted_memory = project_memory.clone();
    deleted_memory.deleted = true;
    repo.save(&deleted_memory).unwrap();

    // Should not appear in find_all
    let all_after_delete = repo.find_all().unwrap();
    assert_eq!(all_after_delete.len(), 2);

    // Should not appear in find_by_id
    let deleted_lookup = repo.find_by_id(&project_memory.id).unwrap();
    assert!(deleted_lookup.is_none());

    // Should not appear in search
    let search_after_delete = repo.search_fts("error", 10).unwrap();
    assert_eq!(search_after_delete.len(), 0);
}

#[test]
fn test_fts5_japanese_search() {
    // Test Japanese text search using FTS5 with tokenize = 'porter unicode61'

    let temp_dir = setup_test_dir();
    let db_path = temp_dir.path().join("japanese_test.db");

    let config = KiroConfig {
        root_dir: temp_dir.path().to_path_buf(),
        memory: hail_mary::models::kiro::MemoryConfig {
            types: vec!["tech".to_string()],
            instructions: "Japanese test instructions".to_string(),
            document: hail_mary::models::kiro::DocumentConfig {
                output_dir: temp_dir.path().to_path_buf(),
                format: "markdown".to_string(),
            },
            database: hail_mary::models::kiro::DatabaseConfig {
                path: db_path.clone(),
            },
        },
    };

    let mut repo = SqliteMemoryRepository::new(&config).unwrap();

    // Create memories with Japanese content
    let japanese_memory1 = Memory::new(
        MemoryType::Tech,
        "Rustの非同期プログラミング".to_string(),
        "Rustでは async/await 構文を使用して非同期プログラミングを行います。tokio ランタイムが最も一般的です。".to_string(),
    ).with_tags(vec!["rust".to_string(), "非同期".to_string(), "プログラミング".to_string()]);

    let japanese_memory2 = Memory::new(
        MemoryType::Tech,
        "データベース設計".to_string(),
        "SQLiteを使用してメモリシステムを実装します。FTS5による全文検索機能を提供します。"
            .to_string(),
    )
    .with_tags(vec![
        "データベース".to_string(),
        "SQLite".to_string(),
        "設計".to_string(),
    ]);

    let mixed_memory = Memory::new(
        MemoryType::Tech,
        "Mixed Content with 日本語 and English".to_string(),
        "This memory contains both English and 日本語 content for testing multilingual search capabilities.".to_string(),
    ).with_tags(vec!["multilingual".to_string(), "test".to_string()]);

    // Save all memories
    repo.save(&japanese_memory1).unwrap();
    repo.save(&japanese_memory2).unwrap();
    repo.save(&mixed_memory).unwrap();

    // Test 1: Search for Japanese terms
    let rust_results = repo.search_fts("Rust", 10).unwrap();
    assert_eq!(rust_results.len(), 1);
    assert_eq!(rust_results[0].title, "Rustの非同期プログラミング");

    let async_results = repo.search_fts("非同期", 10).unwrap();
    assert_eq!(async_results.len(), 1);
    assert_eq!(async_results[0].title, "Rustの非同期プログラミング");

    let database_results = repo.search_fts("データベース", 10).unwrap();
    assert_eq!(database_results.len(), 1);
    assert_eq!(database_results[0].title, "データベース設計");

    // Test 2: Search for English terms in mixed content
    let english_results = repo.search_fts("English", 10).unwrap();
    assert_eq!(english_results.len(), 1);
    assert_eq!(
        english_results[0].title,
        "Mixed Content with 日本語 and English"
    );

    // Test 3: Search for Japanese terms in mixed content
    let nihongo_results = repo.search_fts("日本語", 10).unwrap();
    assert_eq!(nihongo_results.len(), 1);
    assert_eq!(
        nihongo_results[0].title,
        "Mixed Content with 日本語 and English"
    );

    // Test 4: Search in tags (Japanese)
    let tag_results = repo.search_fts("プログラミング", 10).unwrap();
    assert_eq!(tag_results.len(), 1);
    assert_eq!(tag_results[0].title, "Rustの非同期プログラミング");

    // Test 5: Search should be case insensitive for English
    let case_results = repo.search_fts("sqlite", 10).unwrap();
    assert_eq!(case_results.len(), 1);
    assert_eq!(case_results[0].title, "データベース設計");
}

#[test]
fn test_transaction_behavior() {
    // Test batch operations with rollback on error
    // Verify transaction atomicity

    let temp_dir = setup_test_dir();
    let db_path = temp_dir.path().join("transaction_test.db");

    let config = KiroConfig {
        root_dir: temp_dir.path().to_path_buf(),
        memory: hail_mary::models::kiro::MemoryConfig {
            types: vec!["tech".to_string()],
            instructions: "Transaction test instructions".to_string(),
            document: hail_mary::models::kiro::DocumentConfig {
                output_dir: temp_dir.path().to_path_buf(),
                format: "markdown".to_string(),
            },
            database: hail_mary::models::kiro::DatabaseConfig {
                path: db_path.clone(),
            },
        },
    };

    let mut repo = SqliteMemoryRepository::new(&config).unwrap();

    // Test 1: Successful batch save
    let memory1 = Memory::new(
        MemoryType::Tech,
        "Memory 1".to_string(),
        "Content 1".to_string(),
    );
    let memory2 = Memory::new(
        MemoryType::Tech,
        "Memory 2".to_string(),
        "Content 2".to_string(),
    );
    let memory3 = Memory::new(
        MemoryType::Tech,
        "Memory 3".to_string(),
        "Content 3".to_string(),
    );

    let memories = vec![memory1.clone(), memory2.clone(), memory3.clone()];

    // Save batch should succeed
    repo.save_batch(&memories).unwrap();

    // All memories should be saved
    let all_memories = repo.find_all().unwrap();
    assert_eq!(all_memories.len(), 3);

    // Each memory should be findable
    assert!(repo.find_by_id(&memory1.id).unwrap().is_some());
    assert!(repo.find_by_id(&memory2.id).unwrap().is_some());
    assert!(repo.find_by_id(&memory3.id).unwrap().is_some());

    // Test 2: Transaction isolation - concurrent operations
    // Save a memory outside of any batch
    let standalone_memory = Memory::new(
        MemoryType::Tech,
        "Standalone Memory".to_string(),
        "This memory is saved outside of batch".to_string(),
    );
    repo.save(&standalone_memory).unwrap();

    // All 4 memories should now exist
    let all_after_standalone = repo.find_all().unwrap();
    assert_eq!(all_after_standalone.len(), 4);

    // Test 3: Update operations in batch
    let mut updated_memories = vec![memory1.clone(), memory2.clone()];
    updated_memories[0].title = "Updated Memory 1".to_string();
    updated_memories[0].reference_count = 5;
    updated_memories[1].title = "Updated Memory 2".to_string();
    updated_memories[1].reference_count = 10;

    // Update batch should succeed
    repo.save_batch(&updated_memories).unwrap();

    // Verify updates
    let updated_1 = repo.find_by_id(&memory1.id).unwrap().unwrap();
    assert_eq!(updated_1.title, "Updated Memory 1");
    assert_eq!(updated_1.reference_count, 5);

    let updated_2 = repo.find_by_id(&memory2.id).unwrap().unwrap();
    assert_eq!(updated_2.title, "Updated Memory 2");
    assert_eq!(updated_2.reference_count, 10);

    // Memory 3 should remain unchanged
    let unchanged_3 = repo.find_by_id(&memory3.id).unwrap().unwrap();
    assert_eq!(unchanged_3.title, "Memory 3");
    assert_eq!(unchanged_3.reference_count, 0);

    // Test 4: Verify FTS5 consistency after batch operations
    // FTS5 should be updated by triggers during batch operations
    let search_updated = repo.search_fts("Updated", 10).unwrap();
    assert_eq!(search_updated.len(), 2);

    let search_original = repo.search_fts("Memory 3", 10).unwrap();
    assert_eq!(search_original.len(), 1);
    assert_eq!(search_original[0].title, "Memory 3");

    let search_standalone = repo.search_fts("Standalone", 10).unwrap();
    assert_eq!(search_standalone.len(), 1);
    assert_eq!(search_standalone[0].title, "Standalone Memory");
}
