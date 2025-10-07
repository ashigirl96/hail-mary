use crate::domain::errors::DomainError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Project already exists")]
    ProjectAlreadyExists,

    #[error("Project not found")]
    ProjectNotFound,

    #[error("Spec already exists: {0}")]
    SpecAlreadyExists(String),

    #[error("Invalid spec name: {0}")]
    InvalidSpecName(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

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

impl ApplicationError {
    pub fn database_error(msg: impl Into<String>) -> Self {
        Self::DatabaseError(msg.into())
    }

    pub fn configuration_error(msg: impl Into<String>) -> Self {
        Self::ConfigurationError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_already_exists_error() {
        let error = ApplicationError::ProjectAlreadyExists;
        assert_eq!(error.to_string(), "Project already exists");
    }

    #[test]
    fn test_project_not_found_error() {
        let error = ApplicationError::ProjectNotFound;
        assert_eq!(error.to_string(), "Project not found");
    }

    #[test]
    fn test_spec_already_exists_error() {
        let error = ApplicationError::SpecAlreadyExists("test-feature".to_string());
        assert_eq!(error.to_string(), "Spec already exists: test-feature");
    }

    #[test]
    fn test_invalid_spec_name_error() {
        let error = ApplicationError::InvalidSpecName("Invalid_Name".to_string());
        assert_eq!(error.to_string(), "Invalid spec name: Invalid_Name");
    }

    #[test]
    fn test_database_error() {
        let error = ApplicationError::database_error("Connection failed");
        assert_eq!(error.to_string(), "Database error: Connection failed");
    }

    #[test]
    fn test_configuration_error() {
        let error = ApplicationError::configuration_error("Invalid config");
        assert_eq!(error.to_string(), "Configuration error: Invalid config");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_error = ApplicationError::from(io_error);

        assert!(app_error.to_string().contains("File not found"));
        assert!(matches!(app_error, ApplicationError::IoError(_)));
    }

    #[test]
    fn test_domain_error_conversion() {
        let domain_error = DomainError::InvalidSpecName("Bad_Name".to_string());
        let app_error = ApplicationError::from(domain_error);

        assert!(
            app_error
                .to_string()
                .contains("Invalid spec name: Bad_Name")
        );
        assert!(matches!(app_error, ApplicationError::DomainError(_)));
    }

    #[test]
    fn test_application_error_debug() {
        let error = ApplicationError::ProjectAlreadyExists;
        let debug_str = format!("{:?}", error);
        assert_eq!(debug_str, "ProjectAlreadyExists");
    }

    #[test]
    fn test_application_error_is_error() {
        let error = ApplicationError::ProjectNotFound;
        let boxed_error: Box<dyn std::error::Error> = Box::new(error);
        assert_eq!(boxed_error.to_string(), "Project not found");
    }
}
