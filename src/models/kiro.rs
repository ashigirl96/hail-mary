use crate::models::error::{MemoryError, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct KiroConfig {
    pub root_dir: PathBuf,
    pub memory: MemoryConfig,
}

impl Default for KiroConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from(".kiro"),
            memory: MemoryConfig {
                types: vec![
                    "tech".to_string(),
                    "project-tech".to_string(),
                    "domain".to_string(),
                ],
                instructions: "Default memory types for hail-mary".to_string(),
                document: DocumentConfig {
                    output_dir: PathBuf::from(".kiro/memory"),
                    format: "markdown".to_string(),
                },
                database: DatabaseConfig {
                    path: PathBuf::from(".kiro/memory/db.sqlite3"),
                },
            },
        }
    }
}

impl KiroConfig {
    pub fn load_from_root(root: &Path) -> Result<Self> {
        let config_path = root.join("config.toml");

        let mut config = if config_path.exists() {
            let contents = fs::read_to_string(&config_path).map_err(MemoryError::Io)?;

            // Parse just the memory section and create a config with it
            let parsed: MemoryConfigFile = toml::from_str(&contents)
                .map_err(|e| MemoryError::InvalidInput(format!("Invalid TOML: {}", e)))?;

            Self {
                root_dir: root.to_path_buf(),
                memory: parsed.memory,
            }
        } else {
            // Return default config if no config.toml exists
            Self {
                root_dir: root.to_path_buf(),
                ..Self::default()
            }
        };

        // Always set the root_dir to the provided directory
        config.root_dir = root.to_path_buf();
        Ok(config)
    }

    pub fn load() -> Result<Self> {
        let root = Self::find_kiro_root()?;
        Self::load_from_root(&root)
    }

    pub fn validate_memory_type(&self, memory_type: &str) -> bool {
        self.memory.types.contains(&memory_type.to_string())
    }

    pub fn find_kiro_root_from(start_dir: &Path) -> Result<PathBuf> {
        let mut current_dir = start_dir.to_path_buf();

        loop {
            let kiro_dir = current_dir.join(".kiro");
            if kiro_dir.exists() && kiro_dir.is_dir() {
                return Ok(kiro_dir);
            }

            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                return Err(MemoryError::NotFound(
                    ".kiro directory not found".to_string(),
                ));
            }
        }
    }

    pub fn find_kiro_root() -> Result<PathBuf> {
        let current = std::env::current_dir().map_err(MemoryError::Io)?;
        Self::find_kiro_root_from(&current)
    }

    pub fn memory_docs_dir(&self) -> PathBuf {
        self.memory.document.output_dir.clone()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryConfig {
    pub types: Vec<String>,
    pub instructions: String,
    pub document: DocumentConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocumentConfig {
    pub output_dir: PathBuf,
    #[allow(dead_code)] // Used in future document format options
    pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}

// Helper struct for parsing TOML files (without root_dir)
#[derive(Debug, Deserialize)]
struct MemoryConfigFile {
    memory: MemoryConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp directory")
    }

    fn create_test_config_toml() -> String {
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
    use pretty_assertions::assert_eq;
    use std::fs;

    #[test]
    fn test_kiro_config_load_from_file() {
        // Create a temporary directory with config.toml
        let temp_dir = setup_test_dir();
        let config_path = temp_dir.path().join("config.toml");

        fs::write(&config_path, create_test_config_toml()).unwrap();

        // Test that the memory section can be parsed correctly using MemoryConfigFile
        let content = fs::read_to_string(&config_path).unwrap();
        let parsed: MemoryConfigFile = toml::from_str(&content).unwrap();

        // Check memory types from parsed config
        assert_eq!(parsed.memory.types, vec!["tech", "project-tech", "domain"]);
        assert_eq!(
            parsed.memory.instructions,
            "Test memory types for unit testing"
        );

        // Check document config
        assert_eq!(
            parsed.memory.document.output_dir,
            PathBuf::from(".kiro/memory")
        );
        assert_eq!(parsed.memory.document.format, "markdown");

        // Check database config
        assert_eq!(
            parsed.memory.database.path,
            PathBuf::from(".kiro/memory/db.sqlite3")
        );
    }

    #[test]
    fn test_kiro_config_default() {
        // Test default configuration generation
        let config = KiroConfig::default();

        assert_eq!(config.root_dir, PathBuf::from(".kiro"));
        assert_eq!(config.memory.types, vec!["tech", "project-tech", "domain"]);
        assert_eq!(config.memory.document.format, "markdown");
        assert!(config.memory.instructions.contains("memory types"));
    }

    #[test]
    fn test_kiro_config_validate_memory_type() {
        let config = KiroConfig::default();

        // Test valid memory types
        assert!(config.validate_memory_type("tech"));
        assert!(config.validate_memory_type("project-tech"));
        assert!(config.validate_memory_type("domain"));

        // Test invalid memory types
        assert!(!config.validate_memory_type("invalid"));
        assert!(!config.validate_memory_type(""));
        assert!(!config.validate_memory_type("TECH")); // Case sensitive
    }

    #[test]
    fn test_kiro_config_find_kiro_root() {
        // Test finding .kiro directory
        let temp_dir = setup_test_dir();
        let kiro_dir = temp_dir.path().join(".kiro");
        fs::create_dir_all(&kiro_dir).unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Test finding .kiro root
        let found_root = KiroConfig::find_kiro_root().unwrap();
        assert!(found_root.ends_with(".kiro"));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_kiro_config_find_kiro_root_not_found() {
        // Test when .kiro directory doesn't exist using find_kiro_root_from instead
        let temp_dir = setup_test_dir();

        // Test that find_kiro_root_from returns error when no .kiro exists
        let result = KiroConfig::find_kiro_root_from(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_kiro_config_memory_docs_dir() {
        let config = KiroConfig::default();
        let docs_dir = config.memory_docs_dir();

        assert!(docs_dir.to_str().unwrap().contains("memory"));
    }

    #[test]
    fn test_kiro_config_load_missing_file() {
        // Test loading when config file doesn't exist but .kiro directory exists
        let temp_dir = setup_test_dir();

        // Create .kiro directory but no config.toml
        let kiro_dir = temp_dir.path().join(".kiro");
        fs::create_dir_all(&kiro_dir).unwrap();

        // Use new API to avoid global state changes
        let root = KiroConfig::find_kiro_root_from(temp_dir.path()).unwrap();
        let config = KiroConfig::load_from_root(&root).unwrap();

        assert_eq!(config.memory.types, vec!["tech", "project-tech", "domain"]);
        assert!(config.root_dir.ends_with(".kiro"));
    }
}
