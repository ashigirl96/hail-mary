//! Mock implementation of ConfigRepositoryInterface for testing

use crate::application::errors::ApplicationError;
use crate::application::repositories::ConfigRepositoryInterface;
use crate::domain::value_objects::steering::{SteeringBackupConfig, SteeringConfig};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct MockConfigRepository {
    steering_config: RwLock<Option<SteeringConfig>>,
    operations_to_fail: RwLock<HashMap<String, bool>>,
}

impl MockConfigRepository {
    pub fn new() -> Self {
        Self {
            steering_config: RwLock::new(None),
            operations_to_fail: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_steering_config(config: SteeringConfig) -> Self {
        Self {
            steering_config: RwLock::new(Some(config)),
            operations_to_fail: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_steering_config(&self, config: SteeringConfig) {
        *self.steering_config.write().unwrap() = Some(config);
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
    fn load_steering_config(&self) -> Result<SteeringConfig, ApplicationError> {
        if self.should_fail("load_steering_config") {
            return Err(ApplicationError::ConfigurationError(
                "Mock load steering failure".to_string(),
            ));
        }

        Ok(self
            .steering_config
            .read()
            .unwrap()
            .clone()
            .unwrap_or_else(SteeringConfig::default_for_new_project))
    }

    fn load_steering_backup_config(&self) -> Result<SteeringBackupConfig, ApplicationError> {
        if self.should_fail("load_steering_backup_config") {
            return Err(ApplicationError::ConfigurationError(
                "Mock load backup failure".to_string(),
            ));
        }

        let steering_config = self.load_steering_config()?;
        Ok(steering_config.backup)
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

    fn ensure_allowed_operations(&self) -> Result<(), ApplicationError> {
        if self.should_fail("ensure_allowed_operations") {
            return Err(ApplicationError::ConfigurationError(
                "Mock ensure allowed operations failure".to_string(),
            ));
        }

        // For testing purposes, this is a no-op
        Ok(())
    }
}
