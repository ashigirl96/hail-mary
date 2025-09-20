//! Mock implementation of SteeringRepositoryInterface for testing

use crate::application::errors::ApplicationError;
use crate::application::repositories::steering_repository::{
    BackupInfo, SteeringRepositoryInterface,
};
use crate::domain::entities::steering::{Steering, SteeringConfig};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::RwLock;
use std::time::SystemTime;

#[derive(Debug, Default)]
pub struct MockSteeringRepository {
    steering_files: RwLock<HashSet<PathBuf>>,
    backups: RwLock<Vec<BackupInfo>>,
    operations_to_fail: RwLock<HashMap<String, bool>>,
    project_exists: RwLock<bool>,
}

impl MockSteeringRepository {
    pub fn new() -> Self {
        Self {
            steering_files: RwLock::new(HashSet::new()),
            backups: RwLock::new(Vec::new()),
            operations_to_fail: RwLock::new(HashMap::new()),
            project_exists: RwLock::new(false),
        }
    }

    pub fn with_steering_files(files: Vec<PathBuf>) -> Self {
        Self {
            steering_files: RwLock::new(files.into_iter().collect()),
            backups: RwLock::new(Vec::new()),
            operations_to_fail: RwLock::new(HashMap::new()),
            project_exists: RwLock::new(true),
        }
    }

    pub fn set_operation_to_fail(&self, operation: &str) {
        self.operations_to_fail
            .write()
            .unwrap()
            .insert(operation.to_string(), true);
    }

    pub fn clear_failure(&self) {
        self.operations_to_fail.write().unwrap().clear();
    }

    pub fn set_project_exists(&self, exists: bool) {
        *self.project_exists.write().unwrap() = exists;
    }

    fn should_fail(&self, operation: &str) -> bool {
        self.operations_to_fail
            .read()
            .unwrap()
            .get(operation)
            .copied()
            .unwrap_or(false)
    }
}

impl SteeringRepositoryInterface for MockSteeringRepository {
    fn initialize_steering(&self) -> Result<(), ApplicationError> {
        if self.should_fail("initialize_steering") {
            return Err(ApplicationError::FileSystemError(
                "Mock initialize failure".to_string(),
            ));
        }

        // For testing purposes, this is a no-op
        Ok(())
    }

    fn create_steering_files(&self, config: &SteeringConfig) -> Result<(), ApplicationError> {
        if self.should_fail("create_steering_files") {
            return Err(ApplicationError::FileSystemError(
                "Mock create files failure".to_string(),
            ));
        }

        let mut files = self.steering_files.write().unwrap();
        for steering_type in &config.types {
            files.insert(PathBuf::from(format!("{}.md", steering_type.name)));
        }
        Ok(())
    }

    fn list_steering_files(&self) -> Result<Vec<PathBuf>, ApplicationError> {
        if self.should_fail("list_steering_files") {
            return Err(ApplicationError::FileSystemError(
                "Mock list files failure".to_string(),
            ));
        }

        let files = self
            .steering_files
            .read()
            .unwrap()
            .iter()
            .cloned()
            .collect();
        Ok(files)
    }

    fn get_steering_path(&self, name: &str) -> Result<PathBuf, ApplicationError> {
        // Return a path that will be checked by the mock
        let path = PathBuf::from(format!(".kiro/steering/{}.md", name));
        // Check if this file was set in our mock
        if self.steering_files.read().unwrap().contains(&path) {
            // Return a special marker path that exists (current directory)
            // This is a workaround for testing since we check file_path.exists()
            Ok(std::env::current_dir().unwrap())
        } else {
            // Return a path that definitely doesn't exist
            Ok(PathBuf::from("/nonexistent/path/that/will/not/exist"))
        }
    }

    fn create_steering_backup(
        &self,
        timestamp: &str,
        _files: &[PathBuf],
    ) -> Result<(), ApplicationError> {
        if self.should_fail("create_steering_backup") {
            return Err(ApplicationError::FileSystemError(
                "Mock backup failure".to_string(),
            ));
        }

        let backup_info = BackupInfo {
            name: timestamp.to_string(),
            created_at: SystemTime::now(),
            path: PathBuf::from(format!(".kiro/steering/backup/{}", timestamp)),
        };
        self.backups.write().unwrap().push(backup_info);
        Ok(())
    }

    fn list_steering_backups(&self) -> Result<Vec<BackupInfo>, ApplicationError> {
        if self.should_fail("list_steering_backups") {
            return Err(ApplicationError::FileSystemError(
                "Mock list backups failure".to_string(),
            ));
        }

        Ok(self.backups.read().unwrap().clone())
    }

    fn delete_oldest_steering_backups(&self, count: usize) -> Result<(), ApplicationError> {
        if self.should_fail("delete_oldest_steering_backups") {
            return Err(ApplicationError::FileSystemError(
                "Mock delete backups failure".to_string(),
            ));
        }

        let mut backups = self.backups.write().unwrap();
        for _ in 0..std::cmp::min(count, backups.len()) {
            backups.remove(0);
        }
        Ok(())
    }

    fn deploy_slash_commands(&self) -> Result<(), ApplicationError> {
        if self.should_fail("deploy_slash_commands") {
            return Err(ApplicationError::FileSystemError(
                "Mock deploy failure".to_string(),
            ));
        }

        // For testing purposes, this is a no-op
        Ok(())
    }

    fn update_gitignore(&self) -> Result<(), ApplicationError> {
        if self.should_fail("update_gitignore") {
            return Err(ApplicationError::FileSystemError(
                "Mock gitignore failure".to_string(),
            ));
        }

        // For testing purposes, this is a no-op
        Ok(())
    }

    fn exists(&self) -> Result<bool, ApplicationError> {
        if self.should_fail("exists") {
            return Err(ApplicationError::FileSystemError(
                "Mock exists failure".to_string(),
            ));
        }

        Ok(*self.project_exists.read().unwrap())
    }

    fn load_steering_files(
        &self,
        _config: &SteeringConfig,
    ) -> Result<Vec<Steering>, ApplicationError> {
        if self.should_fail("load_steering_files") {
            return Err(ApplicationError::FileSystemError(
                "Mock load steering failure".to_string(),
            ));
        }

        // For testing purposes, return empty vector
        Ok(vec![])
    }
}
