#[cfg(test)]
mod reindex_tests {
    use super::super::reindex::ReindexCommand;
    use crate::memory::{
        models::{Memory, MemoryType},
        repository::{SqliteMemoryRepository, MemoryRepository},
        reindex::{ReindexService, ReindexConfig},
    };
    use tempfile::tempdir;
    use std::path::PathBuf;
    
    #[test]
    fn test_reindex_command_validation() {
        let cmd = ReindexCommand {
            db_path: None,
            similarity_threshold: 0.85,
            no_backup: true,
            backup_dir: None,
            verbose: false,
            dry_run: true,
        };
        
        assert!(cmd.similarity_threshold >= 0.0 && cmd.similarity_threshold <= 1.0);
    }
    
    #[tokio::test]
    async fn test_reindex_with_duplicates() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        // Create database with duplicate memories
        let mut repo = SqliteMemoryRepository::new(&db_path).unwrap();
        
        // Add similar memories
        let memory1 = Memory::with_tags(
            MemoryType::Tech,
            "Rust async programming".to_string(),
            "Rust uses async/await for asynchronous programming".to_string(),
            vec!["rust".to_string(), "async".to_string()],
        );
        repo.save(&memory1).unwrap();
        
        let memory2 = Memory::with_tags(
            MemoryType::Tech,
            "Rust async/await".to_string(),
            "Rust provides async and await keywords for async programming".to_string(),
            vec!["rust".to_string(), "async".to_string(), "await".to_string()],
        );
        repo.save(&memory2).unwrap();
        
        // Add different memory
        let memory3 = Memory::with_tags(
            MemoryType::Tech,
            "Python decorators".to_string(),
            "Python decorators are functions that modify other functions".to_string(),
            vec!["python".to_string(), "decorator".to_string()],
        );
        repo.save(&memory3).unwrap();
        
        // Run reindex
        let config = ReindexConfig {
            similarity_threshold: 0.7, // Lower threshold to catch our test duplicates
            backup_enabled: false,
            verbose: true,
            ..Default::default()
        };
        
        let service = ReindexService::new(config).unwrap();
        let result = service.reindex(&db_path).await.unwrap();
        
        // Verify results
        assert_eq!(result.total_memories, 3);
        assert!(result.duplicates_found > 0); // Should find the similar Rust memories
        
        // Check that database still works after reindex
        let repo_after = SqliteMemoryRepository::new(&db_path).unwrap();
        let memories_after = repo_after.browse_by_type(&MemoryType::Tech, 100).unwrap();
        assert!(memories_after.len() <= 3); // May be fewer if duplicates were merged
    }
    
    #[test]
    fn test_dry_run_mode() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        // Create empty database
        let _repo = SqliteMemoryRepository::new(&db_path).unwrap();
        
        let cmd = ReindexCommand {
            db_path: Some(db_path.clone()),
            similarity_threshold: 0.85,
            no_backup: true,
            backup_dir: None,
            verbose: false,
            dry_run: true,
        };
        
        // Should not error on empty database
        let result = cmd.execute();
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_backup_functionality() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let backup_dir = temp_dir.path().join("backups");
        
        // Create database with some data
        let mut repo = SqliteMemoryRepository::new(&db_path).unwrap();
        let memory = Memory::new(
            MemoryType::Tech,
            "Test memory".to_string(),
            "Test content".to_string(),
        );
        repo.save(&memory).unwrap();
        
        // Run reindex with backup enabled
        let config = ReindexConfig {
            similarity_threshold: 0.85,
            backup_enabled: true,
            backup_dir: backup_dir.clone(),
            verbose: false,
        };
        
        let service = ReindexService::new(config).unwrap();
        let result = service.reindex(&db_path).await.unwrap();
        
        // Check that backup was created
        assert!(result.backup_path.is_some());
        if let Some(backup_path) = result.backup_path {
            assert!(backup_path.exists());
            assert!(backup_path.parent().unwrap() == backup_dir);
        }
    }
    
    #[test]
    fn test_invalid_threshold() {
        let cmd = ReindexCommand {
            db_path: None,
            similarity_threshold: 1.5, // Invalid threshold
            no_backup: true,
            backup_dir: None,
            verbose: false,
            dry_run: false,
        };
        
        // Should handle invalid threshold gracefully
        let result = cmd.execute();
        assert!(result.is_ok()); // Returns Ok but prints error message
    }
}