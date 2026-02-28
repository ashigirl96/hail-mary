use crate::application::errors::ApplicationError;
use std::path::PathBuf;

pub trait SpecRepositoryInterface {
    fn create_spec(&self, name: &str) -> Result<(), ApplicationError>;

    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError>;

    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError>;

    fn get_spec_path(&self, name: &str) -> Result<PathBuf, ApplicationError>;

    fn list_archived_specs(&self) -> Result<Vec<String>, ApplicationError>;
}
