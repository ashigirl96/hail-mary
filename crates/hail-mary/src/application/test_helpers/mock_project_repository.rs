use crate::application::errors::ApplicationError;
use crate::application::repositories::{BackupInfo, ProjectRepository};
use crate::domain::entities::project::ProjectConfig;
use crate::domain::entities::steering::{SteeringBackupConfig, SteeringConfig};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

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
    // Steering backup behavior
    steering_files: Option<Vec<PathBuf>>,
    steering_backups: Option<Vec<BackupInfo>>,
    steering_backup_config: Option<SteeringBackupConfig>,
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
            steering_files: None, // None means use default behavior
            steering_backups: None,
            steering_backup_config: None,
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

    // Steering backup control methods
    pub fn set_steering_files(&mut self, files: Vec<PathBuf>) {
        self.steering_files = Some(files);
    }

    pub fn set_steering_backups(&mut self, backups: Vec<BackupInfo>) {
        self.steering_backups = Some(backups);
    }

    pub fn set_steering_backup_config(&mut self, config: SteeringBackupConfig) {
        self.steering_backup_config = Some(config);
    }

    pub fn with_steering_files(mut self, files: Vec<PathBuf>) -> Self {
        self.steering_files = Some(files);
        self
    }

    pub fn with_empty_steering_files(mut self) -> Self {
        self.steering_files = Some(Vec::new());
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

    fn save_config(&self) -> Result<(), ApplicationError> {
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

    fn deploy_slash_commands(&self) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Failed to deploy slash commands".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "deploy_slash_commands"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock deploy_slash_commands failure".to_string(),
            ));
        }
        Ok(())
    }

    fn list_steering_files(&self) -> Result<Vec<PathBuf>, ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Failed to list steering files".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "list_steering_files"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock list_steering_files failure".to_string(),
            ));
        }

        // Return configured files or default
        if let Some(ref files) = self.steering_files {
            Ok(files.clone())
        } else {
            // Default mock steering files
            Ok(vec![
                PathBuf::from("product.md"),
                PathBuf::from("tech.md"),
                PathBuf::from("structure.md"),
            ])
        }
    }

    fn create_steering_backup(
        &self,
        _timestamp: &str,
        _files: &[PathBuf],
    ) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Failed to create steering backup".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "create_steering_backup"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock create_steering_backup failure".to_string(),
            ));
        }
        Ok(())
    }

    fn list_steering_backups(&self) -> Result<Vec<BackupInfo>, ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Failed to list steering backups".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "list_steering_backups"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock list_steering_backups failure".to_string(),
            ));
        }

        // Return configured backups or default
        if let Some(ref backups) = self.steering_backups {
            Ok(backups.clone())
        } else {
            // Default mock backups
            Ok(vec![
                BackupInfo {
                    name: "2025-01-01-10-00".to_string(),
                    created_at: SystemTime::now(),
                    path: PathBuf::from(".kiro/steering/backup/2025-01-01-10-00"),
                },
                BackupInfo {
                    name: "2025-01-02-10-00".to_string(),
                    created_at: SystemTime::now(),
                    path: PathBuf::from(".kiro/steering/backup/2025-01-02-10-00"),
                },
            ])
        }
    }

    fn delete_oldest_steering_backups(&self, _count: usize) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::FileSystemError(
                "Failed to delete steering backups".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "delete_oldest_steering_backups"
        {
            return Err(ApplicationError::FileSystemError(
                "Mock delete_oldest_steering_backups failure".to_string(),
            ));
        }
        Ok(())
    }

    fn load_steering_backup_config(&self) -> Result<SteeringBackupConfig, ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::ConfigurationError(
                "Failed to load steering backup config".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "load_steering_backup_config"
        {
            return Err(ApplicationError::ConfigurationError(
                "Mock load_steering_backup_config failure".to_string(),
            ));
        }

        // Return configured config or default
        if let Some(ref config) = self.steering_backup_config {
            Ok(config.clone())
        } else {
            Ok(SteeringBackupConfig::default())
        }
    }

    fn ensure_steering_backup_config(&self) -> Result<(), ApplicationError> {
        if self.should_fail_next_operation {
            return Err(ApplicationError::ConfigurationError(
                "Failed to ensure steering backup config".to_string(),
            ));
        }
        if let Some(ref fail_op) = self.should_fail_operation
            && fail_op == "ensure_steering_backup_config"
        {
            return Err(ApplicationError::ConfigurationError(
                "Mock ensure_steering_backup_config failure".to_string(),
            ));
        }
        Ok(())
    }
}
