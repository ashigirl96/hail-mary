use crate::application::errors::ApplicationError;
use crate::domain::entities::project::ProjectConfig;
use crate::domain::entities::steering::{SteeringBackupConfig, SteeringConfig};

/// Repository for managing project configuration
pub trait ConfigRepository: Send + Sync {
    /// Load the complete project configuration from config.toml
    fn load_config(&self) -> Result<ProjectConfig, ApplicationError>;

    /// Save the complete project configuration to config.toml
    fn save_config(&self, config: &ProjectConfig) -> Result<(), ApplicationError>;

    /// Load only the steering configuration
    fn load_steering_config(&self) -> Result<SteeringConfig, ApplicationError>;

    /// Load only the steering backup configuration
    fn load_steering_backup_config(&self) -> Result<SteeringBackupConfig, ApplicationError>;

    /// Ensure steering configuration exists, adding defaults if missing
    fn ensure_steering_config(&self) -> Result<(), ApplicationError>;

    /// Ensure steering backup configuration exists, adding defaults if missing
    fn ensure_steering_backup_config(&self) -> Result<(), ApplicationError>;
}
