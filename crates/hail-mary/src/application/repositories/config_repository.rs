use crate::application::errors::ApplicationError;
use crate::domain::value_objects::steering::{SpecConfig, SteeringBackupConfig, SteeringConfig};

/// Repository interface for managing project configuration
pub trait ConfigRepositoryInterface {
    /// Load only the steering configuration section
    fn load_steering_config(&self) -> Result<SteeringConfig, ApplicationError>;

    /// Load only the steering backup configuration
    fn load_steering_backup_config(&self) -> Result<SteeringBackupConfig, ApplicationError>;

    /// Ensure steering configuration exists, adding defaults if missing
    fn ensure_steering_config(&self) -> Result<(), ApplicationError>;

    /// Ensure steering backup configuration exists, adding defaults if missing
    fn ensure_steering_backup_config(&self) -> Result<(), ApplicationError>;

    /// Ensure allowed_operations exists for all steering types, adding defaults if missing
    fn ensure_allowed_operations(&self) -> Result<(), ApplicationError>;

    /// Load only the spec configuration section
    fn load_spec_config(&self) -> Result<SpecConfig, ApplicationError>;

    /// Ensure spec configuration exists, adding defaults if missing
    fn ensure_spec_config(&self) -> Result<(), ApplicationError>;
}
