use crate::application::errors::ApplicationError;
use crate::application::repositories::SpecRepositoryInterface;
use crate::infrastructure::filesystem::path_manager::PathManager;
use std::fs;
use std::path::PathBuf;

pub struct SpecRepository {
    path_manager: PathManager,
}

impl SpecRepository {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }

    fn create_template_files(
        &self,
        spec_dir: &std::path::Path,
        name: &str,
    ) -> Result<(), ApplicationError> {
        let memo_content = format!(
            r#"# Memo: {}

"#,
            name
        );

        fs::write(spec_dir.join("memo.md"), memo_content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write memo.md: {}", e))
        })?;

        Ok(())
    }
}

impl SpecRepositoryInterface for SpecRepository {
    fn create_spec(&self, name: &str) -> Result<(), ApplicationError> {
        let specs_dir = self.path_manager.specs_dir();
        fs::create_dir_all(&specs_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create specs directory: {}", e))
        })?;

        let date = chrono::Utc::now().format("%Y-%m-%d");
        let spec_name = format!("{}-{}", date, name);
        let spec_dir = specs_dir.join(&spec_name);

        if spec_dir.exists() {
            return Err(ApplicationError::SpecAlreadyExists(name.to_string()));
        }

        fs::create_dir_all(&spec_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create spec directory: {}", e))
        })?;

        self.create_template_files(&spec_dir, name)?;

        Ok(())
    }

    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError> {
        let specs_dir = self.path_manager.specs_dir();
        let mut specs = Vec::new();

        if !specs_dir.exists() {
            return Ok(specs);
        }

        let entries = fs::read_dir(&specs_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read specs directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to read directory entry: {}", e))
            })?;

            let file_type = entry.file_type().map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to get file type: {}", e))
            })?;

            if file_type.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name != "archive" {
                    specs.push((name, false));
                }
            }
        }

        specs.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(specs)
    }

    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError> {
        let source_path = self.path_manager.specs_dir().join(name);

        if !source_path.exists() {
            return Err(ApplicationError::SpecNotFound(name.to_string()));
        }

        if !source_path.is_dir() {
            return Err(ApplicationError::InvalidSpecDirectory(name.to_string()));
        }

        let archive_dir = self.path_manager.archive_dir();
        fs::create_dir_all(&archive_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create archive directory: {}", e))
        })?;

        let dest_path = archive_dir.join(name);

        if dest_path.exists() {
            fs::remove_dir_all(&dest_path).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to remove existing archive: {}",
                    e
                ))
            })?;
        }

        fs::rename(&source_path, &dest_path).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to move spec to archive: {}", e))
        })?;

        Ok(())
    }

    fn get_spec_path(&self, name: &str) -> Result<PathBuf, ApplicationError> {
        let spec_path = self.path_manager.specs_dir().join(name);

        if !spec_path.exists() {
            return Err(ApplicationError::SpecNotFound(name.to_string()));
        }

        if !spec_path.is_dir() {
            return Err(ApplicationError::InvalidSpecDirectory(name.to_string()));
        }

        Ok(spec_path)
    }

    fn list_archived_specs(&self) -> Result<Vec<String>, ApplicationError> {
        let archive_dir = self.path_manager.archive_dir();
        let mut specs = Vec::new();

        if !archive_dir.exists() {
            return Ok(specs);
        }

        let entries = fs::read_dir(&archive_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read archive directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to read directory entry: {}", e))
            })?;

            let file_type = entry.file_type().map_err(|e| {
                ApplicationError::FileSystemError(format!("Failed to get file type: {}", e))
            })?;

            if file_type.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                specs.push(name);
            }
        }

        specs.sort_by(|a, b| b.cmp(a));
        Ok(specs)
    }
}
