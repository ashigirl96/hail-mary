use crate::application::errors::ApplicationError;
use crate::application::repositories::ConfigRepositoryInterface;
use crate::domain::value_objects::steering::{
    Criterion, SpecConfig, SteeringBackupConfig, SteeringConfig, SteeringType,
};
use crate::infrastructure::filesystem::path_manager::PathManager;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct TomlConfig {
    #[serde(default)]
    steering: Option<SteeringSection>,
    #[serde(default)]
    spec: Option<SpecSection>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SpecSection {
    #[serde(default = "default_spec_lang")]
    lang: String,
}

fn default_spec_lang() -> String {
    "ja".to_string()
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

    fn load_spec_config(&self) -> Result<SpecConfig, ApplicationError> {
        let toml_value = self.load_toml()?;

        if let Ok(config) = toml_value.try_into::<TomlConfig>() {
            Ok(config
                .spec
                .as_ref()
                .map(|s| SpecConfig {
                    lang: s.lang.clone(),
                })
                .unwrap_or_default())
        } else {
            Ok(SpecConfig::default())
        }
    }

    fn ensure_spec_config(&self) -> Result<(), ApplicationError> {
        let mut toml_value = self.load_toml()?;
        let table = toml_value.as_table_mut().ok_or_else(|| {
            ApplicationError::ConfigurationError("Invalid TOML structure".to_string())
        })?;

        // Check if spec section exists
        if !table.contains_key("spec") {
            // Add default spec section
            let spec_section = SpecSection {
                lang: default_spec_lang(),
            };

            let spec_value = toml::Value::try_from(spec_section).map_err(|e| {
                ApplicationError::ConfigurationError(format!("Failed to serialize spec: {}", e))
            })?;

            table.insert("spec".to_string(), spec_value);
            self.save_toml(&toml_value)?;
        }

        Ok(())
    }
}
