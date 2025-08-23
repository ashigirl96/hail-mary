use crate::application::errors::ApplicationError;
use crate::application::repositories::ProjectRepository;

pub fn complete_features(
    project_repo: &dyn ProjectRepository,
    spec_names: &[String],
) -> Result<(), ApplicationError> {
    for name in spec_names {
        project_repo.mark_spec_complete(name)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::TestDirectory;
    use crate::infrastructure::filesystem::path_manager::PathManager;
    use crate::infrastructure::repositories::project::ProjectRepository as ConcreteProjectRepository;
    use std::fs;

    #[test]
    fn test_complete_features_success() {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let project_repo = ConcreteProjectRepository::new(path_manager.clone());

        // Initialize project
        project_repo.initialize().unwrap();

        // Create some spec directories
        let specs_dir = path_manager.specs_dir(true);
        fs::create_dir_all(specs_dir.join("2025-01-01-feature-a")).unwrap();
        fs::create_dir_all(specs_dir.join("2025-01-02-feature-b")).unwrap();

        // Complete one feature
        let result = complete_features(&project_repo, &["2025-01-01-feature-a".to_string()]);
        assert!(result.is_ok());

        // Verify it was moved to archive
        let archive_dir = path_manager.archive_dir(true);
        assert!(archive_dir.join("2025-01-01-feature-a").exists());
        assert!(!specs_dir.join("2025-01-01-feature-a").exists());
        assert!(specs_dir.join("2025-01-02-feature-b").exists());
    }

    #[test]
    fn test_complete_features_multiple() {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let project_repo = ConcreteProjectRepository::new(path_manager.clone());

        // Initialize project
        project_repo.initialize().unwrap();

        // Create some spec directories
        let specs_dir = path_manager.specs_dir(true);
        fs::create_dir_all(specs_dir.join("2025-01-01-feature-a")).unwrap();
        fs::create_dir_all(specs_dir.join("2025-01-02-feature-b")).unwrap();

        // Complete multiple features
        let result = complete_features(
            &project_repo,
            &[
                "2025-01-01-feature-a".to_string(),
                "2025-01-02-feature-b".to_string(),
            ],
        );
        assert!(result.is_ok());

        // Verify both were moved to archive
        let archive_dir = path_manager.archive_dir(true);
        assert!(archive_dir.join("2025-01-01-feature-a").exists());
        assert!(archive_dir.join("2025-01-02-feature-b").exists());
        assert!(!specs_dir.join("2025-01-01-feature-a").exists());
        assert!(!specs_dir.join("2025-01-02-feature-b").exists());
    }

    #[test]
    fn test_complete_features_spec_not_found() {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let project_repo = ConcreteProjectRepository::new(path_manager);

        // Initialize project
        project_repo.initialize().unwrap();

        // Try to complete non-existent feature
        let result = complete_features(&project_repo, &["non-existent".to_string()]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::SpecNotFound(name) => assert_eq!(name, "non-existent"),
            _ => panic!("Expected SpecNotFound error"),
        }
    }

    #[test]
    fn test_complete_features_already_archived() {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let project_repo = ConcreteProjectRepository::new(path_manager.clone());

        // Initialize project
        project_repo.initialize().unwrap();

        // Create spec and archive directories
        let specs_dir = path_manager.specs_dir(true);
        let archive_dir = path_manager.archive_dir(true);
        fs::create_dir_all(&archive_dir).unwrap();
        fs::create_dir_all(specs_dir.join("2025-01-01-feature-a")).unwrap();
        fs::create_dir_all(archive_dir.join("2025-01-01-feature-a")).unwrap();

        // Try to complete feature that already exists in archive
        let result = complete_features(&project_repo, &["2025-01-01-feature-a".to_string()]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::ArchiveAlreadyExists(name) => {
                assert_eq!(name, "2025-01-01-feature-a")
            }
            _ => panic!("Expected ArchiveAlreadyExists error"),
        }
    }
}
