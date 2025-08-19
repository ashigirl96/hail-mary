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
fn test_init_command_e2e_success() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Run: hail-mary init
    let output = run_hail_mary_command(&["init"], temp_path).unwrap();

    // Check command succeeded
    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check output contains success message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Initialized .kiro directory structure"));
    assert!(stdout.contains("Created .kiro/"));
    assert!(stdout.contains("Created .kiro/config.toml"));
    assert!(stdout.contains("Created .kiro/memory/"));
    assert!(stdout.contains("Created .kiro/specs/"));
    assert!(stdout.contains("Updated .gitignore"));

    // Check that .kiro directory structure was created
    let kiro_dir = temp_dir.path().join(".kiro");
    assert!(kiro_dir.exists(), ".kiro directory should exist");
    assert!(kiro_dir.is_dir(), ".kiro should be a directory");

    let config_file = kiro_dir.join("config.toml");
    assert!(config_file.exists(), "config.toml should exist");

    let memory_dir = kiro_dir.join("memory");
    assert!(memory_dir.exists(), ".kiro/memory directory should exist");
    assert!(memory_dir.is_dir(), ".kiro/memory should be a directory");

    let specs_dir = kiro_dir.join("specs");
    assert!(specs_dir.exists(), ".kiro/specs directory should exist");
    assert!(specs_dir.is_dir(), ".kiro/specs should be a directory");

    // Check config.toml content
    let config_content = std::fs::read_to_string(config_file).unwrap();
    assert!(config_content.contains("[memory]"));
    assert!(config_content.contains("types = ["));
    assert!(config_content.contains("tech"));
    assert!(config_content.contains("project-tech"));
    assert!(config_content.contains("domain"));
    assert!(config_content.contains("[memory.database]"));
    assert!(config_content.contains(".kiro/memory/db.sqlite3"));

    // Check .gitignore was created and updated
    let gitignore_file = temp_dir.path().join(".gitignore");
    assert!(gitignore_file.exists(), ".gitignore should be created");

    let gitignore_content = std::fs::read_to_string(gitignore_file).unwrap();
    assert!(gitignore_content.contains("hail-mary memory database"));
    assert!(gitignore_content.contains(".kiro/memory/db.sqlite3"));
    assert!(gitignore_content.contains(".kiro/memory/*.sqlite3-*"));
}

#[test]
fn test_init_command_e2e_force_flag() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Run init first time
    let output1 = run_hail_mary_command(&["init"], temp_path).unwrap();
    assert!(output1.status.success(), "First init should succeed");

    // Modify config.toml to test force overwrite
    let config_file = temp_dir.path().join(".kiro/config.toml");
    std::fs::write(&config_file, "# modified config").unwrap();

    // Run init second time without force - should fail
    let output2 = run_hail_mary_command(&["init"], temp_path).unwrap();
    assert!(
        !output2.status.success(),
        "Second init without force should fail"
    );

    // Verify config wasn't overwritten
    let config_content = std::fs::read_to_string(&config_file).unwrap();
    assert!(config_content.contains("# modified config"));

    // Run init with force flag - should succeed
    let output3 = run_hail_mary_command(&["init", "--force"], temp_path).unwrap();
    assert!(
        output3.status.success(),
        "Init with force should succeed, stderr: {}",
        String::from_utf8_lossy(&output3.stderr)
    );

    // Verify config was overwritten
    let config_content = std::fs::read_to_string(&config_file).unwrap();
    assert!(!config_content.contains("# modified config"));
    assert!(config_content.contains("[memory]"));
}

#[test]
fn test_init_command_e2e_with_existing_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Create existing .gitignore
    let gitignore_file = temp_dir.path().join(".gitignore");
    std::fs::write(&gitignore_file, "*.log\ntarget/\n").unwrap();

    // Run: hail-mary init
    let output = run_hail_mary_command(&["init"], temp_path).unwrap();
    assert!(
        output.status.success(),
        "Init should succeed with existing .gitignore, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that existing .gitignore content was preserved and new content added
    let gitignore_content = std::fs::read_to_string(&gitignore_file).unwrap();

    // Should preserve existing content
    assert!(gitignore_content.contains("*.log"));
    assert!(gitignore_content.contains("target/"));

    // Should add new content
    assert!(gitignore_content.contains("hail-mary memory database"));
    assert!(gitignore_content.contains(".kiro/memory/db.sqlite3"));
    assert!(gitignore_content.contains(".kiro/memory/*.sqlite3-*"));
}

#[test]
fn test_init_command_e2e_help() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Run: hail-mary init --help
    let output = run_hail_mary_command(&["init", "--help"], temp_path).unwrap();

    // Check command succeeded
    assert!(output.status.success(), "Help command should succeed");

    // Check help output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Initialize .kiro directory and configuration"));
    assert!(stdout.contains("--force"));
    assert!(stdout.contains("Force overwrite existing configuration"));
}

#[test]
fn test_init_command_e2e_duplicate_without_force() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // First run: hail-mary init
    let output1 = run_hail_mary_command(&["init"], temp_path).unwrap();
    assert!(output1.status.success(), "First init should succeed");

    // Second run: hail-mary init (without force)
    let output2 = run_hail_mary_command(&["init"], temp_path).unwrap();

    // Check second command failed
    assert!(
        !output2.status.success(),
        "Second init without force should fail"
    );

    // Check error message
    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(
        stderr.contains(".kiro directory already exists") || stderr.contains("already exists"),
        "Should contain directory exists error, got: {}",
        stderr
    );
}
