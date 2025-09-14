use hail_mary::application::repositories::{
    ConfigRepositoryInterface, SpecRepositoryInterface,
    steering_repository::SteeringRepositoryInterface,
};
use hail_mary::application::use_cases::{create_feature, initialize_project};
use hail_mary::domain::entities::steering::Steerings;
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

    // Create a test spec
    create_feature(&spec_repo, "test-feature").unwrap();

    // Load config and steering
    let config = config_repo.load_config().unwrap();
    let steering_files = steering_repo.load_steering_files(&config.steering).unwrap();
    let steerings = Steerings(steering_files);

    // Get spec path
    let specs = spec_repo.list_spec_directories().unwrap();
    let spec_name = &specs[0].0;
    let spec_path = spec_repo.get_spec_path(spec_name).unwrap();

    // Create system prompt
    let system_prompt = SystemPrompt::new(spec_name, &spec_path, &steerings);
    let content = system_prompt.as_str();

    // Verify the system prompt contains steering information
    assert!(content.contains("<steering>"));
    assert!(content.contains("</steering>"));
    assert!(content.contains("name: product"));
    assert!(content.contains("name: tech"));
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

    // Create a test spec
    create_feature(&spec_repo, "test-feature").unwrap();

    // Load config and steering (now with no steering files)
    let config = config_repo.load_config().unwrap();
    let steering_files = steering_repo.load_steering_files(&config.steering).unwrap();
    let steerings = Steerings(steering_files);

    // Get spec path
    let specs = spec_repo.list_spec_directories().unwrap();
    let spec_name = &specs[0].0;
    let spec_path = spec_repo.get_spec_path(spec_name).unwrap();

    // Create system prompt
    let system_prompt = SystemPrompt::new(spec_name, &spec_path, &steerings);
    let content = system_prompt.as_str();

    // Verify the system prompt still has the steering section structure
    assert!(content.contains("<steering>"));
    assert!(content.contains("</steering>"));

    // But no actual steering content
    assert!(!content.contains("name: product"));
    assert!(!content.contains("name: tech"));
}

#[test]
fn test_steering_display_format() {
    use hail_mary::domain::entities::steering::{Criterion, Steering, SteeringType};

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
