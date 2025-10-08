use crate::application::errors::ApplicationError;
use crate::application::repositories::steering_repository::{
    BackupInfo, SteeringRepositoryInterface,
};
use crate::domain::value_objects::steering::{Steering, SteeringConfig};
use crate::infrastructure::filesystem::path_manager::PathManager;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct SteeringRepository {
    path_manager: PathManager,
}

impl SteeringRepository {
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

impl SteeringRepositoryInterface for SteeringRepository {
    fn initialize_steering(&self) -> Result<(), ApplicationError> {
        // Create .kiro/steering directory
        let steering_dir = self.steering_dir();
        fs::create_dir_all(&steering_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create steering directory: {}", e))
        })?;

        // Create .kiro/specs directory during initialization
        let specs_dir = self.path_manager.specs_dir(true);
        fs::create_dir_all(&specs_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create specs directory: {}", e))
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
                    "Failed to create steering file for {}: {}",
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
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            // Skip backup directory
            if file_name == "backup" {
                continue;
            }

            // Only include markdown files
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                // Return RELATIVE path (just the filename), not absolute
                files.push(PathBuf::from(path.file_name().unwrap()));
            }
        }

        files.sort();
        Ok(files)
    }

    fn get_steering_path(&self, name: &str) -> Result<PathBuf, ApplicationError> {
        let path = self.steering_dir().join(format!("{}.md", name));
        Ok(path)
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

                // Use modified time as it's more reliable across platforms
                // This is just for the BackupInfo struct, actual sorting will be by name
                let created_at = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

                backups.push(BackupInfo {
                    name: entry.file_name().to_string_lossy().to_string(),
                    created_at,
                    path: entry.path(),
                });
            }
        }

        // Sort by backup name (which contains timestamp) for consistent ordering
        // This ensures tests work reliably across different filesystems
        // Format is typically: YYYY-MM-DD-HH-MM
        backups.sort_by(|a, b| a.name.cmp(&b.name));
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
        use crate::infrastructure::embedded_resources::{
            EmbeddedAgents, EmbeddedPbiCommands, EmbeddedSlashCommands, EmbeddedSpecCommands,
        };

        // Create .claude directory structure
        let claude_dir = self.path_manager.project_root().join(".claude");
        let commands_dir = claude_dir.join("commands");

        // Remove existing .claude/commands/hm directory to ensure clean deployment
        let hm_dir = commands_dir.join("hm");
        if hm_dir.exists() {
            fs::remove_dir_all(&hm_dir).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to remove existing .claude/commands/hm directory: {}",
                    e
                ))
            })?;
        }

        // Remove existing .claude/commands/pbi directory to ensure clean deployment
        let pbi_dir = commands_dir.join("pbi");
        if pbi_dir.exists() {
            fs::remove_dir_all(&pbi_dir).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to remove existing .claude/commands/pbi directory: {}",
                    e
                ))
            })?;
        }

        // Remove existing .claude/commands/spec directory to ensure clean deployment
        let spec_dir = commands_dir.join("spec");
        if spec_dir.exists() {
            fs::remove_dir_all(&spec_dir).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to remove existing .claude/commands/spec directory: {}",
                    e
                ))
            })?;
        }

        // Create .claude/commands/hm directory
        fs::create_dir_all(&hm_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!(
                "Failed to create .claude/commands/hm directory: {}",
                e
            ))
        })?;

        // Create .claude/commands/pbi directory
        fs::create_dir_all(&pbi_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!(
                "Failed to create .claude/commands/pbi directory: {}",
                e
            ))
        })?;

        // Create .claude/commands/spec directory
        fs::create_dir_all(&spec_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!(
                "Failed to create .claude/commands/spec directory: {}",
                e
            ))
        })?;

        // Create .claude/agents directory
        let agents_dir = claude_dir.join("agents");
        fs::create_dir_all(&agents_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!(
                "Failed to create .claude/agents directory: {}",
                e
            ))
        })?;

        // Deploy embedded hm commands (always overwrite for consistency)
        for (name, content) in EmbeddedSlashCommands::get_all() {
            let file_path = hm_dir.join(name);
            fs::write(&file_path, content).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to write slash command {}: {}",
                    name, e
                ))
            })?;
        }

        // Deploy embedded pbi commands (always overwrite for consistency)
        for (name, content) in EmbeddedPbiCommands::get_all() {
            let file_path = pbi_dir.join(name);
            fs::write(&file_path, content).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to write PBI command {}: {}",
                    name, e
                ))
            })?;
        }

        // Deploy embedded spec commands (always overwrite for consistency)
        for (name, content) in EmbeddedSpecCommands::get_all() {
            let file_path = spec_dir.join(name);
            fs::write(&file_path, content).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to write spec command {}: {}",
                    name, e
                ))
            })?;
        }

        // Deploy embedded agents (always overwrite for consistency)
        for (name, content) in EmbeddedAgents::get_all() {
            let file_path = agents_dir.join(name);
            fs::write(&file_path, content).map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to write agent {}: {}", name, e))
            })?;
        }

        Ok(())
    }

    fn update_gitignore(&self) -> Result<(), ApplicationError> {
        let gitignore_path = self.path_manager.project_root().join(".gitignore");

        let content = if gitignore_path.exists() {
            fs::read_to_string(&gitignore_path).map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to read .gitignore: {}", e))
            })?
        } else {
            String::new()
        };

        // No database files to exclude since we're using file-based steering system
        fs::write(gitignore_path, content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write .gitignore: {}", e))
        })?;

        Ok(())
    }

    fn exists(&self) -> Result<bool, ApplicationError> {
        Ok(self.path_manager.kiro_dir(true).exists())
    }

    fn load_steering_files(
        &self,
        config: &SteeringConfig,
    ) -> Result<Vec<Steering>, ApplicationError> {
        let mut steerings = Vec::new();
        let steering_dir = self.steering_dir();

        for steering_type in &config.types {
            let file_path = steering_dir.join(format!("{}.md", steering_type.name));

            // Skip if file doesn't exist
            if !file_path.exists() {
                continue;
            }

            // Read file content
            let content = fs::read_to_string(&file_path).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to read steering file {}: {}",
                    steering_type.name, e
                ))
            })?;

            steerings.push(Steering {
                steering_type: steering_type.clone(),
                content,
            });
        }

        Ok(steerings)
    }
}
