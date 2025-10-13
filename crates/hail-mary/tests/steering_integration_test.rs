use hail_mary::application::repositories::{
    ConfigRepositoryInterface, SpecRepositoryInterface,
    steering_repository::SteeringRepositoryInterface,
};
use hail_mary::application::use_cases::{backup_steering, initialize_project};
use hail_mary::domain::value_objects::steering::Steerings;
use hail_mary::domain::value_objects::system_prompt::SystemPrompt;
use hail_mary::infrastructure::filesystem::path_manager::PathManager;
use hail_mary::infrastructure::repositories::{
    config::ConfigRepository, spec::SpecRepository, steering::SteeringRepository,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_system_prompt_includes_steering_content() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Initialize project with steering
    let path_manager = PathManager::new(temp_path.to_path_buf());
    let config_repo = ConfigRepository::new(path_manager.clone());
    let steering_repo = SteeringRepository::new(path_manager.clone());
    let spec_repo = SpecRepository::new(path_manager.clone());

    // Initialize the project (creates config and steering)
    initialize_project(&config_repo, &spec_repo, &steering_repo).unwrap();

    // Create some steering content
    let steering_dir = temp_path.join(".kiro").join("steering");
    fs::write(
        steering_dir.join("product.md"),
        "# Product Overview\n\nThis is a test product with amazing features.",
    )
    .unwrap();
    fs::write(
        steering_dir.join("tech.md"),
        "# Technical Stack\n\nWe use Rust for everything.",
    )
    .unwrap();

    // Create a test spec directly via repository
    spec_repo.create_spec("test-feature", "en").unwrap();

    // Load config and steering
    let steering_config = config_repo.load_steering_config().unwrap();
    let steering_files = steering_repo.load_steering_files(&steering_config).unwrap();
    let steerings = Steerings(steering_files);

    // Get spec path
    let specs = spec_repo.list_spec_directories().unwrap();
    let spec_name = &specs[0].0;
    let spec_path = spec_repo.get_spec_path(spec_name).unwrap();

    // Create system prompt
    let system_prompt = SystemPrompt::new(Some(spec_name.as_str()), Some(&spec_path), &steerings);
    let content = system_prompt.as_str();

    // Verify the system prompt contains steering information with individual tags
    assert!(content.contains("<steering-product>"));
    assert!(content.contains("</steering-product>"));
    assert!(content.contains("<steering-tech>"));
    assert!(content.contains("</steering-tech>"));
    assert!(content.contains("This is a test product with amazing features"));
    assert!(content.contains("We use Rust for everything"));
}

#[test]
fn test_system_prompt_with_empty_steering() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Initialize project
    let path_manager = PathManager::new(temp_path.to_path_buf());
    let config_repo = ConfigRepository::new(path_manager.clone());
    let steering_repo = SteeringRepository::new(path_manager.clone());
    let spec_repo = SpecRepository::new(path_manager.clone());

    // Initialize the project (creates config and steering)
    initialize_project(&config_repo, &spec_repo, &steering_repo).unwrap();

    // Delete the default steering files that were created
    let steering_dir = temp_path.join(".kiro").join("steering");
    if let Ok(entries) = fs::read_dir(&steering_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    // Create a test spec directly via repository
    spec_repo.create_spec("test-feature", "en").unwrap();

    // Load config and steering (now with no steering files)
    let steering_config = config_repo.load_steering_config().unwrap();
    let steering_files = steering_repo.load_steering_files(&steering_config).unwrap();
    let steerings = Steerings(steering_files);

    // Get spec path
    let specs = spec_repo.list_spec_directories().unwrap();
    let spec_name = &specs[0].0;
    let spec_path = spec_repo.get_spec_path(spec_name).unwrap();

    // Create system prompt
    let system_prompt = SystemPrompt::new(Some(spec_name.as_str()), Some(&spec_path), &steerings);
    let content = system_prompt.as_str();

    // When there are no steering files, the steering vector should be empty
    assert_eq!(steerings.0.len(), 0, "Steerings should be empty");

    // The template text is still present
    assert!(content.contains("steering tags below")); // Template text is still there

    // But no actual steering content tags should be generated when steerings is empty
    // Note: The template contains example references like "`<steering-product>`" but not actual tags
    assert!(!content.contains("<steering-product>\n")); // Actual tag would have newline
    assert!(!content.contains("<steering-tech>\n")); // Actual tag would have newline
    assert!(!content.contains("</steering-product>")); // Actual closing tag
    assert!(!content.contains("</steering-tech>")); // Actual closing tag
}

#[test]
fn test_steering_display_format() {
    use hail_mary::domain::value_objects::steering::{Criterion, Steering, SteeringType};

    // Create test steering data
    let steering = Steering {
        steering_type: SteeringType {
            name: "test".to_string(),
            purpose: "Test purpose".to_string(),
            criteria: vec![
                Criterion {
                    name: "Criterion 1".to_string(),
                    description: "Description 1".to_string(),
                },
                Criterion {
                    name: "Criterion 2".to_string(),
                    description: "Description 2".to_string(),
                },
            ],
            allowed_operations: vec!["refresh".to_string()],
        },
        content: "Test content here".to_string(),
    };

    let formatted = steering.to_string();

    // Verify the format matches expected structure
    assert!(formatted.contains("name: test"));
    assert!(formatted.contains("criteria:"));
    assert!(formatted.contains("- Criterion 1: Description 1"));
    assert!(formatted.contains("- Criterion 2: Description 2"));
    assert!(formatted.contains("content:\nTest content here"));
}

#[test]
fn test_steerings_display_format_with_individual_tags() {
    use hail_mary::domain::value_objects::steering::{Steering, SteeringType, Steerings};

    // Create test steering data
    let product_steering = Steering {
        steering_type: SteeringType {
            name: "product".to_string(),
            purpose: "Product purpose".to_string(),
            criteria: vec![],
            allowed_operations: vec![],
        },
        content: "Product content".to_string(),
    };

    let tech_steering = Steering {
        steering_type: SteeringType {
            name: "tech".to_string(),
            purpose: "Tech purpose".to_string(),
            criteria: vec![],
            allowed_operations: vec![],
        },
        content: "Tech content".to_string(),
    };

    let steerings = Steerings(vec![product_steering, tech_steering]);
    let formatted = steerings.to_string();

    // Verify each steering has its own tag
    assert!(formatted.contains("<steering-product>"));
    assert!(formatted.contains("Product content"));
    assert!(formatted.contains("</steering-product>"));

    assert!(formatted.contains("<steering-tech>"));
    assert!(formatted.contains("Tech content"));
    assert!(formatted.contains("</steering-tech>"));

    // Verify the old single steering tag is NOT present
    assert!(!formatted.contains("<steering>\n"));
    assert!(!formatted.contains("-----"));
}

#[test]
fn test_backup_rotation_maintains_max_limit() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Initialize project
    let path_manager = PathManager::new(temp_path.to_path_buf());
    let config_repo = ConfigRepository::new(path_manager.clone());
    let steering_repo = SteeringRepository::new(path_manager.clone());
    let spec_repo = SpecRepository::new(path_manager.clone());

    // Initialize the project
    initialize_project(&config_repo, &spec_repo, &steering_repo).unwrap();

    // Update config with max = 3 for testing
    let config_path = temp_path.join(".kiro").join("config.toml");
    let config_content = fs::read_to_string(&config_path).unwrap();
    let updated_config =
        config_content.replace("[steering.backup]\nmax = 10", "[steering.backup]\nmax = 3");
    fs::write(config_path, updated_config).unwrap();

    // Create some steering files to backup
    let steering_dir = temp_path.join(".kiro").join("steering");
    fs::write(steering_dir.join("product.md"), "Product content").unwrap();
    fs::write(steering_dir.join("tech.md"), "Tech content").unwrap();

    // Create 3 existing backups manually
    let backup_dir = steering_dir.join("backup");
    for i in 1..=3 {
        let backup_name = format!("2024-01-0{}-12-00", i);
        let backup_path = backup_dir.join(&backup_name);
        fs::create_dir_all(&backup_path).unwrap();
        fs::write(backup_path.join("product.md"), "Old product").unwrap();
        fs::write(backup_path.join("tech.md"), "Old tech").unwrap();
    }

    // Verify initial state: 3 backups exist
    let initial_backups = steering_repo.list_steering_backups().unwrap();
    assert_eq!(initial_backups.len(), 3, "Should start with 3 backups");

    // Run backup_steering - should delete oldest and create new one
    let result = backup_steering(&config_repo, &steering_repo).unwrap();
    assert!(result.contains("Created backup"));

    // Verify we still have exactly 3 backups (not 4)
    let final_backups = steering_repo.list_steering_backups().unwrap();
    assert_eq!(final_backups.len(), 3, "Should maintain max=3 after backup");

    // Verify the oldest backup (2024-01-01) was deleted
    let backup_names: Vec<String> = final_backups.iter().map(|b| b.name.clone()).collect();
    assert!(
        !backup_names.contains(&"2024-01-01-12-00".to_string()),
        "Oldest backup should be deleted"
    );

    // Test is complete - we've verified:
    // 1. Starting with 3 backups at max limit
    // 2. Creating new backup deletes oldest and maintains max=3
    // 3. Oldest backup was properly removed

    // Note: We don't test second backup here since without seconds in timestamp,
    // it could create same-named backup in same minute and cause issues
}

#[test]
fn test_backup_rotation_with_excess_backups() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Initialize project
    let path_manager = PathManager::new(temp_path.to_path_buf());
    let config_repo = ConfigRepository::new(path_manager.clone());
    let steering_repo = SteeringRepository::new(path_manager.clone());
    let spec_repo = SpecRepository::new(path_manager.clone());

    // Initialize the project
    initialize_project(&config_repo, &spec_repo, &steering_repo).unwrap();

    // Update config with max = 2 for testing
    let config_path = temp_path.join(".kiro").join("config.toml");
    let config_content = fs::read_to_string(&config_path).unwrap();
    let updated_config =
        config_content.replace("[steering.backup]\nmax = 10", "[steering.backup]\nmax = 2");
    fs::write(config_path, updated_config).unwrap();

    // Create a steering file
    let steering_dir = temp_path.join(".kiro").join("steering");
    fs::write(steering_dir.join("product.md"), "Product content").unwrap();

    // Create 5 existing backups (exceeding max=2)
    let backup_dir = steering_dir.join("backup");
    for i in 1..=5 {
        let backup_name = format!("2024-01-0{}-12-00", i);
        let backup_path = backup_dir.join(&backup_name);
        fs::create_dir_all(&backup_path).unwrap();
        fs::write(backup_path.join("product.md"), "Old content").unwrap();
    }

    // Verify initial state: 5 backups exist (exceeding max)
    let initial_backups = steering_repo.list_steering_backups().unwrap();
    assert_eq!(initial_backups.len(), 5, "Should start with 5 backups");

    // Run backup_steering - should delete 4 oldest and create new one
    let result = backup_steering(&config_repo, &steering_repo).unwrap();
    assert!(result.contains("Created backup"));

    // Verify we have exactly 2 backups (not 3 or 6)
    let final_backups = steering_repo.list_steering_backups().unwrap();
    assert_eq!(
        final_backups.len(),
        2,
        "Should have exactly max=2 after backup"
    );

    // Verify the 4 oldest backups were deleted
    let backup_names: Vec<String> = final_backups.iter().map(|b| b.name.clone()).collect();
    assert!(
        !backup_names.contains(&"2024-01-01-12-00".to_string()),
        "Backup 1 should be deleted"
    );
    assert!(
        !backup_names.contains(&"2024-01-02-12-00".to_string()),
        "Backup 2 should be deleted"
    );
    assert!(
        !backup_names.contains(&"2024-01-03-12-00".to_string()),
        "Backup 3 should be deleted"
    );
    assert!(
        !backup_names.contains(&"2024-01-04-12-00".to_string()),
        "Backup 4 should be deleted"
    );
}
