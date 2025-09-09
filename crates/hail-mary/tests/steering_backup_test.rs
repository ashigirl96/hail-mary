use hail_mary::application::repositories::ProjectRepository;
use hail_mary::infrastructure::filesystem::path_manager::PathManager;
use hail_mary::infrastructure::repositories::project::ProjectRepository as ProjectRepositoryImpl;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_list_steering_files_returns_markdown_files() {
    let temp_dir = TempDir::new().unwrap();

    // Setup: Create .kiro/steering directory with markdown files
    let steering_dir = temp_dir.path().join(".kiro/steering");
    fs::create_dir_all(&steering_dir).unwrap();

    // Create markdown files
    fs::write(steering_dir.join("product.md"), "# Product").unwrap();
    fs::write(steering_dir.join("tech.md"), "# Tech").unwrap();
    fs::write(steering_dir.join("structure.md"), "# Structure").unwrap();

    // Create backup and draft directories (should be excluded)
    fs::create_dir_all(steering_dir.join("backup")).unwrap();
    fs::create_dir_all(steering_dir.join("draft")).unwrap();
    fs::write(steering_dir.join("backup/old.md"), "old backup").unwrap();
    fs::write(steering_dir.join("draft/draft.md"), "draft content").unwrap();

    // Create non-markdown file (should be excluded)
    fs::write(steering_dir.join("config.txt"), "config").unwrap();

    let path_manager = PathManager::new(temp_dir.path().to_path_buf());
    let repo = ProjectRepositoryImpl::new(path_manager);

    // Act
    let files = repo.list_steering_files().unwrap();

    // Assert
    assert_eq!(files.len(), 3);
    assert!(files.contains(&PathBuf::from("product.md")));
    assert!(files.contains(&PathBuf::from("tech.md")));
    assert!(files.contains(&PathBuf::from("structure.md")));
}

#[test]
fn test_create_steering_backup_creates_timestamped_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Setup: Create .kiro/steering directory with files
    let steering_dir = temp_dir.path().join(".kiro/steering");
    fs::create_dir_all(&steering_dir).unwrap();
    fs::write(steering_dir.join("product.md"), "# Product content").unwrap();
    fs::write(steering_dir.join("tech.md"), "# Tech content").unwrap();

    let path_manager = PathManager::new(temp_dir.path().to_path_buf());
    let repo = ProjectRepositoryImpl::new(path_manager);

    // Act
    let timestamp = "2025-01-09-10-30";
    let files = vec![PathBuf::from("product.md"), PathBuf::from("tech.md")];
    repo.create_steering_backup(timestamp, &files).unwrap();

    // Assert
    let backup_dir = steering_dir.join("backup").join(timestamp);
    assert!(backup_dir.exists());
    assert!(backup_dir.join("product.md").exists());
    assert!(backup_dir.join("tech.md").exists());

    // Verify content was copied correctly
    let product_content = fs::read_to_string(backup_dir.join("product.md")).unwrap();
    assert_eq!(product_content, "# Product content");

    let tech_content = fs::read_to_string(backup_dir.join("tech.md")).unwrap();
    assert_eq!(tech_content, "# Tech content");
}

#[test]
fn test_list_steering_backups_returns_sorted_by_creation_time() {
    let temp_dir = TempDir::new().unwrap();

    // Setup: Create backup directories
    let backup_dir = temp_dir.path().join(".kiro/steering/backup");
    fs::create_dir_all(&backup_dir).unwrap();

    // Create backups with different timestamps
    let backup1 = backup_dir.join("2025-01-01-10-00");
    let backup2 = backup_dir.join("2025-01-02-10-00");
    let backup3 = backup_dir.join("2025-01-03-10-00");

    fs::create_dir_all(&backup1).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    fs::create_dir_all(&backup2).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    fs::create_dir_all(&backup3).unwrap();

    let path_manager = PathManager::new(temp_dir.path().to_path_buf());
    let repo = ProjectRepositoryImpl::new(path_manager);

    // Act
    let backups = repo.list_steering_backups().unwrap();

    // Assert
    assert_eq!(backups.len(), 3);
    assert_eq!(backups[0].name, "2025-01-01-10-00");
    assert_eq!(backups[1].name, "2025-01-02-10-00");
    assert_eq!(backups[2].name, "2025-01-03-10-00");

    // Verify they're sorted by creation time (oldest first)
    assert!(backups[0].created_at <= backups[1].created_at);
    assert!(backups[1].created_at <= backups[2].created_at);
}

#[test]
fn test_delete_oldest_steering_backups_removes_correct_directories() {
    let temp_dir = TempDir::new().unwrap();

    // Setup: Create backup directories
    let backup_dir = temp_dir.path().join(".kiro/steering/backup");
    fs::create_dir_all(&backup_dir).unwrap();

    // Create 5 backups
    for i in 1..=5 {
        let backup = backup_dir.join(format!("2025-01-0{}-10-00", i));
        fs::create_dir_all(&backup).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let path_manager = PathManager::new(temp_dir.path().to_path_buf());
    let repo = ProjectRepositoryImpl::new(path_manager);

    // Act: Delete 2 oldest backups
    repo.delete_oldest_steering_backups(2).unwrap();

    // Assert
    assert!(!backup_dir.join("2025-01-01-10-00").exists());
    assert!(!backup_dir.join("2025-01-02-10-00").exists());
    assert!(backup_dir.join("2025-01-03-10-00").exists());
    assert!(backup_dir.join("2025-01-04-10-00").exists());
    assert!(backup_dir.join("2025-01-05-10-00").exists());

    // Verify list returns only 3 remaining
    let remaining = repo.list_steering_backups().unwrap();
    assert_eq!(remaining.len(), 3);
}

#[test]
fn test_load_steering_backup_config_from_toml() {
    let temp_dir = TempDir::new().unwrap();

    // Setup: Create config.toml with steering backup configuration
    let kiro_dir = temp_dir.path().join(".kiro");
    fs::create_dir_all(&kiro_dir).unwrap();

    let config_content = r#"
[[steering.types]]
name = "product"
purpose = "Product overview"
criteria = ["Overview: Description"]

[steering.backup]
max = 20
"#;
    fs::write(kiro_dir.join("config.toml"), config_content).unwrap();

    let path_manager = PathManager::new(temp_dir.path().to_path_buf());
    let repo = ProjectRepositoryImpl::new(path_manager);

    // Act
    let config = repo.load_steering_backup_config().unwrap();

    // Assert
    assert_eq!(config.max, 20);
}

#[test]
fn test_load_steering_backup_config_returns_default_when_missing() {
    let temp_dir = TempDir::new().unwrap();

    // Setup: Create config.toml without steering.backup section
    let kiro_dir = temp_dir.path().join(".kiro");
    fs::create_dir_all(&kiro_dir).unwrap();

    let config_content = r#"
[[steering.types]]
name = "product"
purpose = "Product overview"
criteria = ["Overview: Description"]
"#;
    fs::write(kiro_dir.join("config.toml"), config_content).unwrap();

    let path_manager = PathManager::new(temp_dir.path().to_path_buf());
    let repo = ProjectRepositoryImpl::new(path_manager);

    // Act
    let config = repo.load_steering_backup_config().unwrap();

    // Assert
    assert_eq!(config.max, 10); // Default value
}

#[test]
fn test_ensure_steering_backup_config_adds_missing_section() {
    let temp_dir = TempDir::new().unwrap();

    // Setup: Create config.toml without steering.backup section
    let kiro_dir = temp_dir.path().join(".kiro");
    fs::create_dir_all(&kiro_dir).unwrap();

    let config_content = r#"
[[steering.types]]
name = "product"
purpose = "Product overview"
criteria = ["Overview: Description"]
"#;
    fs::write(kiro_dir.join("config.toml"), config_content).unwrap();

    let path_manager = PathManager::new(temp_dir.path().to_path_buf());
    let repo = ProjectRepositoryImpl::new(path_manager);

    // Act
    repo.ensure_steering_backup_config().unwrap();

    // Assert: Read config and verify backup section was added
    let updated_content = fs::read_to_string(kiro_dir.join("config.toml")).unwrap();
    assert!(updated_content.contains("[steering.backup]"));
    assert!(updated_content.contains("max = 10"));

    // Verify it can be loaded correctly
    let config = repo.load_steering_backup_config().unwrap();
    assert_eq!(config.max, 10);
}

#[test]
fn test_ensure_steering_backup_config_preserves_existing_value() {
    let temp_dir = TempDir::new().unwrap();

    // Setup: Create config.toml with existing steering.backup section
    let kiro_dir = temp_dir.path().join(".kiro");
    fs::create_dir_all(&kiro_dir).unwrap();

    let config_content = r#"
[[steering.types]]
name = "product"
purpose = "Product overview"
criteria = ["Overview: Description"]

[steering.backup]
max = 25
"#;
    fs::write(kiro_dir.join("config.toml"), config_content).unwrap();

    let path_manager = PathManager::new(temp_dir.path().to_path_buf());
    let repo = ProjectRepositoryImpl::new(path_manager);

    // Act
    repo.ensure_steering_backup_config().unwrap();

    // Assert: Verify existing value is preserved
    let config = repo.load_steering_backup_config().unwrap();
    assert_eq!(config.max, 25);
}
