use std::env;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to run hail-mary command in a test environment
fn run_hail_mary_command(
    args: &[&str],
    working_dir: &str,
) -> Result<std::process::Output, std::io::Error> {
    // Get the project root directory
    let project_root = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    // Path to the built binary
    let binary_path = project_root.join("target/debug/hail-mary");

    Command::new(binary_path)
        .args(args)
        .current_dir(working_dir)
        .output()
}

#[test]
fn test_new_command_e2e_success() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Run: hail-mary new test-feature
    let output = run_hail_mary_command(&["new", "test-feature"], temp_path).unwrap();

    // Check command succeeded
    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check output contains success message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Feature 'test-feature' created successfully!"));
    assert!(stdout.contains("Location:"));
    assert!(stdout.contains("Files created:"));
    assert!(stdout.contains("requirements.md"));
    assert!(stdout.contains("design.md"));
    assert!(stdout.contains("task.md"));
    assert!(stdout.contains("spec.json"));

    // Check that .kiro/specs directory was created
    let specs_dir = temp_dir.path().join(".kiro/specs");
    assert!(specs_dir.exists(), ".kiro/specs directory should exist");

    // Find the created feature directory
    let entries = std::fs::read_dir(&specs_dir).unwrap();
    let mut feature_dir_found = false;

    for entry in entries {
        let entry = entry.unwrap();
        let dir_name = entry.file_name().to_str().unwrap().to_string();

        if dir_name.ends_with("-test-feature") {
            feature_dir_found = true;
            let feature_path = entry.path();

            // Check all required files exist
            assert!(feature_path.join("requirements.md").exists());
            assert!(feature_path.join("design.md").exists());
            assert!(feature_path.join("task.md").exists());
            assert!(feature_path.join("spec.json").exists());

            // Check file contents
            let spec_json_content =
                std::fs::read_to_string(feature_path.join("spec.json")).unwrap();
            assert_eq!(spec_json_content.trim(), "{}");

            let requirements_content =
                std::fs::read_to_string(feature_path.join("requirements.md")).unwrap();
            assert_eq!(requirements_content, "");

            break;
        }
    }

    assert!(
        feature_dir_found,
        "Feature directory with correct naming should be created"
    );
}

#[test]
fn test_new_command_e2e_invalid_name() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Run: hail-mary new Invalid_Name (with underscore)
    let output = run_hail_mary_command(&["new", "Invalid_Name"], temp_path).unwrap();

    // Check command failed
    assert!(
        !output.status.success(),
        "Command should fail with invalid name"
    );

    // Check error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("InvalidFeatureName"),
        "Should contain invalid name error, got: {}",
        stderr
    );
}

#[test]
fn test_new_command_e2e_duplicate_feature() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // First run: hail-mary new test-feature
    let output1 = run_hail_mary_command(&["new", "test-feature"], temp_path).unwrap();
    assert!(output1.status.success(), "First command should succeed");

    // Second run: hail-mary new test-feature (duplicate)
    let output2 = run_hail_mary_command(&["new", "test-feature"], temp_path).unwrap();

    // Check second command failed
    assert!(
        !output2.status.success(),
        "Second command should fail due to duplicate"
    );

    // Check error message
    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(
        stderr.contains("FeatureAlreadyExists"),
        "Should contain duplicate error, got: {}",
        stderr
    );
}

#[test]
fn test_new_command_e2e_help() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Run: hail-mary new --help
    let output = run_hail_mary_command(&["new", "--help"], temp_path).unwrap();

    // Check command succeeded
    assert!(output.status.success(), "Help command should succeed");

    // Check help output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Create a new feature specification"));
    assert!(stdout.contains("FEATURE_NAME"));
    assert!(stdout.contains("kebab-case"));
}

#[test]
fn test_new_command_e2e_complex_feature_name() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Run: hail-mary new complex-feature-name-123
    let output = run_hail_mary_command(&["new", "complex-feature-name-123"], temp_path).unwrap();

    // Check command succeeded
    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that feature directory was created with correct name
    let specs_dir = temp_dir.path().join(".kiro/specs");
    let entries = std::fs::read_dir(&specs_dir).unwrap();
    let mut found = false;

    for entry in entries {
        let entry = entry.unwrap();
        let dir_name = entry.file_name().to_str().unwrap().to_string();

        if dir_name.ends_with("-complex-feature-name-123") {
            found = true;
            break;
        }
    }

    assert!(found, "Should create directory with complex feature name");
}
