//! Test directory helper for managing temporary directories in tests
//!
//! This module provides utilities for managing test environments,
//! particularly for handling temporary directories and current directory changes
//! in a thread-safe manner during parallel test execution.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

// Global mutex to synchronize current directory changes across tests
// This prevents race conditions when multiple tests try to change the
// current directory simultaneously during parallel test execution
static TEST_DIR_MUTEX: Mutex<()> = Mutex::new(());

/// Test directory helper that automatically changes to a temporary directory
/// and restores the original directory when dropped (RAII pattern).
///
/// Uses a global mutex to prevent current_dir race conditions in parallel tests.
/// This is essential because `env::set_current_dir()` modifies global process state,
/// which can cause tests to interfere with each other when running in parallel.
///
/// # Examples
///
/// ```
/// use hail_mary::application::test_helpers::TestDirectory;
///
/// fn test_with_temp_dir() {
///     let test_dir = TestDirectory::new();
///     // Current directory is now a temp directory
///     
///     // Do test operations...
///     std::fs::write("test.txt", "content").unwrap();
///     
///     // TestDirectory automatically restores original directory on drop
/// }
/// ```
pub struct TestDirectory {
    _temp_dir: TempDir,
    original_dir: Option<PathBuf>,
    _guard: Option<MutexGuard<'static, ()>>,
}

impl Default for TestDirectory {
    fn default() -> Self {
        Self::new()
    }
}

impl TestDirectory {
    /// Creates a new temporary directory and changes current directory to it.
    ///
    /// Acquires a global lock to prevent race conditions in parallel test execution.
    /// The lock is held for the lifetime of the TestDirectory instance.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Failed to acquire the mutex lock
    /// - Failed to get the current directory
    /// - Failed to create a temporary directory
    /// - Failed to change to the temporary directory
    pub fn new() -> Self {
        // Acquire global lock to prevent concurrent directory changes
        // Handle PoisonError by ignoring the poison state (test cleanup is best-effort)
        let guard = TEST_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Save current directory for restoration
        let original_dir = env::current_dir().expect("Failed to get current directory");

        // Create temporary directory
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        // Change to temporary directory
        env::set_current_dir(temp_dir.path()).expect("Failed to change to temp directory");

        Self {
            _temp_dir: temp_dir,
            original_dir: Some(original_dir),
            _guard: Some(guard),
        }
    }

    /// Creates a new temporary directory WITHOUT changing the current directory.
    ///
    /// This is useful for tests that only need a temporary directory path
    /// but don't need to change the current directory. Since no global state
    /// is modified, this doesn't require mutex locking.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Failed to create a temporary directory
    pub fn new_no_cd() -> Self {
        // Create temporary directory
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");

        // No need for mutex or original directory since we're not changing directories
        Self {
            _temp_dir: temp_dir,
            original_dir: None,
            _guard: None,
        }
    }

    /// Creates a new temporary directory with a specific prefix for identification.
    ///
    /// Useful for debugging test failures as the directory name will contain the prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Prefix for the temporary directory name
    pub fn with_prefix(prefix: &str) -> Self {
        // Acquire global lock to prevent concurrent directory changes
        // Handle PoisonError by ignoring the poison state (test cleanup is best-effort)
        let guard = TEST_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Save current directory for restoration
        let original_dir = env::current_dir().expect("Failed to get current directory");

        // Create temporary directory with prefix
        let temp_dir = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir()
            .expect("Failed to create temp directory with prefix");

        // Change to temporary directory
        env::set_current_dir(temp_dir.path()).expect("Failed to change to temp directory");

        Self {
            _temp_dir: temp_dir,
            original_dir: Some(original_dir),
            _guard: Some(guard),
        }
    }

    /// Returns the path to the temporary directory.
    ///
    /// This can be used to access files within the test directory
    /// or to pass the path to functions under test.
    pub fn path(&self) -> &Path {
        self._temp_dir.path()
    }

    /// Creates a subdirectory within the test directory.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the subdirectory to create
    ///
    /// # Returns
    ///
    /// Path to the created subdirectory
    pub fn create_subdir(&self, name: &str) -> PathBuf {
        let subdir = self.path().join(name);
        fs::create_dir_all(&subdir).expect("Failed to create subdirectory");
        subdir
    }

    /// Creates a file within the test directory.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the file to create
    /// * `content` - Content to write to the file
    ///
    /// # Returns
    ///
    /// Path to the created file
    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let file_path = self.path().join(name);
        fs::write(&file_path, content).expect("Failed to create file");
        file_path
    }
}

impl Drop for TestDirectory {
    /// Restores the original directory when the TestDirectory is dropped.
    ///
    /// This ensures that the test environment is properly cleaned up even if
    /// the test panics or fails. The mutex guard is automatically released
    /// when dropped, allowing other tests to proceed.
    fn drop(&mut self) {
        // Only restore directory if we changed it (i.e., original_dir is Some)
        if let Some(ref original_dir) = self.original_dir {
            // Best effort to restore original directory
            // We ignore errors here since we're in the cleanup phase
            // and don't want to panic during drop
            let _ = env::set_current_dir(original_dir);
        }

        // Mutex guard is automatically released when _guard is dropped (if Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_with_prefix() {
        let test_dir = TestDirectory::with_prefix("test_prefix_");
        let path_str = test_dir.path().to_string_lossy();
        assert!(path_str.contains("test_prefix_"));
    }

    #[test]
    fn test_create_subdir() {
        let test_dir = TestDirectory::new();
        let subdir = test_dir.create_subdir("subdir");

        assert!(subdir.exists());
        assert!(subdir.is_dir());
    }

    #[test]
    fn test_create_file() {
        let test_dir = TestDirectory::new();
        let file_path = test_dir.create_file("test.txt", "test content");

        assert!(file_path.exists());
        assert!(file_path.is_file());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }
}
