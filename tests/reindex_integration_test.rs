// #[cfg(test)]
// mod reindex_integration_tests {
//     use hail_mary::memory::{
//         models::{Memory, MemoryType},
//         reindex::{ReindexConfig, ReindexService},
//         repository::{MemoryRepository, SqliteMemoryRepository},
//     };
//     use tempfile::tempdir;
//
//     #[tokio::test]
//     async fn test_full_reindex_workflow() {
//         let temp_dir = tempdir().unwrap();
//         let db_path = temp_dir.path().join("test.db");
//
//         // Step 1: Create database with some memories
//         let mut repo = SqliteMemoryRepository::new(&db_path).unwrap();
//
//         // Add similar Rust memories that should be merged
//         let mut rust1 = Memory::new(
//             MemoryType::Tech,
//             "Rust async programming".to_string(),
//             "Rust uses async/await for asynchronous programming".to_string(),
//         );
//         rust1.tags = vec!["rust".to_string(), "async".to_string()];
//         repo.save(&rust1).unwrap();
//
//         let mut rust2 = Memory::new(
//             MemoryType::Tech,
//             "Rust async and await".to_string(),
//             "Rust provides async/await keywords for async code".to_string(),
//         );
//         rust2.tags = vec!["rust".to_string(), "async".to_string()];
//         repo.save(&rust2).unwrap();
//
//         // Add different memories that should not be merged
//         let mut python = Memory::new(
//             MemoryType::Tech,
//             "Python decorators".to_string(),
//             "Python decorators are functions that modify other functions".to_string(),
//         );
//         python.tags = vec!["python".to_string(), "decorator".to_string()];
//         repo.save(&python).unwrap();
//
//         let mut project = Memory::new(
//             MemoryType::ProjectTech,
//             "Project error handling".to_string(),
//             "Use anyhow::Result for all error handling".to_string(),
//         );
//         project.tags = vec!["error".to_string(), "anyhow".to_string()];
//         repo.save(&project).unwrap();
//
//         // Step 2: Run reindex with a lower threshold to catch similar memories
//         let config = ReindexConfig {
//             similarity_threshold: 0.6, // Lower threshold for test
//             backup_enabled: true,
//             backup_dir: temp_dir.path().join("backups"),
//             verbose: false,
//             generate_embeddings: false,
//             embedding_batch_size: 32,
//             force_regenerate_embeddings: false,
//         };
//
//         let service = ReindexService::new(config).unwrap();
//         let result = service.reindex(&db_path).await.unwrap();
//
//         // Step 3: Verify results
//         assert_eq!(result.total_memories, 4, "Should start with 4 memories");
//         assert!(
//             result.duplicates_found > 0,
//             "Should find at least one duplicate pair"
//         );
//         assert!(result.backup_path.is_some(), "Should create a backup");
//
//         // Step 4: Verify database after reindex
//         let repo_after = SqliteMemoryRepository::new(&db_path).unwrap();
//         let tech_memories = repo_after.browse_by_type(&MemoryType::Tech, 100).unwrap();
//         let project_memories = repo_after
//             .browse_by_type(&MemoryType::ProjectTech, 100)
//             .unwrap();
//
//         // Should have fewer tech memories after merging duplicates
//         assert!(
//             tech_memories.len() < 3,
//             "Tech memories should be reduced after merge"
//         );
//         assert_eq!(
//             project_memories.len(),
//             1,
//             "Project memories should remain unchanged"
//         );
//
//         // Step 5: Verify backup was created
//         if let Some(backup_path) = result.backup_path {
//             assert!(backup_path.exists(), "Backup file should exist");
//
//             // Verify backup contains original data
//             let backup_repo = SqliteMemoryRepository::new(&backup_path).unwrap();
//             let backup_memories = backup_repo.browse_by_type(&MemoryType::Tech, 100).unwrap();
//             assert_eq!(
//                 backup_memories.len(),
//                 3,
//                 "Backup should contain original 3 tech memories"
//             );
//         }
//     }
//
//     #[test]
//     fn test_similarity_calculation() {
//         use hail_mary::memory::embeddings::EmbeddingService;
//
//         // Test vectors
//         let identical = vec![1.0, 0.0, 0.0];
//         let similar = vec![0.9, 0.1, 0.0];
//         let orthogonal = vec![0.0, 1.0, 0.0];
//         let opposite = vec![-1.0, 0.0, 0.0];
//
//         // Test identical vectors
//         let sim = EmbeddingService::cosine_similarity(&identical, &identical);
//         assert!(
//             (sim - 1.0).abs() < 0.001,
//             "Identical vectors should have similarity ~1.0"
//         );
//
//         // Test similar vectors
//         let sim = EmbeddingService::cosine_similarity(&identical, &similar);
//         assert!(sim > 0.8, "Similar vectors should have high similarity");
//
//         // Test orthogonal vectors
//         let sim = EmbeddingService::cosine_similarity(&identical, &orthogonal);
//         assert!(
//             (sim - 0.0).abs() < 0.001,
//             "Orthogonal vectors should have similarity ~0.0"
//         );
//
//         // Test opposite vectors
//         let sim = EmbeddingService::cosine_similarity(&identical, &opposite);
//         assert!(
//             (sim - -1.0).abs() < 0.001,
//             "Opposite vectors should have similarity ~-1.0"
//         );
//     }
// }
