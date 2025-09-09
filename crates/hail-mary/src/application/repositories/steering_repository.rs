use crate::application::errors::ApplicationError;
use crate::application::repositories::BackupInfo;
use crate::domain::entities::steering::SteeringConfig;
use std::path::PathBuf;

/// Repository for managing steering system and backups
pub trait SteeringRepository: Send + Sync {
    /// Initialize steering system directories
    fn initialize_steering(&self) -> Result<(), ApplicationError>;

    /// Create steering files from configuration
    fn create_steering_files(&self, config: &SteeringConfig) -> Result<(), ApplicationError>;

    /// List all steering markdown files (excluding backup/ and draft/ directories)
    fn list_steering_files(&self) -> Result<Vec<PathBuf>, ApplicationError>;

    /// Create a backup of steering files in a timestamped directory
    fn create_steering_backup(
        &self,
        timestamp: &str,
        files: &[PathBuf],
    ) -> Result<(), ApplicationError>;

    /// List all existing steering backups sorted by creation time (oldest first)
    fn list_steering_backups(&self) -> Result<Vec<BackupInfo>, ApplicationError>;

    /// Delete the oldest steering backups
    fn delete_oldest_steering_backups(&self, count: usize) -> Result<(), ApplicationError>;

    /// Deploy embedded slash command markdown files to .claude/commands/hm
    fn deploy_slash_commands(&self) -> Result<(), ApplicationError>;
}
