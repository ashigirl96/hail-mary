#[cfg(test)]
mod delete_functionality_tests {
    use hail_mary::memory::{
        models::{Memory, MemoryType},
        repository::{MemoryRepository, SqliteMemoryRepository},
        service::MemoryService,
    };
    use tempfile::tempdir;

    #[test]
    fn test_repository_soft_delete() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut repo = SqliteMemoryRepository::new(&db_path).unwrap();

        // Create and save a memory
        let memory = Memory::new(
            MemoryType::Tech,
            "Test Memory".to_string(),
            "Test content".to_string(),
        );
        let memory_id = memory.id.clone();
        repo.save(&memory).unwrap();

        // Verify it exists
        let found = repo.find_by_id(&memory_id).unwrap();
        assert!(found.is_some());

        // Soft delete the memory
        repo.soft_delete(&memory_id).unwrap();

        // Verify it's no longer found (soft deleted)
        let found = repo.find_by_id(&memory_id).unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_service_delete_memory() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and save a memory using repository
        let mut repository = SqliteMemoryRepository::new(&db_path).unwrap();
        let memory = Memory::new(
            MemoryType::Tech,
            "Test Memory".to_string(),
            "Test content".to_string(),
        );
        let memory_id = memory.id.clone();
        repository.save(&memory).unwrap();

        // Now create service and test
        let repository = SqliteMemoryRepository::new(&db_path).unwrap();
        let mut service = MemoryService::new(repository);

        // Verify it exists via service
        let found = service.get_memory(&memory_id).await.unwrap();
        assert!(found.is_some());

        // Delete via service
        service.delete_memory(&memory_id).await.unwrap();

        // Verify it's deleted
        let found = service.get_memory(&memory_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_mcp_delete_tool() {
        use hail_mary::mcp::server::MemoryMcpServer;
        use hail_mary::memory::models::{RmcpDeleteParams, RmcpRememberParams};
        use rmcp::handler::server::tool::Parameters;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let server = MemoryMcpServer::new(&db_path).unwrap();

        // First, add a memory via MCP
        let remember_params = RmcpRememberParams {
            r#type: "tech".to_string(),
            title: "Test Memory".to_string(),
            content: "Test content".to_string(),
            tags: Some(vec!["test".to_string()]),
            examples: None,
        };
        let response = server.remember(Parameters(remember_params)).await.unwrap();
        let memory_id = response.0.memory_id.clone();

        // Delete the memory via MCP
        let delete_params = RmcpDeleteParams {
            memory_id: memory_id.clone(),
        };
        let delete_response = server
            .delete_memory(Parameters(delete_params))
            .await
            .unwrap();

        assert!(delete_response.0.deleted);
        assert_eq!(delete_response.0.memory_id, memory_id);

        // Verify deletion via recall
        let recall_params = hail_mary::memory::models::RmcpRecallParams {
            query: "Test Memory".to_string(),
            r#type: Some("tech".to_string()),
            tags: None,
            limit: Some(10),
        };
        let recall_response = server.recall(Parameters(recall_params)).await.unwrap();

        // Should not find the deleted memory
        assert_eq!(recall_response.0.memories.len(), 0);
    }

    #[test]
    fn test_cli_delete_command() {
        use std::process::Command;
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // First, create a memory database with test data
        let mut repo = SqliteMemoryRepository::new(&db_path).unwrap();
        let memory = Memory::new(
            MemoryType::Tech,
            "CLI Test Memory".to_string(),
            "Test content for CLI".to_string(),
        );
        let memory_id = memory.id.clone();
        repo.save(&memory).unwrap();

        // Test dry run (without --confirm)
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("memory")
            .arg("delete")
            .arg(&memory_id)
            .arg("--db-path")
            .arg(db_path.to_str().unwrap())
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Dry run mode"));

        // Verify memory still exists
        let found = repo.find_by_id(&memory_id).unwrap();
        assert!(found.is_some());

        // Test actual deletion with --confirm
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("memory")
            .arg("delete")
            .arg(&memory_id)
            .arg("--db-path")
            .arg(db_path.to_str().unwrap())
            .arg("--confirm")
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("has been deleted"));

        // Verify memory is deleted
        let found = repo.find_by_id(&memory_id).unwrap();
        assert!(found.is_none());
    }
}
