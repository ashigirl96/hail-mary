use crate::application::errors::ApplicationError;
use std::path::PathBuf;

/// Repository interface for managing feature specifications
pub trait SpecRepositoryInterface {
    /// Create a new feature specification with template files
    fn create_feature(&self, name: &str) -> Result<(), ApplicationError>;

    /// List all specification directories
    /// Returns a vector of (name, is_archived) tuples
    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError>;

    /// Mark a specification as complete by moving it to archive
    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError>;

    /// Get the path to a specification directory
    fn get_spec_path(&self, name: &str) -> Result<PathBuf, ApplicationError>;

    /// List all archived specifications
    fn list_archived_specs(&self) -> Result<Vec<String>, ApplicationError>;

    /// Check if a spec is a PBI (has SBI subdirectories)
    fn is_pbi(&self, spec_name: &str) -> Result<bool, ApplicationError>;

    /// List all SBIs in a PBI
    fn list_sbis(&self, pbi_name: &str) -> Result<Vec<String>, ApplicationError>;

    /// Create a new SBI in a PBI (used by TUI "Create new SBI")
    fn create_sbi(
        &self,
        pbi_name: &str,
        sbi_name: &str,
        sbi_type: &str,
    ) -> Result<(), ApplicationError>;
}
