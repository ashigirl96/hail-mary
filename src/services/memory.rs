use crate::models::error::Result;
use crate::models::kiro::KiroConfig;
use crate::models::memory::{Memory, MemoryType};
use crate::repositories::memory::MemoryRepository;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Input structure for creating memories
#[derive(Debug, Clone)]
pub struct MemoryInput {
    pub memory_type: MemoryType,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub confidence: Option<f32>,
}

/// Service layer for memory management with business logic
pub struct MemoryService<R: MemoryRepository> {
    repository: Arc<Mutex<R>>,
    config: KiroConfig,
}

impl<R: MemoryRepository + 'static> MemoryService<R> {
    /// Create new MemoryService with dependency injection
    pub fn new(repository: R, config: KiroConfig) -> Self {
        Self {
            repository: Arc::new(Mutex::new(repository)),
            config,
        }
    }

    /// Remember a batch of memories with validation
    pub async fn remember_batch(&mut self, memories: Vec<MemoryInput>) -> Result<Vec<Memory>> {
        use crate::models::error::MemoryError;

        if memories.is_empty() {
            return Ok(Vec::new());
        }

        let mut created_memories = Vec::new();

        for input in memories {
            // Validate memory type against KiroConfig
            let type_str = input.memory_type.to_string();
            if !self.config.validate_memory_type(&type_str) {
                return Err(MemoryError::InvalidInput(format!(
                    "Invalid memory type '{}'. Allowed types: {:?}",
                    type_str, self.config.memory.types
                )));
            }

            // Validate required fields
            if input.title.trim().is_empty() {
                return Err(MemoryError::InvalidInput(
                    "Title cannot be empty".to_string(),
                ));
            }

            if input.content.trim().is_empty() {
                return Err(MemoryError::InvalidInput(
                    "Content cannot be empty".to_string(),
                ));
            }

            // Validate confidence range if provided
            if let Some(confidence) = input.confidence
                && (!(0.0..=1.0).contains(&confidence))
            {
                return Err(MemoryError::InvalidInput(format!(
                    "Confidence must be between 0.0 and 1.0, got {}",
                    confidence
                )));
            }

            // Create Memory instance from input
            let memory = Memory::new(input.memory_type, input.title, input.content)
                .with_tags(input.tags)
                .with_confidence(input.confidence.unwrap_or(1.0));

            created_memories.push(memory);
        }

        // Save all memories using repository
        let mut repository = self.repository.lock().await;
        repository.save_batch(&created_memories)?;

        Ok(created_memories)
    }

    /// Recall memories with filtering and sorting
    pub async fn recall(
        &mut self,
        query: &str,
        limit: usize,
        type_filter: Option<MemoryType>,
        tag_filter: Vec<String>,
    ) -> Result<String> {
        let repository = self.repository.lock().await;

        // If we have type or tag filters, or empty query, get all memories first then filter
        // Otherwise use FTS search for performance
        let mut memories =
            if type_filter.is_some() || !tag_filter.is_empty() || query.trim().is_empty() {
                // Get all memories then filter manually
                let all_memories = repository.find_all()?;
                if query.trim().is_empty() {
                    // Empty query returns all memories
                    all_memories
                } else {
                    all_memories
                        .into_iter()
                        .filter(|m| {
                            // Simple text matching in title and content
                            m.title.to_lowercase().contains(&query.to_lowercase())
                                || m.content.to_lowercase().contains(&query.to_lowercase())
                        })
                        .collect()
                }
            } else {
                // Use FTS search for efficiency when no additional filters
                repository.search_fts(query, limit * 2)? // Get more for sorting
            };

        drop(repository); // Release lock early

        // Apply business logic filters
        if let Some(memory_type) = type_filter {
            memories.retain(|m| m.memory_type == memory_type);
        }

        if !tag_filter.is_empty() {
            memories.retain(|m| {
                tag_filter
                    .iter()
                    .any(|filter_tag| m.tags.contains(filter_tag))
            });
        }

        // Sort by confidence (desc) then reference_count (desc)
        memories.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap()
                .then(b.reference_count.cmp(&a.reference_count))
        });

        // Apply limit after filtering and sorting
        memories.truncate(limit);

        // Asynchronously update reference counts without blocking
        let memory_ids: Vec<String> = memories.iter().map(|m| m.id.clone()).collect();
        let repository_clone = Arc::clone(&self.repository);
        tokio::spawn(async move {
            let mut repo = repository_clone.lock().await;
            for id in memory_ids {
                let _ = repo.increment_reference_count(&id);
            }
        });

        // Format as Markdown and return
        Ok(self.format_as_markdown(&memories))
    }

    /// Generate documents grouped by memory type
    pub async fn generate_documents(&self, config: &KiroConfig) -> Result<()> {
        use std::collections::HashMap;
        use std::fs;

        let output_dir = config.memory_docs_dir();

        // Ensure output directory exists
        if let Some(parent) = output_dir.parent() {
            fs::create_dir_all(parent).map_err(crate::models::error::MemoryError::Io)?;
        }
        fs::create_dir_all(&output_dir).map_err(crate::models::error::MemoryError::Io)?;

        let repository = self.repository.lock().await;
        let memories = repository.find_all()?;
        drop(repository); // Release lock early

        // Group memories by type
        let mut by_type: HashMap<
            crate::models::memory::MemoryType,
            Vec<crate::models::memory::Memory>,
        > = HashMap::new();

        for memory in memories {
            by_type
                .entry(memory.memory_type.clone())
                .or_default()
                .push(memory);
        }

        // Generate a file for each memory type
        for (memory_type, mut memories) in by_type {
            // Sort by confidence (desc) then reference_count (desc)
            memories.sort_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap()
                    .then(b.reference_count.cmp(&a.reference_count))
            });

            let content = self.format_as_markdown(&memories);
            let filename = format!("{}.md", memory_type);
            let path = output_dir.join(filename);

            fs::write(path, content).map_err(crate::models::error::MemoryError::Io)?;
        }

        Ok(())
    }

    /// Format memories as Markdown
    pub fn format_as_markdown(&self, memories: &[Memory]) -> String {
        let mut output = String::new();

        for memory in memories {
            output.push_str(&format!("## {}\n", memory.title));
            output.push_str(&format!("*Tags: {}*\n", memory.tags.join(", ")));
            output.push_str(&format!(
                "*References: {}, Confidence: {:.2}*\n\n",
                memory.reference_count, memory.confidence
            ));
            output.push_str(&memory.content);
            output.push_str("\n\n");
            output.push_str("---\n\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::memory::MemoryType;
    use crate::repositories::memory::InMemoryRepository;
    use pretty_assertions::assert_eq;

    fn setup_test_service() -> MemoryService<InMemoryRepository> {
        let repository = InMemoryRepository::new();
        let config = KiroConfig::default();
        MemoryService::new(repository, config)
    }

    fn create_test_memory_input() -> MemoryInput {
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "Test Memory".to_string(),
            content: "Test content for memory".to_string(),
            tags: vec!["test".to_string(), "rust".to_string()],
            confidence: Some(0.9),
        }
    }

    #[tokio::test]
    async fn test_memory_service_remember_single() {
        // Test saving a single memory with correct ID generation and defaults
        let mut service = setup_test_service();
        let input = create_test_memory_input();

        let result = service.remember_batch(vec![input.clone()]).await.unwrap();

        // Should return exactly one memory
        assert_eq!(result.len(), 1);

        let memory = &result[0];
        // Check that UUID was generated
        assert!(!memory.id.is_empty());
        assert!(uuid::Uuid::parse_str(&memory.id).is_ok());

        // Check core fields match input
        assert_eq!(memory.memory_type, input.memory_type);
        assert_eq!(memory.title, input.title);
        assert_eq!(memory.content, input.content);
        assert_eq!(memory.tags, input.tags);
        assert_eq!(memory.confidence, input.confidence.unwrap());

        // Check default values
        assert_eq!(memory.reference_count, 0);
        assert_eq!(memory.deleted, false);
        assert!(memory.last_accessed.is_none());

        // created_at should be recent
        let now = chrono::Utc::now().timestamp();
        assert!(memory.created_at <= now);
        assert!(memory.created_at >= now - 5); // Allow 5 seconds
    }

    #[tokio::test]
    async fn test_memory_service_remember_batch() {
        // Test saving multiple memories atomically
        let mut service = setup_test_service();

        let inputs = vec![
            MemoryInput {
                memory_type: MemoryType::Tech,
                title: "Memory 1".to_string(),
                content: "Content 1".to_string(),
                tags: vec!["test".to_string()],
                confidence: Some(0.9),
            },
            MemoryInput {
                memory_type: MemoryType::ProjectTech,
                title: "Memory 2".to_string(),
                content: "Content 2".to_string(),
                tags: vec!["project".to_string()],
                confidence: Some(0.8),
            },
            MemoryInput {
                memory_type: MemoryType::Domain,
                title: "Memory 3".to_string(),
                content: "Content 3".to_string(),
                tags: vec!["domain".to_string()],
                confidence: None, // Test default confidence
            },
        ];

        let result = service.remember_batch(inputs.clone()).await.unwrap();

        // Should return all memories
        assert_eq!(result.len(), 3);

        // Check all memories have unique IDs
        let ids: std::collections::HashSet<String> = result.iter().map(|m| m.id.clone()).collect();
        assert_eq!(ids.len(), 3, "All memories should have unique IDs");

        // Check each memory matches input
        for (i, memory) in result.iter().enumerate() {
            let input = &inputs[i];
            assert_eq!(memory.memory_type, input.memory_type);
            assert_eq!(memory.title, input.title);
            assert_eq!(memory.content, input.content);
            assert_eq!(memory.tags, input.tags);

            // Check confidence handling
            if let Some(conf) = input.confidence {
                assert_eq!(memory.confidence, conf);
            } else {
                assert_eq!(memory.confidence, 1.0, "Default confidence should be 1.0");
            }
        }
    }

    #[tokio::test]
    async fn test_memory_service_validates_memory_type() {
        // Test that KiroConfig memory type validation is enforced
        let repository = InMemoryRepository::new();
        let mut config = KiroConfig::default();
        // Limit allowed types to only "tech"
        config.memory.types = vec!["tech".to_string()];

        let mut service = MemoryService::new(repository, config);

        // Valid type should succeed
        let valid_input = MemoryInput {
            memory_type: MemoryType::Tech,
            title: "Valid Memory".to_string(),
            content: "Valid content".to_string(),
            tags: vec![],
            confidence: None,
        };

        let result = service.remember_batch(vec![valid_input]).await;
        assert!(result.is_ok(), "Valid memory type should be accepted");

        // Invalid type should fail
        let invalid_input = MemoryInput {
            memory_type: MemoryType::ProjectTech, // Not in allowed types
            title: "Invalid Memory".to_string(),
            content: "Invalid content".to_string(),
            tags: vec![],
            confidence: None,
        };

        let result = service.remember_batch(vec![invalid_input]).await;
        assert!(result.is_err(), "Invalid memory type should be rejected");

        // Error message should mention the invalid type
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("project-tech") || error_msg.contains("Invalid"),
            "Error should reference the invalid type: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_memory_service_validates_empty_fields() {
        // Test validation of required fields
        let mut service = setup_test_service();

        // Empty title should fail
        let empty_title = MemoryInput {
            memory_type: MemoryType::Tech,
            title: "".to_string(),
            content: "Valid content".to_string(),
            tags: vec![],
            confidence: None,
        };

        let result = service.remember_batch(vec![empty_title]).await;
        assert!(result.is_err(), "Empty title should be rejected");

        // Empty content should fail
        let empty_content = MemoryInput {
            memory_type: MemoryType::Tech,
            title: "Valid title".to_string(),
            content: "".to_string(),
            tags: vec![],
            confidence: None,
        };

        let result = service.remember_batch(vec![empty_content]).await;
        assert!(result.is_err(), "Empty content should be rejected");
    }

    #[tokio::test]
    async fn test_memory_service_confidence_validation() {
        // Test confidence range validation (0.0-1.0)
        let mut service = setup_test_service();

        // Valid confidence values
        let valid_confidences = vec![0.0, 0.5, 1.0];
        for confidence in valid_confidences {
            let input = MemoryInput {
                memory_type: MemoryType::Tech,
                title: format!("Test {}", confidence),
                content: "Valid content".to_string(),
                tags: vec![],
                confidence: Some(confidence),
            };

            let result = service.remember_batch(vec![input]).await;
            assert!(result.is_ok(), "Confidence {} should be valid", confidence);
        }

        // Invalid confidence values
        let invalid_confidences = vec![-0.1, 1.1, 2.0];
        for confidence in invalid_confidences {
            let input = MemoryInput {
                memory_type: MemoryType::Tech,
                title: format!("Invalid {}", confidence),
                content: "Valid content".to_string(),
                tags: vec![],
                confidence: Some(confidence),
            };

            let result = service.remember_batch(vec![input]).await;
            assert!(
                result.is_err(),
                "Confidence {} should be invalid",
                confidence
            );
        }
    }

    // Recall functionality tests

    async fn setup_service_with_memories() -> MemoryService<InMemoryRepository> {
        let mut service = setup_test_service();

        // Add test memories with different types, tags, and confidence levels
        let memories = vec![
            MemoryInput {
                memory_type: MemoryType::Tech,
                title: "Rust Programming".to_string(),
                content: "Rust is a systems programming language with memory safety".to_string(),
                tags: vec!["rust".to_string(), "programming".to_string()],
                confidence: Some(0.9),
            },
            MemoryInput {
                memory_type: MemoryType::Tech,
                title: "JavaScript Basics".to_string(),
                content: "JavaScript is a dynamic programming language for web development"
                    .to_string(),
                tags: vec!["javascript".to_string(), "web".to_string()],
                confidence: Some(0.8),
            },
            MemoryInput {
                memory_type: MemoryType::ProjectTech,
                title: "Project Setup".to_string(),
                content: "How to set up this specific project with dependencies".to_string(),
                tags: vec!["setup".to_string(), "dependencies".to_string()],
                confidence: Some(0.95),
            },
            MemoryInput {
                memory_type: MemoryType::Domain,
                title: "Business Logic".to_string(),
                content: "Domain-driven design principles for business logic".to_string(),
                tags: vec!["domain".to_string(), "business".to_string()],
                confidence: Some(0.7),
            },
        ];

        service.remember_batch(memories).await.unwrap();
        service
    }

    #[tokio::test]
    async fn test_recall_with_type_filter() {
        // Test filtering memories by type
        let mut service = setup_service_with_memories().await;

        // Filter by Tech type should return 2 memories
        let tech_result = service
            .recall("programming", 10, Some(MemoryType::Tech), vec![])
            .await
            .unwrap();

        // Should contain both Rust and JavaScript memories
        assert!(tech_result.contains("Rust Programming"));
        assert!(tech_result.contains("JavaScript Basics"));
        assert!(!tech_result.contains("Project Setup")); // ProjectTech
        assert!(!tech_result.contains("Business Logic")); // Domain

        // Filter by ProjectTech should return 1 memory
        let project_result = service
            .recall("setup", 10, Some(MemoryType::ProjectTech), vec![])
            .await
            .unwrap();

        assert!(project_result.contains("Project Setup"));
        assert!(!project_result.contains("Rust Programming"));

        // Filter by Domain should return 1 memory
        let domain_result = service
            .recall("business", 10, Some(MemoryType::Domain), vec![])
            .await
            .unwrap();

        assert!(domain_result.contains("Business Logic"));
        assert!(!domain_result.contains("Rust Programming"));
    }

    #[tokio::test]
    async fn test_recall_with_tag_filter() {
        // Test filtering memories by tags (OR logic)
        let mut service = setup_service_with_memories().await;

        // Filter by "rust" tag should return Rust memory
        let rust_result = service
            .recall("programming", 10, None, vec!["rust".to_string()])
            .await
            .unwrap();

        assert!(rust_result.contains("Rust Programming"));
        assert!(!rust_result.contains("JavaScript Basics"));

        // Filter by "web" tag should return JavaScript memory
        let web_result = service
            .recall("programming", 10, None, vec!["web".to_string()])
            .await
            .unwrap();

        assert!(web_result.contains("JavaScript Basics"));
        assert!(!web_result.contains("Rust Programming"));

        // Filter by multiple tags (OR logic) should return multiple memories
        let multi_result = service
            .recall(
                "programming",
                10,
                None,
                vec!["rust".to_string(), "web".to_string()],
            )
            .await
            .unwrap();

        assert!(multi_result.contains("Rust Programming"));
        assert!(multi_result.contains("JavaScript Basics"));

        // Filter by non-existent tag should return no memories
        let empty_result = service
            .recall("programming", 10, None, vec!["nonexistent".to_string()])
            .await
            .unwrap();

        assert!(!empty_result.contains("Rust Programming"));
        assert!(!empty_result.contains("JavaScript Basics"));
    }

    #[tokio::test]
    async fn test_recall_sorts_by_confidence() {
        // Test that memories are sorted by confidence (desc) then reference_count (desc)
        let mut service = setup_service_with_memories().await;

        // Search for all memories using an empty query to get all results
        let result = service.recall("", 10, None, vec![]).await.unwrap();

        // Extract memory order from markdown output
        let lines: Vec<&str> = result.lines().collect();
        let mut memory_order = Vec::new();

        for line in lines {
            if line.starts_with("## ") {
                let title = line.trim_start_matches("## ");
                memory_order.push(title);
            }
        }

        // Should be sorted by confidence: Project Setup (0.95) > Rust (0.9) > JavaScript (0.8) > Business Logic (0.7)
        assert!(memory_order.len() >= 3);

        // First should be Project Setup (highest confidence 0.95)
        assert_eq!(memory_order[0], "Project Setup");
    }

    #[tokio::test]
    async fn test_recall_returns_markdown() {
        // Test that recall returns properly formatted Markdown
        let mut service = setup_service_with_memories().await;

        let result = service.recall("Rust", 1, None, vec![]).await.unwrap();

        // Should be valid Markdown with expected sections
        assert!(result.contains("## Rust Programming"));
        assert!(result.contains("*Tags: rust, programming*"));
        assert!(result.contains("*References: 0, Confidence: 0.90*"));
        assert!(result.contains("Rust is a systems programming language"));
        assert!(result.contains("---")); // Separator
    }

    #[tokio::test]
    async fn test_recall_updates_reference_count_async() {
        // Test that reference counts are updated asynchronously without blocking
        let mut service = setup_service_with_memories().await;

        // First recall should not block and should return immediately
        let start_time = std::time::Instant::now();
        let _result = service.recall("Rust", 1, None, vec![]).await.unwrap();
        let elapsed = start_time.elapsed();

        // Should return quickly (not blocked by reference count updates)
        assert!(elapsed < std::time::Duration::from_millis(100));

        // Give some time for async updates to complete
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Verify that reference count was actually updated
        // We'll need to check this through the repository
        let repository = service.repository.lock().await;
        let all_memories = repository.find_all().unwrap();

        // Find the Rust memory and check its reference count
        let rust_memory = all_memories
            .iter()
            .find(|m| m.title == "Rust Programming")
            .unwrap();

        assert_eq!(rust_memory.reference_count, 1);
        assert!(rust_memory.last_accessed.is_some());
    }

    #[tokio::test]
    async fn test_recall_with_limit() {
        // Test that recall respects the limit parameter
        let mut service = setup_service_with_memories().await;

        // Limit to 2 results
        let result = service
            .recall("programming", 2, None, vec![])
            .await
            .unwrap();

        // Count number of memory sections (## headers)
        let memory_count = result.matches("## ").count();
        assert_eq!(memory_count, 2, "Should return exactly 2 memories");

        // Limit to 1 result
        let result_one = service
            .recall("programming", 1, None, vec![])
            .await
            .unwrap();

        let memory_count_one = result_one.matches("## ").count();
        assert_eq!(memory_count_one, 1, "Should return exactly 1 memory");
    }

    // Document generation tests

    #[tokio::test]
    async fn test_generate_documents_creates_files() {
        // Test that generate_documents creates type-specific files in correct paths
        use std::fs;
        use tempfile::TempDir;

        let service = setup_service_with_memories().await;

        // Create temporary directory for output
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("memory");
        fs::create_dir_all(&output_path).unwrap();

        // Create custom config with test output directory
        let mut config = KiroConfig::default();
        config.memory.document.output_dir = output_path.clone();

        // Generate documents
        let result = service.generate_documents(&config).await;
        assert!(result.is_ok(), "Document generation should succeed");

        // Check that files were created for each memory type
        let tech_file = output_path.join("tech.md");
        let project_tech_file = output_path.join("project-tech.md");
        let domain_file = output_path.join("domain.md");

        assert!(tech_file.exists(), "tech.md should be created");
        assert!(
            project_tech_file.exists(),
            "project-tech.md should be created"
        );
        assert!(domain_file.exists(), "domain.md should be created");

        // Verify content of tech.md contains expected memories
        let tech_content = fs::read_to_string(&tech_file).unwrap();
        assert!(
            tech_content.contains("Rust Programming"),
            "Should contain Rust memory"
        );
        assert!(
            tech_content.contains("JavaScript Basics"),
            "Should contain JavaScript memory"
        );
        assert!(
            !tech_content.contains("Project Setup"),
            "Should not contain ProjectTech memory"
        );

        // Verify content of project-tech.md
        let project_content = fs::read_to_string(&project_tech_file).unwrap();
        assert!(
            project_content.contains("Project Setup"),
            "Should contain Project Setup memory"
        );
        assert!(
            !project_content.contains("Rust Programming"),
            "Should not contain Tech memory"
        );

        // Verify content of domain.md
        let domain_content = fs::read_to_string(&domain_file).unwrap();
        assert!(
            domain_content.contains("Business Logic"),
            "Should contain Business Logic memory"
        );
        assert!(
            !domain_content.contains("Rust Programming"),
            "Should not contain Tech memory"
        );

        // Verify memories are sorted by confidence within each file
        // Tech file should have Rust (0.9) before JavaScript (0.8)
        let rust_pos = tech_content.find("Rust Programming").unwrap();
        let js_pos = tech_content.find("JavaScript Basics").unwrap();
        assert!(
            rust_pos < js_pos,
            "Rust (0.9 confidence) should appear before JavaScript (0.8)"
        );
    }
}
