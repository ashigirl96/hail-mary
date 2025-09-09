pub mod config_repository;
pub mod project_repository;
pub mod spec_repository;
pub mod steering_repository;

// Re-export traits and types for easier access
pub use config_repository::ConfigRepository;
pub use project_repository::{BackupInfo, ProjectRepository};
pub use spec_repository::SpecRepository;
pub use steering_repository::SteeringRepository;
