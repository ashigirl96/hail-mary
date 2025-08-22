use crate::application::errors::ApplicationError;
use crate::application::repositories::ProjectRepository;
use crate::domain::entities::memory::Memory;
use crate::domain::entities::project::ProjectConfig;

/// Mock implementation of ProjectRepository for testing
/// Combines features from all existing mock implementations
#[derive(Debug, Default)]
pub struct MockProjectRepository {
    // State management
    initialized: bool,
    config_exists: bool,
    features: Vec<String>,
    // Failure simulation
    should_fail_next_operation: bool,
    should_fail_operation: Option<String>,
    // Custom behavior
    feature_exists: bool,
}

impl MockProjectRepository {
    pub fn new() -> Self {
        Self::default()
    }

    // State management
    pub fn set_initialized(&mut self, initialized: bool) {
        self.initialized = initialized;
    }

    pub fn with_config(mut self, _config: ProjectConfig) -> Self {
        self.config_exists = true;
        self
    }

    pub fn set_config_exists(&mut self, exists: bool) {
        self.config_exists = exists;
    }

    pub fn add_feature(&mut self, feature: String) {
        self.features.push(feature);
    }

    pub fn get_features(&self) -> &[String] {
        &self.features
    }

    // Failure simulation
    pub fn set_should_fail_next_operation(&mut self, should_fail: bool) {
        self.should_fail_next_operation = should_fail;
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
        if let Some(ref fail_op) = self.should_fail_operation {
            if fail_op == "initialize" {
                return Err(ApplicationError::ProjectInitializationError(
                    "Mock initialization failure".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn exists(&self) -> Result<bool, ApplicationError> {
        if let Some(ref fail_op) = self.should_fail_operation {
            if fail_op == "exists" {
                return Err(ApplicationError::FileSystemError(
                    "Mock exists failure".to_string(),
                ));
            }
        }
        Ok(self.initialized)
    }

    fn save_config(&self, _config: &ProjectConfig) -> Result<(), ApplicationError> {
        if let Some(ref fail_op) = self.should_fail_operation {
            if fail_op == "save_config" {
                return Err(ApplicationError::ConfigurationError(
                    "Mock save_config failure".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn load_config(&self) -> Result<ProjectConfig, ApplicationError> {
        if !self.config_exists {
            return Err(ApplicationError::ProjectNotFound);
        }
        Ok(ProjectConfig::default_for_new_project())
    }

    fn update_gitignore(&self) -> Result<(), ApplicationError> {
        if let Some(ref fail_op) = self.should_fail_operation {
            if fail_op == "update_gitignore" {
                return Err(ApplicationError::FileSystemError(
                    "Mock update_gitignore failure".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn create_feature(&self, name: &str) -> Result<(), ApplicationError> {
        if let Some(ref fail_op) = self.should_fail_operation {
            if fail_op == "create_feature" {
                return Err(ApplicationError::FeatureCreationError(format!(
                    "Mock creation failure for: {}",
                    name
                )));
            }
        }
        if self.feature_exists || self.features.contains(&name.to_string()) {
            return Err(ApplicationError::FeatureAlreadyExists(name.to_string()));
        }
        Ok(())
    }

    fn save_document(
        &self,
        _memory_type: &str,
        _memories: &[Memory],
    ) -> Result<(), ApplicationError> {
        Ok(())
    }
}
