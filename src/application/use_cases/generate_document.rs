use crate::application::errors::ApplicationError;
use crate::application::repositories::{MemoryRepository, ProjectRepository};
use std::path::PathBuf;

pub fn generate_document(
    memory_repo: &mut impl MemoryRepository,
    project_repo: &impl ProjectRepository,
    memory_type: Option<&str>,
) -> Result<PathBuf, ApplicationError> {
    // Load project configuration
    let config = project_repo.load_config()?;

    if let Some(mt) = memory_type {
        // Validate memory type
        if !config.validate_memory_type(mt) {
            return Err(ApplicationError::InvalidMemoryType(mt.to_string()));
        }

        // Generate document for specific type
        let memories = memory_repo.find_by_type(mt)?;
        project_repo.save_document(mt, &memories)?;
    } else {
        // Generate documents for all types
        for memory_type in &config.memory_types {
            let memories = memory_repo.find_by_type(memory_type)?;
            project_repo.save_document(memory_type, &memories)?;
        }
    }

    // Return output directory
    Ok(PathBuf::from(".kiro/memory"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::errors::ApplicationError;
    use crate::application::repositories::{MemoryRepository, ProjectRepository};
    use crate::domain::entities::memory::Memory;
    use crate::domain::entities::project::ProjectConfig;
    use crate::domain::value_objects::confidence::Confidence;
    use std::collections::HashMap;
    use uuid::Uuid;

    // Mock Memory Repository for testing
    struct MockMemoryRepository {
        memories: HashMap<String, Vec<Memory>>,
    }

    impl MockMemoryRepository {
        fn new() -> Self {
            Self {
                memories: HashMap::new(),
            }
        }

        fn add_memory_by_type(&mut self, memory_type: &str, memory: Memory) {
            self.memories
                .entry(memory_type.to_string())
                .or_insert_with(Vec::new)
                .push(memory);
        }
    }

    impl MemoryRepository for MockMemoryRepository {
        fn save(&mut self, _memory: &Memory) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn save_batch(&mut self, _memories: &[Memory]) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn find_by_id(&mut self, _id: &Uuid) -> Result<Option<Memory>, ApplicationError> {
            Ok(None)
        }

        fn search_fts(
            &mut self,
            _query: &str,
            _limit: usize,
        ) -> Result<Vec<Memory>, ApplicationError> {
            Ok(Vec::new())
        }

        fn find_all(&mut self) -> Result<Vec<Memory>, ApplicationError> {
            Ok(Vec::new())
        }

        fn find_by_type(&mut self, memory_type: &str) -> Result<Vec<Memory>, ApplicationError> {
            Ok(self.memories.get(memory_type).cloned().unwrap_or_default())
        }

        fn increment_reference_count(&mut self, _id: &Uuid) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn cleanup_deleted(&mut self) -> Result<usize, ApplicationError> {
            Ok(0)
        }

        fn rebuild_fts_index(&mut self) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn vacuum(&mut self) -> Result<(), ApplicationError> {
            Ok(())
        }
    }

    // Mock Project Repository for testing
    struct MockProjectRepository {
        config: Option<ProjectConfig>,
    }

    impl MockProjectRepository {
        fn new() -> Self {
            Self { config: None }
        }

        fn with_config(mut self, config: ProjectConfig) -> Self {
            self.config = Some(config);
            self
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn initialize(&self) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn exists(&self) -> Result<bool, ApplicationError> {
            Ok(true)
        }

        fn save_config(&self, _config: &ProjectConfig) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn load_config(&self) -> Result<ProjectConfig, ApplicationError> {
            self.config.clone().ok_or(ApplicationError::ProjectNotFound)
        }

        fn update_gitignore(&self) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn create_feature(&self, _name: &str) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn save_document(
            &self,
            _memory_type: &str,
            _memories: &[Memory],
        ) -> Result<(), ApplicationError> {
            // Note: This trait method should take &mut self, but the trait definition uses &self
            // For now, we'll just return Ok(())
            Ok(())
        }
    }

    fn create_test_memory(memory_type: &str, title: &str, content: &str) -> Memory {
        Memory::new(
            memory_type.to_string(),
            title.to_string(),
            content.to_string(),
        )
        .with_confidence(Confidence::new(0.9).unwrap())
    }

    #[test]
    fn test_generate_document_specific_type() {
        let mut memory_repo = MockMemoryRepository::new();
        let project_repo =
            MockProjectRepository::new().with_config(ProjectConfig::default_for_new_project());

        // Add test memories
        memory_repo.add_memory_by_type(
            "tech",
            create_test_memory("tech", "Rust Ownership", "Rust ownership concept"),
        );
        memory_repo.add_memory_by_type(
            "tech",
            create_test_memory("tech", "Pattern Matching", "Match expressions"),
        );

        let result = generate_document(&mut memory_repo, &project_repo, Some("tech"));

        assert!(result.is_ok());
        let output_path = result.unwrap();
        assert_eq!(output_path, PathBuf::from(".kiro/memory"));
    }

    #[test]
    fn test_generate_document_all_types() {
        let mut memory_repo = MockMemoryRepository::new();
        let project_repo =
            MockProjectRepository::new().with_config(ProjectConfig::default_for_new_project());

        // Add memories of different types
        memory_repo.add_memory_by_type("tech", create_test_memory("tech", "Rust", "Rust content"));
        memory_repo.add_memory_by_type(
            "project-tech",
            create_test_memory("project-tech", "Project", "Project content"),
        );
        memory_repo.add_memory_by_type(
            "domain",
            create_test_memory("domain", "Business", "Business content"),
        );

        let result = generate_document(&mut memory_repo, &project_repo, None);

        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_document_invalid_memory_type() {
        let mut memory_repo = MockMemoryRepository::new();
        let project_repo =
            MockProjectRepository::new().with_config(ProjectConfig::default_for_new_project());

        let result = generate_document(&mut memory_repo, &project_repo, Some("invalid_type"));

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::InvalidMemoryType(_)
        ));
    }

    #[test]
    fn test_generate_document_no_config() {
        let mut memory_repo = MockMemoryRepository::new();
        let project_repo = MockProjectRepository::new();

        let result = generate_document(&mut memory_repo, &project_repo, Some("tech"));

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::ProjectNotFound
        ));
    }

    #[test]
    fn test_generate_document_empty_memories() {
        let mut memory_repo = MockMemoryRepository::new();
        let project_repo =
            MockProjectRepository::new().with_config(ProjectConfig::default_for_new_project());

        let result = generate_document(&mut memory_repo, &project_repo, Some("tech"));

        assert!(result.is_ok());
        let output_path = result.unwrap();
        assert_eq!(output_path, PathBuf::from(".kiro/memory"));
    }
}
