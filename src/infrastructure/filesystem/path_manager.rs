use crate::application::errors::ApplicationError;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug)]
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

        // Walk up directory tree to find .git (project root)
        loop {
            let git_dir = current_dir.join(".git");
            if git_dir.exists() {
                // Found project root with .git
                return Ok(Self::new(current_dir));
            }

            if !current_dir.pop() {
                return Err(ApplicationError::ProjectNotFound);
            }
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn kiro_dir(&self, absolute: bool) -> PathBuf {
        if absolute {
            self.project_root.join(".kiro")
        } else {
            PathBuf::from(".kiro")
        }
    }

    pub fn config_path(&self, absolute: bool) -> PathBuf {
        if absolute {
            self.kiro_dir(true).join("config.toml")
        } else {
            PathBuf::from(".kiro/config.toml")
        }
    }

    pub fn specs_dir(&self, absolute: bool) -> PathBuf {
        if absolute {
            self.kiro_dir(true).join("specs")
        } else {
            PathBuf::from(".kiro/specs")
        }
    }

    pub fn memory_dir(&self, absolute: bool) -> PathBuf {
        if absolute {
            self.kiro_dir(true).join("memory")
        } else {
            PathBuf::from(".kiro/memory")
        }
    }

    pub fn memory_db_path(&self, absolute: bool) -> PathBuf {
        if absolute {
            self.memory_dir(true).join("db.sqlite3")
        } else {
            PathBuf::from(".kiro/memory/db.sqlite3")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_path_manager_new() {
        // Test creating PathManager with project root
        let project_root = PathBuf::from("/test/project");
        let path_manager = PathManager::new(project_root.clone());

        assert_eq!(path_manager.project_root, project_root);
    }

    #[test]
    fn test_discover_finds_git_in_current_dir() {
        // Create temporary directory with .git
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).expect("Failed to create .git dir");

        // Change to temp directory
        let original_dir = env::current_dir().expect("Failed to get current dir");
        env::set_current_dir(&temp_dir).expect("Failed to change dir");

        // Test discovery
        let result = PathManager::discover();

        // Restore original directory
        env::set_current_dir(&original_dir).expect("Failed to restore dir");

        assert!(result.is_ok());
        let path_manager = result.unwrap();
        // Use canonicalize to handle macOS symlink resolution (/var -> /private/var)
        let expected = temp_dir
            .path()
            .canonicalize()
            .expect("Failed to canonicalize temp dir");
        let actual = path_manager
            .project_root()
            .canonicalize()
            .expect("Failed to canonicalize project root");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_discover_walks_up_directory_tree() {
        // Create temporary directory structure: temp/.git and temp/subdir/nested
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).expect("Failed to create .git dir");

        let subdir = temp_dir.path().join("subdir");
        let nested_dir = subdir.join("nested");
        fs::create_dir_all(&nested_dir).expect("Failed to create nested dirs");

        // Change to nested directory
        let original_dir = env::current_dir().expect("Failed to get current dir");
        env::set_current_dir(&nested_dir).expect("Failed to change to nested dir");

        // Test discovery - should find .git in parent directory
        let result = PathManager::discover();

        // Restore original directory before temp_dir goes out of scope
        env::set_current_dir(&original_dir).expect("Failed to restore dir");

        // Validate result
        assert!(result.is_ok());
        let path_manager = result.unwrap();
        // Use canonicalize to handle macOS symlink resolution (/var -> /private/var)
        let expected = temp_dir
            .path()
            .canonicalize()
            .expect("Failed to canonicalize temp dir");
        let actual = path_manager
            .project_root()
            .canonicalize()
            .expect("Failed to canonicalize project root");
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_discover_returns_error_when_no_git() {
        // Create temporary directory without .git
        let temp_dir = tempdir().expect("Failed to create temp dir");

        // Change to temp directory
        let original_dir = env::current_dir().expect("Failed to get current dir");
        env::set_current_dir(&temp_dir).expect("Failed to change dir");

        // Test discovery - should fail
        let result = PathManager::discover();

        // Restore original directory
        env::set_current_dir(&original_dir).expect("Failed to restore dir");

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ApplicationError::ProjectNotFound
        ));
    }

    #[test]
    fn test_kiro_dir_absolute_and_relative() {
        let project_root = PathBuf::from("/test/project");
        let path_manager = PathManager::new(project_root);

        // Test absolute path
        let absolute_path = path_manager.kiro_dir(true);
        assert_eq!(absolute_path, PathBuf::from("/test/project/.kiro"));

        // Test relative path
        let relative_path = path_manager.kiro_dir(false);
        assert_eq!(relative_path, PathBuf::from(".kiro"));
    }

    #[test]
    fn test_config_path_absolute_and_relative() {
        let project_root = PathBuf::from("/test/project");
        let path_manager = PathManager::new(project_root);

        // Test absolute path
        let absolute_path = path_manager.config_path(true);
        assert_eq!(
            absolute_path,
            PathBuf::from("/test/project/.kiro/config.toml")
        );

        // Test relative path
        let relative_path = path_manager.config_path(false);
        assert_eq!(relative_path, PathBuf::from(".kiro/config.toml"));
    }

    #[test]
    fn test_specs_dir_absolute_and_relative() {
        let project_root = PathBuf::from("/test/project");
        let path_manager = PathManager::new(project_root);

        // Test absolute path
        let absolute_path = path_manager.specs_dir(true);
        assert_eq!(absolute_path, PathBuf::from("/test/project/.kiro/specs"));

        // Test relative path
        let relative_path = path_manager.specs_dir(false);
        assert_eq!(relative_path, PathBuf::from(".kiro/specs"));
    }

    #[test]
    fn test_memory_dir_absolute_and_relative() {
        let project_root = PathBuf::from("/test/project");
        let path_manager = PathManager::new(project_root);

        // Test absolute path
        let absolute_path = path_manager.memory_dir(true);
        assert_eq!(absolute_path, PathBuf::from("/test/project/.kiro/memory"));

        // Test relative path
        let relative_path = path_manager.memory_dir(false);
        assert_eq!(relative_path, PathBuf::from(".kiro/memory"));
    }

    #[test]
    fn test_memory_db_path_absolute_and_relative() {
        let project_root = PathBuf::from("/test/project");
        let path_manager = PathManager::new(project_root);

        // Test absolute path
        let absolute_path = path_manager.memory_db_path(true);
        assert_eq!(
            absolute_path,
            PathBuf::from("/test/project/.kiro/memory/db.sqlite3")
        );

        // Test relative path
        let relative_path = path_manager.memory_db_path(false);
        assert_eq!(relative_path, PathBuf::from(".kiro/memory/db.sqlite3"));
    }

    #[test]
    fn test_project_root_accessor() {
        let project_root = PathBuf::from("/test/project");
        let path_manager = PathManager::new(project_root.clone());

        assert_eq!(path_manager.project_root(), project_root.as_path());
    }

    #[test]
    fn test_discover_root_directory_handling() {
        // Test that discovery stops at root directory without infinite loop
        // This is a safety test to ensure we don't get stuck in pathological cases

        // We can't easily test this without potentially affecting the real filesystem,
        // but we can at least ensure that the logic handles edge cases properly.
        // This test would be implemented once we have the actual discovery logic.

        // For now, just verify the function signature exists
        let _ = PathManager::discover();
    }
}
