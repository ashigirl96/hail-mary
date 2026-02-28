use crate::application::errors::ApplicationError;
use std::fs;
use std::path::Path;

pub fn initialize_project(project_root: &Path) -> Result<(), ApplicationError> {
    let specs_dir = project_root.join(".specs");
    let archive_dir = specs_dir.join("archive");

    // Create .specs/ and .specs/archive/ (idempotent)
    fs::create_dir_all(&archive_dir).map_err(|e| {
        ApplicationError::ProjectInitializationError(format!(
            "Failed to create .specs directories: {}",
            e
        ))
    })?;

    // Auto-migration: .kiro/specs/ → .specs/
    let old_specs_dir = project_root.join(".kiro").join("specs");
    if old_specs_dir.exists() && is_dir_empty_or_nonexistent(&specs_dir, &["archive"]) {
        migrate_directory_contents(&old_specs_dir, &specs_dir)?;
    }

    // Auto-migration: .kiro/archive/ → .specs/archive/
    let old_archive_dir = project_root.join(".kiro").join("archive");
    if old_archive_dir.exists() && is_dir_empty(&archive_dir) {
        migrate_directory_contents(&old_archive_dir, &archive_dir)?;
    }

    Ok(())
}

fn is_dir_empty(dir: &Path) -> bool {
    if !dir.exists() {
        return true;
    }
    match fs::read_dir(dir) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => true,
    }
}

fn is_dir_empty_or_nonexistent(dir: &Path, ignore: &[&str]) -> bool {
    if !dir.exists() {
        return true;
    }
    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !ignore.contains(&name.as_str()) {
                    return false;
                }
            }
            true
        }
        Err(_) => true,
    }
}

fn migrate_directory_contents(src: &Path, dst: &Path) -> Result<(), ApplicationError> {
    let entries = fs::read_dir(src).map_err(|e| {
        ApplicationError::ProjectInitializationError(format!(
            "Failed to read migration source {}: {}",
            src.display(),
            e
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            ApplicationError::ProjectInitializationError(format!(
                "Failed to read directory entry: {}",
                e
            ))
        })?;
        let dest_path = dst.join(entry.file_name());
        if !dest_path.exists() {
            fs::rename(entry.path(), &dest_path).map_err(|e| {
                ApplicationError::ProjectInitializationError(format!(
                    "Failed to migrate {}: {}",
                    entry.path().display(),
                    e
                ))
            })?;
        }
    }

    Ok(())
}
