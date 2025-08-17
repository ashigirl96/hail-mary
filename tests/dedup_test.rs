// #[cfg(test)]
// mod dedup_tests {
//     use hail_mary::memory::{
//         embeddings::EmbeddingService,
//         models::{Memory, MemoryType},
//         repository::{MemoryRepository, SqliteMemoryRepository},
//     };
//     use tempfile::TempDir;
//
//     #[tokio::test]
//     async fn test_find_duplicates_with_embeddings() {
//         let temp_dir = TempDir::new().unwrap();
//         let db_path = temp_dir.path().join("test.db");
//
//         // Create repository and add similar memories
//         let mut repo = SqliteMemoryRepository::new(&db_path).unwrap();
//
//         // Add nearly identical memories about Rust async
//         let mut memory1 = Memory::new(
//             MemoryType::Tech,
//             "Rust async programming".to_string(),
//             "Rust uses async/await for asynchronous programming with tokio runtime".to_string(),
//         );
//         memory1.tags = vec!["rust".to_string(), "async".to_string()];
//         memory1.confidence = 0.8;
//         repo.save(&memory1).unwrap();
//
//         let mut memory2 = Memory::new(
//             MemoryType::Tech,
//             "Rust asynchronous programming".to_string(),
//             "Rust provides async/await keywords for async programming using tokio".to_string(),
//         );
//         memory2.tags = vec!["rust".to_string(), "async".to_string(), "tokio".to_string()];
//         memory2.confidence = 0.9;
//         repo.save(&memory2).unwrap();
//
//         // Add a different memory that should not be merged
//         let mut memory3 = Memory::new(
//             MemoryType::Tech,
//             "Python decorators".to_string(),
//             "Python decorators are functions that modify other functions".to_string(),
//         );
//         memory3.tags = vec!["python".to_string(), "decorator".to_string()];
//         repo.save(&memory3).unwrap();
//
//         // Generate embeddings and check similarity
//         let embedding_service = EmbeddingService::new().unwrap();
//
//         let text1 = format!("{} {}", memory1.topic, memory1.content);
//         let text2 = format!("{} {}", memory2.topic, memory2.content);
//         let text3 = format!("{} {}", memory3.topic, memory3.content);
//
//         let embeddings = embedding_service
//             .embed_texts(vec![text1, text2, text3])
//             .await
//             .unwrap();
//
//         // Check similarity between Rust memories (should be high)
//         let similarity_rust = EmbeddingService::cosine_similarity(&embeddings[0], &embeddings[1]);
//         assert!(similarity_rust > 0.7, "Rust memories should be similar");
//
//         // Check similarity between Rust and Python (should be low)
//         let similarity_diff = EmbeddingService::cosine_similarity(&embeddings[0], &embeddings[2]);
//         assert!(
//             similarity_diff < 0.5,
//             "Different topics should have low similarity"
//         );
//     }
//
//     #[test]
//     fn test_merge_strategies() {
//         use hail_mary::commands::memory::dedup::{DedupCommand, MergeStrategy};
//
//         let mut newer_memory = Memory::new(
//             MemoryType::Tech,
//             "Test Topic".to_string(),
//             "Newer content".to_string(),
//         );
//         newer_memory.created_at = 2000;
//         newer_memory.confidence = 0.7;
//         newer_memory.reference_count = 3;
//
//         let mut older_memory = Memory::new(
//             MemoryType::Tech,
//             "Test Topic".to_string(),
//             "Older content".to_string(),
//         );
//         older_memory.created_at = 1000;
//         older_memory.confidence = 0.9;
//         older_memory.reference_count = 5;
//         older_memory.tags = vec!["important".to_string()];
//
//         // Test Newer strategy
//         let cmd_newer = DedupCommand {
//             db_path: None,
//             similarity_threshold: 0.8,
//             r#type: None,
//             limit: 100,
//             verbose: false,
//             dry_run: true,
//             interactive: false,
//             strategy: MergeStrategy::Newer,
//             backup: false,
//         };
//
//         let (keep_idx, merged) = cmd_newer.determine_merge(&newer_memory, &older_memory);
//         assert_eq!(keep_idx, 0); // Should keep newer_memory
//         assert!(merged.content.contains("Newer content"));
//         assert!(merged.tags.contains(&"important".to_string())); // Should merge tags
//
//         // Test Confidence strategy
//         let cmd_confidence = DedupCommand {
//             db_path: None,
//             similarity_threshold: 0.8,
//             r#type: None,
//             limit: 100,
//             verbose: false,
//             dry_run: true,
//             interactive: false,
//             strategy: MergeStrategy::Confidence,
//             backup: false,
//         };
//
//         let (keep_idx, merged) = cmd_confidence.determine_merge(&newer_memory, &older_memory);
//         assert_eq!(keep_idx, 1); // Should keep older_memory (higher confidence)
//         assert!(merged.content.contains("Older content"));
//
//         // Test References strategy
//         let cmd_references = DedupCommand {
//             db_path: None,
//             similarity_threshold: 0.8,
//             r#type: None,
//             limit: 100,
//             verbose: false,
//             dry_run: true,
//             interactive: false,
//             strategy: MergeStrategy::References,
//             backup: false,
//         };
//
//         let (keep_idx, merged) = cmd_references.determine_merge(&newer_memory, &older_memory);
//         assert_eq!(keep_idx, 1); // Should keep older_memory (more references)
//         assert_eq!(merged.reference_count, 8); // Should sum references
//     }
//
//     #[test]
//     fn test_smart_merge_strategy() {
//         use hail_mary::commands::memory::dedup::{DedupCommand, MergeStrategy};
//
//         let mut high_quality = Memory::new(
//             MemoryType::Tech,
//             "High Quality Memory".to_string(),
//             "A".repeat(400), // Long content
//         );
//         high_quality.confidence = 0.95;
//         high_quality.reference_count = 10;
//         high_quality.tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
//         high_quality.examples = vec!["ex1".to_string(), "ex2".to_string()];
//         high_quality.created_at = chrono::Utc::now().timestamp() - 60 * 60 * 24 * 5; // 5 days old
//
//         let mut low_quality = Memory::new(
//             MemoryType::Tech,
//             "Low Quality Memory".to_string(),
//             "Short content".to_string(),
//         );
//         low_quality.confidence = 0.6;
//         low_quality.reference_count = 1;
//         low_quality.created_at = chrono::Utc::now().timestamp() - 60 * 60 * 24 * 30; // 30 days old
//
//         let cmd = DedupCommand {
//             db_path: None,
//             similarity_threshold: 0.8,
//             r#type: None,
//             limit: 100,
//             verbose: false,
//             dry_run: true,
//             interactive: false,
//             strategy: MergeStrategy::Smart,
//             backup: false,
//         };
//
//         let score_high = cmd.calculate_memory_score(&high_quality);
//         let score_low = cmd.calculate_memory_score(&low_quality);
//
//         assert!(
//             score_high > score_low,
//             "High quality memory should have higher score"
//         );
//
//         let (keep_idx, _) = cmd.determine_merge(&high_quality, &low_quality);
//         assert_eq!(keep_idx, 0); // Should keep high_quality memory
//     }
// }
