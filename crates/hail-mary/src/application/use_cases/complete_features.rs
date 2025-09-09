use crate::application::errors::ApplicationError;
use crate::application::repositories::SpecRepositoryInterface;

pub fn complete_features(
    spec_repo: &dyn SpecRepositoryInterface,
    spec_names: &[String],
) -> Result<(), ApplicationError> {
    for name in spec_names {
        spec_repo.mark_spec_complete(name)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::{MockSpecRepository, TestDirectory};
    use crate::infrastructure::filesystem::path_manager::PathManager;
    use crate::infrastructure::repositories::spec::SpecRepository as ConcreteSpecRepository;
    use std::fs;

    #[test]
    fn test_complete_features_success() {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let spec_repo = ConcreteSpecRepository::new(path_manager.clone());

        // Create some spec directories
        let specs_dir = path_manager.specs_dir(true);
        fs::create_dir_all(&specs_dir).unwrap();
        fs::create_dir_all(specs_dir.join("2025-01-01-feature-a")).unwrap();
        fs::create_dir_all(specs_dir.join("2025-01-02-feature-b")).unwrap();

        // Complete one feature
        let result = complete_features(&spec_repo, &["2025-01-01-feature-a".to_string()]);
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
        let spec_repo = ConcreteSpecRepository::new(path_manager.clone());

        // Create some spec directories
        let specs_dir = path_manager.specs_dir(true);
        fs::create_dir_all(&specs_dir).unwrap();
        fs::create_dir_all(specs_dir.join("2025-01-01-feature-a")).unwrap();
        fs::create_dir_all(specs_dir.join("2025-01-02-feature-b")).unwrap();

        // Complete multiple features
        let result = complete_features(
            &spec_repo,
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
        let spec_repo = ConcreteSpecRepository::new(path_manager);

        // Try to complete non-existent feature (no need to initialize)
        let result = complete_features(&spec_repo, &["non-existent".to_string()]);
        assert!(result.is_err());
        match result.unwrap_err() {
            ApplicationError::SpecNotFound(name) => assert_eq!(name, "non-existent"),
            _ => panic!("Expected SpecNotFound error"),
        }
    }

    #[test]
    fn test_complete_features_with_mock() {
        let mock_repo = MockSpecRepository::with_specs(vec![
            "2025-01-01-feature-a".to_string(),
            "2025-01-02-feature-b".to_string(),
        ]);

        let result = complete_features(&mock_repo, &["2025-01-01-feature-a".to_string()]);
        assert!(result.is_ok());

        // Verify spec is in archived list
        let archived = mock_repo.list_archived_specs().unwrap();
        assert!(archived.contains(&"2025-01-01-feature-a".to_string()));
    }

    #[test]
    fn test_complete_features_already_archived() {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let spec_repo = ConcreteSpecRepository::new(path_manager.clone());

        // Create spec and archive directories
        let specs_dir = path_manager.specs_dir(true);
        let archive_dir = path_manager.archive_dir(true);
        fs::create_dir_all(&archive_dir).unwrap();
        fs::create_dir_all(specs_dir.join("2025-01-01-feature-a")).unwrap();

        // Create a file in the spec to verify it gets moved
        fs::write(
            specs_dir.join("2025-01-01-feature-a").join("new.txt"),
            "new content",
        )
        .unwrap();

        // Create existing archive with different content
        fs::create_dir_all(archive_dir.join("2025-01-01-feature-a")).unwrap();
        fs::write(
            archive_dir.join("2025-01-01-feature-a").join("old.txt"),
            "old content",
        )
        .unwrap();

        // Complete feature that already exists in archive - should overwrite
        let result = complete_features(&spec_repo, &["2025-01-01-feature-a".to_string()]);
        assert!(result.is_ok());

        // Verify the new spec replaced the old archive
        assert!(!specs_dir.join("2025-01-01-feature-a").exists());
        assert!(archive_dir.join("2025-01-01-feature-a").exists());
        assert!(
            archive_dir
                .join("2025-01-01-feature-a")
                .join("new.txt")
                .exists()
        );
        assert!(
            !archive_dir
                .join("2025-01-01-feature-a")
                .join("old.txt")
                .exists()
        );

        // Verify new content
        let new_content =
            fs::read_to_string(archive_dir.join("2025-01-01-feature-a").join("new.txt")).unwrap();
        assert_eq!(new_content, "new content");
    }
}
