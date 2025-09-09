//! Mock implementation of ConfigRepositoryInterface for testing

use crate::application::errors::ApplicationError;
use crate::application::repositories::ConfigRepositoryInterface;
use crate::domain::entities::project::{DocumentFormat, ProjectConfig};
use crate::domain::entities::steering::{SteeringBackupConfig, SteeringConfig};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct MockConfigRepository {
    config: RwLock<Option<ProjectConfig>>,
    operations_to_fail: RwLock<HashMap<String, bool>>,
}

impl MockConfigRepository {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(None),
            operations_to_fail: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_config(config: ProjectConfig) -> Self {
        Self {
            config: RwLock::new(Some(config)),
            operations_to_fail: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_config(&self, config: ProjectConfig) {
        *self.config.write().unwrap() = Some(config);
    }

    pub fn set_operation_to_fail(&self, operation: &str) {
        self.operations_to_fail
            .write()
            .unwrap()
            .insert(operation.to_string(), true);
    }

    pub fn clear_failure(&self) {
        self.operations_to_fail.write().unwrap().clear();
    }

    fn should_fail(&self, operation: &str) -> bool {
        self.operations_to_fail
            .read()
            .unwrap()
            .get(operation)
            .copied()
            .unwrap_or(false)
    }
}

impl ConfigRepositoryInterface for MockConfigRepository {
    fn load_config(&self) -> Result<ProjectConfig, ApplicationError> {
        if self.should_fail("load_config") {
            return Err(ApplicationError::ConfigurationError(
                "Mock load failure".to_string(),
            ));
        }

        Ok(self
            .config
            .read()
            .unwrap()
            .clone()
            .unwrap_or_else(|| ProjectConfig {
                instructions: "Mock instructions".to_string(),
                document_format: DocumentFormat::Markdown,
                steering: SteeringConfig::default_for_new_project(),
            }))
    }

    fn save_config(&self, config: &ProjectConfig) -> Result<(), ApplicationError> {
        if self.should_fail("save_config") {
            return Err(ApplicationError::ConfigurationError(
                "Mock save failure".to_string(),
            ));
        }

        *self.config.write().unwrap() = Some(config.clone());
        Ok(())
    }

    fn load_steering_config(&self) -> Result<SteeringConfig, ApplicationError> {
        if self.should_fail("load_steering_config") {
            return Err(ApplicationError::ConfigurationError(
                "Mock load steering failure".to_string(),
            ));
        }

        let config = self.load_config()?;
        Ok(config.steering)
    }

    fn load_steering_backup_config(&self) -> Result<SteeringBackupConfig, ApplicationError> {
        if self.should_fail("load_steering_backup_config") {
            return Err(ApplicationError::ConfigurationError(
                "Mock load backup failure".to_string(),
            ));
        }

        let config = self.load_config()?;
        Ok(config.steering.backup)
    }

    fn ensure_steering_config(&self) -> Result<(), ApplicationError> {
        if self.should_fail("ensure_steering_config") {
            return Err(ApplicationError::ConfigurationError(
                "Mock ensure steering failure".to_string(),
            ));
        }

        // For testing purposes, this is a no-op
        Ok(())
    }

    fn ensure_steering_backup_config(&self) -> Result<(), ApplicationError> {
        if self.should_fail("ensure_steering_backup_config") {
            return Err(ApplicationError::ConfigurationError(
                "Mock ensure backup failure".to_string(),
            ));
        }

        // For testing purposes, this is a no-op
        Ok(())
    }
}
