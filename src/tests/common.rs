use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

// Global mutex to synchronize current directory changes across tests
static TEST_DIR_MUTEX: Mutex<()> = Mutex::new(());

/// Test utilities for Memory MCP v3 tests
///
/// Setup temporary directory for tests
pub fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Test directory helper that automatically changes to a temporary directory
/// and restores the original directory when dropped (RAII pattern)
/// Uses a global mutex to prevent current_dir race conditions in parallel tests
pub struct TestDirectory {
    _temp_dir: TempDir,
    original_dir: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl Default for TestDirectory {
    fn default() -> Self {
        Self::new()
    }
}

impl TestDirectory {
    /// Creates a new temporary directory and changes current directory to it
    /// Acquires a global lock to prevent race conditions in parallel test execution
    pub fn new() -> Self {
        let guard = TEST_DIR_MUTEX
            .lock()
            .expect("Failed to acquire test directory mutex");
        let original_dir = env::current_dir().expect("Failed to get current directory");
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        env::set_current_dir(temp_dir.path()).expect("Failed to change to temp directory");

        Self {
            _temp_dir: temp_dir,
            original_dir,
            _guard: guard,
        }
    }

    /// Returns the path to the temporary directory
    pub fn path(&self) -> &Path {
        self._temp_dir.path()
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        // Best effort to restore original directory
        // Ignore errors since we're in cleanup phase
        let _ = env::set_current_dir(&self.original_dir);
        // Mutex guard is automatically released when _guard is dropped
    }
}

/// Initialize test logging
pub fn init_test_logging() {
    let _ = tracing_subscriber::fmt().try_init();
}

/// Create test TOML content for KiroConfig testing
pub fn create_test_config_toml() -> String {
    r#"
[memory]
types = ["tech", "project-tech", "domain"]
instructions = "Test memory types for unit testing"

[memory.document]
output_dir = ".kiro/memory"
format = "markdown"

[memory.database]
path = ".kiro/memory/db.sqlite3"
"#
    .to_string()
}

/// Test data fixtures
pub mod fixtures {
    pub fn test_memory_title() -> String {
        "Test Memory Title".to_string()
    }

    pub fn test_memory_content() -> String {
        "This is test memory content for testing purposes.".to_string()
    }

    pub fn test_tags() -> Vec<String> {
        vec!["test".to_string(), "unit".to_string(), "rust".to_string()]
    }
}
