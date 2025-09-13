use crate::application::errors::ApplicationError;
use crate::application::repositories::ConfigRepositoryInterface;
use crate::domain::entities::project::{DocumentFormat, ProjectConfig};
use crate::domain::entities::steering::{
    Criterion, SteeringBackupConfig, SteeringConfig, SteeringType,
};
use crate::infrastructure::filesystem::path_manager::PathManager;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct TomlConfig {
    #[serde(default = "default_instructions")]
    instructions: Option<String>,
    #[serde(default)]
    document_format: Option<DocumentFormat>,
    #[serde(default)]
    steering: Option<SteeringSection>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SteeringSection {
    #[serde(default)]
    types: Vec<SteeringTypeToml>,
    #[serde(default)]
    backup: Option<SteeringBackupToml>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SteeringTypeToml {
    name: String,
    purpose: String,
    criteria: Vec<String>,
    #[serde(default)]
    allowed_operations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SteeringBackupToml {
    #[serde(default = "default_backup_max")]
    max: usize,
}

fn default_instructions() -> Option<String> {
    Some("Follow the project guidelines and best practices.".to_string())
}

fn default_backup_max() -> usize {
    10
}

pub struct ConfigRepository {
    path_manager: PathManager,
}

impl ConfigRepository {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }

    fn load_toml(&self) -> Result<toml::Value, ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        if !config_path.exists() {
            return Ok(toml::Value::Table(toml::map::Map::new()));
        }

        let content = fs::read_to_string(&config_path).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to read config file: {}", e))
        })?;

        toml::from_str(&content).map_err(|e| {
            ApplicationError::ConfigurationError(format!("Failed to parse TOML: {}", e))
        })
    }

    fn save_toml(&self, value: &toml::Value) -> Result<(), ApplicationError> {
        let config_path = self.path_manager.config_path(true);
        let content = toml::to_string_pretty(value).map_err(|e| {
            ApplicationError::ConfigurationError(format!("Failed to serialize TOML: {}", e))
        })?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ApplicationError::FileSystemError(format!(
                    "Failed to create config directory: {}",
                    e
                ))
            })?;
        }

        fs::write(config_path, content).map_err(|e| {
            ApplicationError::FileSystemError(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    fn parse_steering_config(steering_section: &SteeringSection) -> SteeringConfig {
        let types = steering_section
            .types
            .iter()
            .map(|t| SteeringType {
                name: t.name.clone(),
                purpose: t.purpose.clone(),
                criteria: t
                    .criteria
                    .iter()
                    .filter_map(|c| {
                        if let Some((name, desc)) = c.split_once(": ") {
                            Some(Criterion {
                                name: name.to_string(),
                                description: desc.to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
                allowed_operations: t.allowed_operations.clone(),
            })
            .collect();

        let backup = Self::parse_backup_config(&steering_section.backup);

        SteeringConfig { types, backup }
    }

    fn parse_backup_config(backup: &Option<SteeringBackupToml>) -> SteeringBackupConfig {
        backup
            .as_ref()
            .map(|b| SteeringBackupConfig { max: b.max })
            .unwrap_or_else(|| SteeringBackupConfig {
                max: default_backup_max(),
            })
    }
}

impl ConfigRepositoryInterface for ConfigRepository {
    fn load_config(&self) -> Result<ProjectConfig, ApplicationError> {
        let toml_value = self.load_toml()?;

        if let Ok(config) = toml_value.try_into::<TomlConfig>() {
            let steering = config
                .steering
                .as_ref()
                .map(Self::parse_steering_config)
                .unwrap_or_else(SteeringConfig::default_for_new_project);

            Ok(ProjectConfig {
                instructions: config
                    .instructions
                    .unwrap_or_else(|| default_instructions().unwrap()),
                document_format: config.document_format.unwrap_or_default(),
                steering,
            })
        } else {
            // Return default config if parsing fails
            Ok(ProjectConfig::default_for_new_project())
        }
    }

    fn save_config(&self, config: &ProjectConfig) -> Result<(), ApplicationError> {
        let config_path = self.path_manager.config_path(true);

        // Never overwrite existing config.toml
        if config_path.exists() {
            return Ok(());
        }

        let toml_config = TomlConfig {
            instructions: Some(config.instructions.clone()),
            document_format: Some(config.document_format.clone()),
            steering: Some(SteeringSection {
                types: config
                    .steering
                    .types
                    .iter()
                    .map(|t| SteeringTypeToml {
                        name: t.name.clone(),
                        purpose: t.purpose.clone(),
                        criteria: t
                            .criteria
                            .iter()
                            .map(|c| format!("{}: {}", c.name, c.description))
                            .collect(),
                        allowed_operations: t.allowed_operations.clone(),
                    })
                    .collect(),
                backup: Some(SteeringBackupToml {
                    max: default_backup_max(),
                }),
            }),
        };

        let toml_value = toml::Value::try_from(toml_config).map_err(|e| {
            ApplicationError::ConfigurationError(format!("Failed to convert to TOML: {}", e))
        })?;

        self.save_toml(&toml_value)
    }

    fn load_steering_config(&self) -> Result<SteeringConfig, ApplicationError> {
        let toml_value = self.load_toml()?;

        if let Ok(config) = toml_value.try_into::<TomlConfig>() {
            Ok(config
                .steering
                .as_ref()
                .map(Self::parse_steering_config)
                .unwrap_or_else(SteeringConfig::default_for_new_project))
        } else {
            Ok(SteeringConfig::default_for_new_project())
        }
    }

    fn load_steering_backup_config(&self) -> Result<SteeringBackupConfig, ApplicationError> {
        let toml_value = self.load_toml()?;

        if let Ok(config) = toml_value.try_into::<TomlConfig>() {
            Ok(config
                .steering
                .as_ref()
                .map(|s| Self::parse_backup_config(&s.backup))
                .unwrap_or_else(|| SteeringBackupConfig {
                    max: default_backup_max(),
                }))
        } else {
            Ok(SteeringBackupConfig {
                max: default_backup_max(),
            })
        }
    }

    fn ensure_steering_config(&self) -> Result<(), ApplicationError> {
        let mut toml_value = self.load_toml()?;
        let table = toml_value.as_table_mut().ok_or_else(|| {
            ApplicationError::ConfigurationError("Invalid TOML structure".to_string())
        })?;

        // Check if steering section exists
        if !table.contains_key("steering") {
            // Add default steering section
            let default_steering = SteeringConfig::default_for_new_project();
            let steering_section = SteeringSection {
                types: default_steering
                    .types
                    .iter()
                    .map(|t| SteeringTypeToml {
                        name: t.name.clone(),
                        purpose: t.purpose.clone(),
                        criteria: t
                            .criteria
                            .iter()
                            .map(|c| format!("{}: {}", c.name, c.description))
                            .collect(),
                        allowed_operations: t.allowed_operations.clone(),
                    })
                    .collect(),
                backup: None,
            };

            let steering_value = toml::Value::try_from(steering_section).map_err(|e| {
                ApplicationError::ConfigurationError(format!("Failed to serialize steering: {}", e))
            })?;

            table.insert("steering".to_string(), steering_value);
            self.save_toml(&toml_value)?;
        }

        Ok(())
    }

    fn ensure_steering_backup_config(&self) -> Result<(), ApplicationError> {
        let mut toml_value = self.load_toml()?;
        let table = toml_value.as_table_mut().ok_or_else(|| {
            ApplicationError::ConfigurationError("Invalid TOML structure".to_string())
        })?;

        // Ensure steering section exists first
        if !table.contains_key("steering") {
            self.ensure_steering_config()?;
            toml_value = self.load_toml()?;
        }

        let table = toml_value.as_table_mut().ok_or_else(|| {
            ApplicationError::ConfigurationError("Invalid TOML structure".to_string())
        })?;

        // Check if backup exists in steering section
        if let Some(steering) = table.get_mut("steering")
            && let Some(steering_table) = steering.as_table_mut()
            && !steering_table.contains_key("backup")
        {
            let backup_config = SteeringBackupToml {
                max: default_backup_max(),
            };
            let backup_value = toml::Value::try_from(backup_config).map_err(|e| {
                ApplicationError::ConfigurationError(format!(
                    "Failed to serialize backup config: {}",
                    e
                ))
            })?;
            steering_table.insert("backup".to_string(), backup_value);
            self.save_toml(&toml_value)?;
        }

        Ok(())
    }

    fn ensure_allowed_operations(&self) -> Result<(), ApplicationError> {
        let mut toml_value = self.load_toml()?;
        let table = toml_value.as_table_mut().ok_or_else(|| {
            ApplicationError::ConfigurationError("Invalid TOML structure".to_string())
        })?;

        // Ensure steering section exists first
        if !table.contains_key("steering") {
            self.ensure_steering_config()?;
            toml_value = self.load_toml()?;
        }

        let table = toml_value.as_table_mut().ok_or_else(|| {
            ApplicationError::ConfigurationError("Invalid TOML structure".to_string())
        })?;

        let mut modified = false;

        if let Some(steering) = table.get_mut("steering")
            && let Some(steering_table) = steering.as_table_mut()
        {
            // Get the types array
            if let Some(types) = steering_table.get_mut("types")
                && let Some(types_array) = types.as_array_mut()
            {
                // Iterate through each steering type in the array
                for type_value in types_array.iter_mut() {
                    if let Some(type_table) = type_value.as_table_mut() {
                        // Check if allowed_operations already exists
                        if !type_table.contains_key("allowed_operations") {
                            // Get the type name to determine default operations
                            let type_name = type_table
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");

                            // Add default based on type name
                            let default_ops = match type_name {
                                "product" | "tech" | "structure" => {
                                    vec!["refresh".to_string(), "discover".to_string()]
                                }
                                _ => vec![],
                            };

                            let ops_value = toml::Value::Array(
                                default_ops.into_iter().map(toml::Value::String).collect(),
                            );
                            type_table.insert("allowed_operations".to_string(), ops_value);
                            modified = true;
                        }
                    }
                }
            }
        }

        if modified {
            self.save_toml(&toml_value)?;
        }

        Ok(())
    }
}
