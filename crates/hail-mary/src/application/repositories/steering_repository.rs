use crate::application::errors::ApplicationError;
use crate::domain::entities::steering::{Steering, SteeringConfig};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub name: String,
    pub created_at: SystemTime,
    pub path: PathBuf,
}

/// Repository interface for managing steering system
pub trait SteeringRepositoryInterface {
    /// Initialize steering system directories
    fn initialize_steering(&self) -> Result<(), ApplicationError>;

    /// Create steering files from configuration
    fn create_steering_files(&self, config: &SteeringConfig) -> Result<(), ApplicationError>;

    /// List all steering markdown files (excluding backup/ and draft/)
    fn list_steering_files(&self) -> Result<Vec<PathBuf>, ApplicationError>;

    /// Get the path for a specific steering file
    fn get_steering_path(&self, name: &str) -> Result<PathBuf, ApplicationError>;

    /// Create a backup of steering files
    fn create_steering_backup(
        &self,
        timestamp: &str,
        files: &[PathBuf],
    ) -> Result<(), ApplicationError>;

    /// List all existing steering backups sorted by creation time (oldest first)
    fn list_steering_backups(&self) -> Result<Vec<BackupInfo>, ApplicationError>;

    /// Delete the oldest steering backups
    fn delete_oldest_steering_backups(&self, count: usize) -> Result<(), ApplicationError>;

    /// Deploy embedded slash command markdown files
    fn deploy_slash_commands(&self) -> Result<(), ApplicationError>;

    /// Update .gitignore file with necessary entries
    fn update_gitignore(&self) -> Result<(), ApplicationError>;

    /// Check if project exists
    fn exists(&self) -> Result<bool, ApplicationError>;

    /// Load steering files with their content
    fn load_steering_files(
        &self,
        config: &SteeringConfig,
    ) -> Result<Vec<Steering>, ApplicationError>;
}
