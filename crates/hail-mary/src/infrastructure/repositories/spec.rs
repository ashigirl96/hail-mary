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

    fn validate_spec_name(&self, name: &str) -> Result<(), ApplicationError> {
        // Validate spec name (kebab-case)
        if !name
            .chars()
            .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
            || name.starts_with('-')
            || name.ends_with('-')
            || name.contains("--")
        {
            return Err(ApplicationError::InvalidSpecName(name.to_string()));
        }
        Ok(())
    }

    fn create_template_files(
        &self,
        spec_dir: &std::path::Path,
        name: &str,
        lang: &str,
    ) -> Result<(), ApplicationError> {
        // Create essential template files including tasks.md for orchestration

        // TODO: Commenting out design.md and investigation.md generation for now
        // until we determine the final approach
        // let design_content = format!(
        //     r#"# Design
        //
        // ## Overview
        // [High-level architecture for {}]
        //
        // "#,
        //     name
        // );

        let memo_content = format!(
            r#"# Memo: {}

"#,
            name
        );

        // let investigation_content = format!(
        //     r#"# Investigation: {}
        //
        // "#,
        //     name
        // );

        // Create tasks.md with initial State Tracking and Timeline
        let tasks_content = format!(
            r#"# Tasks

**Language**: {}

## State Tracking

| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | pending | - | Define requirements |
| tasks.md#Timeline | pending | 0% | Plan implementation order |

## Timeline

- [x] Spec created → {}
- [ ] Requirements definition
- [ ] Implementation planning
"#,
            lang, name
        );

        // Write all template files including tasks.md
        // TODO: Commented out design.md and investigation.md creation
        // fs::write(spec_dir.join("design.md"), design_content).map_err(|e| {
        //     ApplicationError::FileSystemError(format!("Failed to write design.md: {}", e))
        // })?;

        fs::write(spec_dir.join("memo.md"), memo_content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write memo.md: {}", e))
        })?;

        // fs::write(spec_dir.join("investigation.md"), investigation_content).map_err(|e| {
        //     ApplicationError::FileSystemError(format!("Failed to write investigation.md: {}", e))
        // })?;

        fs::write(spec_dir.join("tasks.md"), tasks_content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write tasks.md: {}", e))
        })?;

        Ok(())
    }
}

impl SpecRepositoryInterface for SpecRepository {
    fn create_spec(&self, name: &str, lang: &str) -> Result<(), ApplicationError> {
        // Validate spec name
        self.validate_spec_name(name)?;

        // Ensure specs directory exists (like ConfigRepository pattern)
        let specs_dir = self.path_manager.specs_dir(true);
        fs::create_dir_all(&specs_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create specs directory: {}", e))
        })?;

        // Generate directory name with date prefix
        let date = chrono::Utc::now().format("%Y-%m-%d");
        let spec_name = format!("{}-{}", date, name);
        let spec_dir = specs_dir.join(&spec_name);

        // Check if spec already exists
        if spec_dir.exists() {
            return Err(ApplicationError::SpecAlreadyExists(name.to_string()));
        }

        // Create spec directory
        fs::create_dir_all(&spec_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create spec directory: {}", e))
        })?;

        // Create template files
        self.create_template_files(&spec_dir, name, lang)?;

        Ok(())
    }

    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError> {
        let specs_dir = self.path_manager.specs_dir(true);
        let mut specs = Vec::new();

        if !specs_dir.exists() {
            return Ok(specs);
        }

        let entries = fs::read_dir(specs_dir).map_err(|e| {
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
                // For now, all specs in the specs directory are considered not archived
                specs.push((name, false));
            }
        }

        // Sort by name in reverse order (newer dates first)
        specs.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(specs)
    }

    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError> {
        let source_path = self.path_manager.specs_dir(true).join(name);

        if !source_path.exists() {
            return Err(ApplicationError::SpecNotFound(name.to_string()));
        }

        if !source_path.is_dir() {
            return Err(ApplicationError::InvalidSpecDirectory(name.to_string()));
        }

        // Create archive directory
        let archive_dir = self.path_manager.archive_dir(true);
        fs::create_dir_all(&archive_dir).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create archive directory: {}", e))
        })?;

        let dest_path = archive_dir.join(name);

        // If already exists in archive, remove it first to allow overwriting
        if dest_path.exists() {
            fs::remove_dir_all(&dest_path).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to remove existing archive: {}",
                    e
                ))
            })?;
        }

        // Move directory to archive
        fs::rename(&source_path, &dest_path).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to move spec to archive: {}", e))
        })?;

        Ok(())
    }

    fn get_spec_path(&self, name: &str) -> Result<PathBuf, ApplicationError> {
        let specs_dir = self.path_manager.specs_dir(true);
        let spec_path = specs_dir.join(name);

        if !spec_path.exists() {
            return Err(ApplicationError::SpecNotFound(name.to_string()));
        }

        if !spec_path.is_dir() {
            return Err(ApplicationError::InvalidSpecDirectory(name.to_string()));
        }

        Ok(spec_path)
    }

    fn list_archived_specs(&self) -> Result<Vec<String>, ApplicationError> {
        let archive_dir = self.path_manager.archive_dir(true);
        let mut specs = Vec::new();

        if !archive_dir.exists() {
            return Ok(specs);
        }

        let entries = fs::read_dir(archive_dir).map_err(|e| {
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

        // Sort by name in reverse order (newer dates first)
        specs.sort_by(|a, b| b.cmp(a));
        Ok(specs)
    }

    fn is_pbi(&self, spec_name: &str) -> Result<bool, ApplicationError> {
        let sbis = self.list_sbis(spec_name)?;
        Ok(!sbis.is_empty())
    }

    fn list_sbis(&self, pbi_name: &str) -> Result<Vec<String>, ApplicationError> {
        let pbi_path = self.path_manager.specs_dir(true).join(pbi_name);

        if !pbi_path.exists() {
            return Ok(Vec::new());
        }

        let mut sbis = Vec::new();

        let entries = fs::read_dir(&pbi_path).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read PBI directory: {}", e))
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
                if name.starts_with("sbi-") {
                    sbis.push(name);
                }
            }
        }

        sbis.sort();
        Ok(sbis)
    }

    fn create_sbi(
        &self,
        pbi_name: &str,
        sbi_name: &str,
        lang: &str,
    ) -> Result<(), ApplicationError> {
        // Validate SBI name
        self.validate_spec_name(sbi_name)?;

        let sbi_path = self
            .path_manager
            .specs_dir(true)
            .join(pbi_name)
            .join(sbi_name);

        // Check if SBI already exists
        if sbi_path.exists() {
            return Err(ApplicationError::SpecAlreadyExists(sbi_name.to_string()));
        }

        // Create SBI directory
        fs::create_dir_all(&sbi_path).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to create SBI directory: {}", e))
        })?;

        // Reuse create_template_files to generate tasks.md and memo.md
        // Note: requirements.md is NOT generated here - created by /decompose or /add-sbi slash commands
        self.create_template_files(&sbi_path, sbi_name, lang)?;

        Ok(())
    }

    fn ensure_sbi_files(
        &self,
        pbi_name: &str,
        sbi_name: &str,
        lang: &str,
    ) -> Result<(), ApplicationError> {
        let sbi_path = self
            .path_manager
            .specs_dir(true)
            .join(pbi_name)
            .join(sbi_name);

        // Check if SBI directory exists
        if !sbi_path.exists() {
            return Err(ApplicationError::SpecNotFound(sbi_name.to_string()));
        }

        let tasks_path = sbi_path.join("tasks.md");
        let memo_path = sbi_path.join("memo.md");

        // Only generate missing files
        if !tasks_path.exists() || !memo_path.exists() {
            self.create_template_files(&sbi_path, sbi_name, lang)?;
        }

        Ok(())
    }
}
