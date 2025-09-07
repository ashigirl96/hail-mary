use crate::application::errors::ApplicationError;
use crate::application::repositories::ProjectRepository;
use crate::domain::entities::memory::Memory;
use crate::domain::entities::project::ProjectConfig;
use crate::domain::entities::steering::SteeringConfig;
use std::collections::HashMap;

/// Mock implementation of ProjectRepository for testing
/// Combines features from all existing mock implementations
#[derive(Debug)]
pub struct MockProjectRepository {
    // State management
    initialized: bool,
    config: Option<ProjectConfig>,
    config_exists: bool,
    features: Vec<String>,
    created_features: Vec<String>,
    saved_documents: HashMap<String, usize>, // memory_type -> count
    // Failure simulation
    should_fail_next_operation: bool,
    should_fail_operation: Option<String>,
    // Custom behavior
    feature_exists: bool,
}

impl Default for MockProjectRepository {
    fn default() -> Self {
        Self {
            initialized: false,
            config: None,
            config_exists: true, // Default to true for compatibility with existing tests
            features: Vec::new(),
            created_features: Vec::new(),
            saved_documents: HashMap::new(),
            should_fail_next_operation: false,
            should_fail_operation: None,
            feature_exists: false,
        }
    }
}

impl MockProjectRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_without_config() -> Self {
        MockProjectRepository {
            config_exists: false,
            ..Default::default()
        }
    }

    // State management
    pub fn set_initialized(&mut self, initialized: bool) {
        self.initialized = initialized;
    }

    pub fn with_config(mut self, config: ProjectConfig) -> Self {
        self.config = Some(config);
        self.config_exists = true;
        self
    }

    pub fn set_config(&mut self, config: ProjectConfig) {
        self.config = Some(config);
        self.config_exists = true;
    }

    pub fn set_config_exists(&mut self, exists: bool) {
        self.config_exists = exists;
    }

    pub fn add_feature(&mut self, feature: String) {
        self.features.push(feature);
    }

    pub fn add_created_feature(&mut self, name: &str) {
        self.created_features.push(name.to_string());
    }

    pub fn add_saved_document(&mut self, memory_type: &str, count: usize) {
        self.saved_documents.insert(memory_type.to_string(), count);
    }

    pub fn get_features(&self) -> &[String] {
        &self.features
    }

    pub fn get_created_features(&self) -> &[String] {
        &self.created_features
    }

    pub fn get_saved_documents(&self) -> &HashMap<String, usize> {
        &self.saved_documents
    }

    // Failure simulation
    pub fn set_should_fail_next_operation(&mut self, should_fail: bool) {
        self.should_fail_next_operation = should_fail;
    }

    pub fn set_next_operation_to_fail(&mut self) {
        self.should_fail_next_operation = true;
    }

    pub fn reset_failure_flag(&mut self) {
        self.should_fail_next_operation = false;
    }

    pub fn set_should_fail_operation(&mut self, operation: Option<String>) {
        self.should_fail_operation = operation;
    }

    // Custom behavior
    pub fn set_feature_exists(&mut self, exists: bool) {
        self.feature_exists = exists;
    }

    pub fn set_operation_to_fail(&mut self, operation: &str) {
        self.should_fail_operation = Some(operation.to_string());
    }

    pub fn clear_failure(&mut self) {
        self.should_fail_operation = None;
    }

    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }
}

impl ProjectRepository for MockProjectRepository {
    fn initialize(&self) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::ProjectInitializationError(
                "Simulated initialization failure".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "initialize"
        {
            return Err(ApplicationError::ProjectInitializationError(
                "Mock initialization failure".to_string(),
            ));
        }
        Ok(())
    }

    fn exists(&self) -> Result<bool, ApplicationError> {
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "exists"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock exists failure".to_string(),
            ));
        }
        Ok(self.initialized)
    }

    fn save_config(&self, _config: &ProjectConfig) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::ConfigurationError(
                "Simulated save_config failure".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "save_config"
        {
            return Err(ApplicationError::ConfigurationError(
                "Mock save_config failure".to_string(),
            ));
        }
        Ok(())
    }

    fn load_config(&self) -> Result<ProjectConfig, ApplicationError> {
        if let Some(ref config) = self.config {
            Ok(config.clone())
        } else if self.config_exists {
            // Return default config if config_exists is true but no custom config set
            Ok(ProjectConfig::default_for_new_project())
        } else {
            // Return error when config doesn't exist
            Err(ApplicationError::ProjectNotFound)
        }
    }

    fn update_gitignore(&self) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Simulated update_gitignore failure".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "update_gitignore"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock update_gitignore failure".to_string(),
            ));
        }
        Ok(())
    }

    fn create_feature(&self, name: &str) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FeatureCreationError(format!(
                "Failed to create feature: {}",
                name
            )));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "create_feature"
        {
            return Err(ApplicationError::FeatureCreationError(format!(
                "Mock creation failure for: {}",
                name
            )));
        }

        // Validate feature name (kebab-case)
        if !name
            .chars()
            .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
            || name.starts_with('-')
            || name.ends_with('-')
            || name.contains("--")
        {
            return Err(ApplicationError::InvalidFeatureName(name.to_string()));
        }

        if self.feature_exists
            || self.features.contains(&name.to_string())
            || self.created_features.contains(&name.to_string())
        {
            return Err(ApplicationError::FeatureAlreadyExists(name.to_string()));
        }
        Ok(())
    }

    fn save_document(
        &self,
        memory_type: &str,
        _memories: &[Memory],
    ) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::DocumentGenerationError(format!(
                "Failed to save document for type: {}",
                memory_type
            )));
        }
        Ok(())
    }

    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Failed to list spec directories".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "list_spec_directories"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock list_spec_directories failure".to_string(),
            ));
        }
        // Return created features as specs (simulating specs directory)
        let specs = self
            .created_features
            .iter()
            .map(|f| (f.clone(), false))
            .collect();
        Ok(specs)
    }

    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Failed to mark spec as complete".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "mark_spec_complete"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock mark_spec_complete failure".to_string(),
            ));
        }
        if !self.created_features.contains(&name.to_string()) {
            return Err(ApplicationError::SpecNotFound(name.to_string()));
        }
        Ok(())
    }

    fn get_spec_path(&self, name: &str) -> Result<std::path::PathBuf, ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Failed to get spec path".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "get_spec_path"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock get_spec_path failure".to_string(),
            ));
        }
        if !self.created_features.contains(&name.to_string()) {
            return Err(ApplicationError::SpecNotFound(name.to_string()));
        }

        // Return a mock path
        let mock_path = std::path::PathBuf::from(".kiro/specs").join(name);
        Ok(mock_path)
    }

    fn initialize_steering(&self) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Failed to initialize steering".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "initialize_steering"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock initialize_steering failure".to_string(),
            ));
        }
        Ok(())
    }

    fn create_steering_files(&self, _config: &SteeringConfig) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Failed to create steering files".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "create_steering_files"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock create_steering_files failure".to_string(),
            ));
        }
        Ok(())
    }

    fn ensure_steering_config(&self) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::ConfigurationError(
                "Failed to ensure steering config".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "ensure_steering_config"
        {
            return Err(ApplicationError::ConfigurationError(
                "Mock ensure_steering_config failure".to_string(),
            ));
        }
        Ok(())
    }
}
