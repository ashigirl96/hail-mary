use crate::domain::errors::DomainError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Project not found")]
    ProjectNotFound,

    #[error("Spec already exists: {0}")]
    SpecAlreadyExists(String),

    #[error("Invalid spec name: {0}")]
    InvalidSpecName(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),

    #[error("Project initialization error: {0}")]
    ProjectInitializationError(String),

    #[error("File system error: {0}")]
    FileSystemError(String),

    #[error("Spec creation error: {0}")]
    SpecCreationError(String),

    #[error("Spec directory not found: {0}")]
    SpecNotFound(String),

    #[error("Invalid spec directory: {0}")]
    InvalidSpecDirectory(String),

    #[error("Spec already exists in archive: {0}")]
    ArchiveAlreadyExists(String),

    #[error("Process launch error: {0}")]
    ProcessLaunchError(String),
}
