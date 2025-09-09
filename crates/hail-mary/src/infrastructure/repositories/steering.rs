use crate::application::errors::ApplicationError;
use crate::application::repositories::{BackupInfo, SteeringRepository};
use crate::domain::entities::steering::SteeringConfig;
use crate::infrastructure::filesystem::path_manager::PathManager;
use std::fs;
use std::path::PathBuf;

pub struct SteeringRepositoryImpl {
    path_manager: PathManager,
}

impl SteeringRepositoryImpl {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }

    fn steering_dir(&self) -> PathBuf {
        self.path_manager.kiro_dir(true).join("steering")
    }

    fn backup_dir(&self) -> PathBuf {
        self.steering_dir().join("backup")
    }
}

impl SteeringRepository for SteeringRepositoryImpl {
    fn initialize_steering(&self) -> Result<(), ApplicationError> {
        // Create .kiro/steering directory
        let steering_dir = self.steering_dir();
        fs::create_dir_all(&steering_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create steering directory: {}", e))
        })?;

        Ok(())
    }

    fn create_steering_files(&self, config: &SteeringConfig) -> Result<(), ApplicationError> {
        for steering_type in &config.types {
            let file_path = self
                .steering_dir()
                .join(format!("{}.md", steering_type.name));

            // Never overwrite existing files
            if file_path.exists() {
                continue;
            }

            // Generate simple markdown header only
            let content = format!("# {}\n\n", steering_type.name);

            fs::write(file_path, content).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to write steering file {}: {}",
                    steering_type.name, e
                ))
            })?;
        }
        Ok(())
    }

    fn list_steering_files(&self) -> Result<Vec<PathBuf>, ApplicationError> {
        let steering_dir = self.steering_dir();

        if !steering_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        let entries = fs::read_dir(&steering_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read steering directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip backup and draft directories
            if file_name == "backup" || file_name == "draft" {
                continue;
            }

            // Only include markdown files
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                // Return relative path from steering directory
                files.push(PathBuf::from(file_name));
            }
        }

        Ok(files)
    }

    fn create_steering_backup(
        &self,
        timestamp: &str,
        files: &[PathBuf],
    ) -> Result<(), ApplicationError> {
        let steering_dir = self.steering_dir();
        let backup_dir = self.backup_dir().join(timestamp);

        // Create backup directory
        fs::create_dir_all(&backup_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create backup directory: {}", e))
        })?;

        // Copy each file to backup
        for file in files {
            let source = steering_dir.join(file);
            let dest = backup_dir.join(file);

            if source.exists() {
                fs::copy(&source, &dest).map_err(|e| {
                    ApplicationError::FileSystemError(format!(
                        "Failed to backup file {}: {}",
                        file.display(),
                        e
                    ))
                })?;
            }
        }

        Ok(())
    }

    fn list_steering_backups(&self) -> Result<Vec<BackupInfo>, ApplicationError> {
        let backup_dir = self.backup_dir();

        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let entries = fs::read_dir(&backup_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read backup directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to read directory entry: {}", e))
            })?;

            if entry
                .file_type()
                .map_err(|e| {
                    ApplicationError::FileSystemError(format!("Failed to get file type: {}", e))
                })?
                .is_dir()
            {
                let metadata = entry.metadata().map_err(|e| {
                    ApplicationError::FileSystemError(format!("Failed to get metadata: {}", e))
                })?;

                backups.push(BackupInfo {
                    name: entry.file_name().to_string_lossy().to_string(),
                    created_at: metadata.created().unwrap_or_else(|_| {
                        metadata.modified().unwrap_or(std::time::SystemTime::now())
                    }),
                    path: entry.path(),
                });
            }
        }

        // Sort by creation time (oldest first)
        backups.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        Ok(backups)
    }

    fn delete_oldest_steering_backups(&self, count: usize) -> Result<(), ApplicationError> {
        let backups = self.list_steering_backups()?;

        // Take the oldest 'count' backups
        for backup in backups.iter().take(count) {
            fs::remove_dir_all(&backup.path).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to delete backup {}: {}",
                    backup.name, e
                ))
            })?;
        }

        Ok(())
    }

    fn deploy_slash_commands(&self) -> Result<(), ApplicationError> {
        use crate::infrastructure::embedded_resources::EmbeddedSlashCommands;

        // Create .claude/commands/hm directory
        let claude_dir = self.path_manager.project_root().join(".claude");
        let commands_dir = claude_dir.join("commands");
        let hm_dir = commands_dir.join("hm");

        // Create directory structure
        fs::create_dir_all(&hm_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!(
                "Failed to create .claude/commands/hm directory: {}",
                e
            ))
        })?;

        // Deploy all embedded markdown files (force overwrite)
        for (filename, content) in EmbeddedSlashCommands::get_all() {
            let file_path = hm_dir.join(filename);
            fs::write(&file_path, content).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to write slash command file {}: {}",
                    filename, e
                ))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::TestDirectory;
    use crate::domain::entities::steering::SteeringType;

    fn create_test_repository() -> (SteeringRepositoryImpl, TestDirectory) {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let repository = SteeringRepositoryImpl::new(path_manager);
        (repository, test_dir)
    }

    #[test]
    fn test_initialize_steering() {
        let (repository, test_dir) = create_test_repository();

        // Create .kiro directory first
        fs::create_dir_all(test_dir.path().join(".kiro")).unwrap();

        let result = repository.initialize_steering();
        assert!(result.is_ok());

        // Verify steering directory was created
        assert!(test_dir.path().join(".kiro/steering").exists());
    }

    #[test]
    fn test_create_steering_files() {
        let (repository, test_dir) = create_test_repository();

        // Create steering directory
        let steering_dir = test_dir.path().join(".kiro/steering");
        fs::create_dir_all(&steering_dir).unwrap();

        // Create config with steering types
        let config = SteeringConfig {
            types: vec![
                SteeringType {
                    name: "product".to_string(),
                    purpose: "Product overview".to_string(),
                    criteria: vec![],
                },
                SteeringType {
                    name: "tech".to_string(),
                    purpose: "Technical stack".to_string(),
                    criteria: vec![],
                },
            ],
            backup: Default::default(),
        };

        repository.create_steering_files(&config).unwrap();

        // Verify files were created
        assert!(steering_dir.join("product.md").exists());
        assert!(steering_dir.join("tech.md").exists());

        // Verify content
        let content = fs::read_to_string(steering_dir.join("product.md")).unwrap();
        assert_eq!(content, "# product\n\n");
    }

    #[test]
    fn test_create_steering_files_does_not_overwrite() {
        let (repository, test_dir) = create_test_repository();

        // Create steering directory with existing file
        let steering_dir = test_dir.path().join(".kiro/steering");
        fs::create_dir_all(&steering_dir).unwrap();
        fs::write(steering_dir.join("product.md"), "Existing content").unwrap();

        // Create config
        let config = SteeringConfig {
            types: vec![SteeringType {
                name: "product".to_string(),
                purpose: "Product overview".to_string(),
                criteria: vec![],
            }],
            backup: Default::default(),
        };

        repository.create_steering_files(&config).unwrap();

        // Verify existing content is preserved
        let content = fs::read_to_string(steering_dir.join("product.md")).unwrap();
        assert_eq!(content, "Existing content");
    }

    #[test]
    fn test_list_steering_files() {
        let (repository, test_dir) = create_test_repository();

        // Create steering directory with files
        let steering_dir = test_dir.path().join(".kiro/steering");
        fs::create_dir_all(&steering_dir).unwrap();
        fs::write(steering_dir.join("product.md"), "").unwrap();
        fs::write(steering_dir.join("tech.md"), "").unwrap();
        fs::create_dir(steering_dir.join("backup")).unwrap(); // Should be ignored
        fs::create_dir(steering_dir.join("draft")).unwrap(); // Should be ignored

        let files = repository.list_steering_files().unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.contains(&PathBuf::from("product.md")));
        assert!(files.contains(&PathBuf::from("tech.md")));
    }

    #[test]
    fn test_create_and_list_steering_backup() {
        let (repository, test_dir) = create_test_repository();

        // Create steering directory with files
        let steering_dir = test_dir.path().join(".kiro/steering");
        fs::create_dir_all(&steering_dir).unwrap();
        fs::write(steering_dir.join("product.md"), "Product content").unwrap();
        fs::write(steering_dir.join("tech.md"), "Tech content").unwrap();

        // Create backup
        let files = vec![PathBuf::from("product.md"), PathBuf::from("tech.md")];
        repository
            .create_steering_backup("2025-01-01-123456", &files)
            .unwrap();

        // List backups
        let backups = repository.list_steering_backups().unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].name, "2025-01-01-123456");

        // Verify backup content
        let backup_dir = steering_dir.join("backup/2025-01-01-123456");
        assert!(backup_dir.join("product.md").exists());
        assert!(backup_dir.join("tech.md").exists());
    }

    #[test]
    fn test_delete_oldest_steering_backups() {
        let (repository, test_dir) = create_test_repository();

        // Create backup directory with multiple backups
        let backup_dir = test_dir.path().join(".kiro/steering/backup");
        fs::create_dir_all(&backup_dir).unwrap();
        fs::create_dir(backup_dir.join("2025-01-01")).unwrap();
        fs::create_dir(backup_dir.join("2025-01-02")).unwrap();
        fs::create_dir(backup_dir.join("2025-01-03")).unwrap();

        // Delete oldest 2
        repository.delete_oldest_steering_backups(2).unwrap();

        // Verify only newest remains
        let backups = repository.list_steering_backups().unwrap();
        assert_eq!(backups.len(), 1);
        assert!(backup_dir.join("2025-01-03").exists());
        assert!(!backup_dir.join("2025-01-01").exists());
        assert!(!backup_dir.join("2025-01-02").exists());
    }
}
