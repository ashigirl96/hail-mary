use crate::application::errors::ApplicationError;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathManager {
    project_root: PathBuf,
}

impl PathManager {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub fn discover() -> Result<Self, ApplicationError> {
        let mut current_dir = env::current_dir().map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to get current directory: {}", e))
        })?;

        loop {
            // Check for .specs directory
            if current_dir.join(".specs").exists() {
                return Ok(Self::new(current_dir.clone()));
            }

            // Check for .git directory (git repository root)
            if current_dir.join(".git").exists() {
                return Ok(Self::new(current_dir.clone()));
            }

            if !current_dir.pop() {
                return Err(ApplicationError::ProjectNotFound);
            }
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn specs_dir(&self) -> PathBuf {
        self.project_root.join(".specs")
    }

    pub fn archive_dir(&self) -> PathBuf {
        self.project_root.join(".specs").join("archive")
    }
}
