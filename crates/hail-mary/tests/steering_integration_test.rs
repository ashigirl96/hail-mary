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

#[test]
fn test_steerings_display_format_with_individual_tags() {
    use hail_mary::domain::entities::steering::{Steering, SteeringType, Steerings};

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
