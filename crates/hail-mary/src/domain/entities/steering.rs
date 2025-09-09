use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringType {
    pub name: String,
    pub purpose: String,
    pub criteria: Vec<Criterion>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringBackupConfig {
    #[serde(default = "default_backup_max")]
    pub max: usize,
}

impl Default for SteeringBackupConfig {
    fn default() -> Self {
        Self { max: 10 }
    }
}

fn default_backup_max() -> usize {
    10
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Criterion {
    pub name: String,
    pub description: String,
}

impl Criterion {
    /// Parse criterion from "Name: Description" format
    pub fn parse_from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() == 2 {
            Some(Self {
                name: parts[0].trim().to_string(),
                description: parts[1].trim().to_string(),
            })
        } else {
            None
        }
    }
}

impl fmt::Display for Criterion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.description)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringConfig {
    #[serde(default)]
    pub types: Vec<SteeringType>,
    #[serde(default)]
    pub backup: SteeringBackupConfig,
}

impl SteeringConfig {
    pub fn default_for_new_project() -> Self {
        Self {
            backup: SteeringBackupConfig::default(),
            types: vec![
                SteeringType {
                    name: "product".to_string(),
                    purpose: "Product overview and value proposition".to_string(),
                    criteria: vec![
                        Criterion {
                            name: "Product Overview".to_string(),
                            description: "Brief description of what the product is".to_string(),
                        },
                        Criterion {
                            name: "Core Features".to_string(),
                            description: "Bulleted list of main capabilities".to_string(),
                        },
                        Criterion {
                            name: "Target Use Case".to_string(),
                            description: "Specific scenarios the product addresses".to_string(),
                        },
                        Criterion {
                            name: "Key Value Proposition".to_string(),
                            description: "Unique benefits and differentiators".to_string(),
                        },
                    ],
                },
                SteeringType {
                    name: "tech".to_string(),
                    purpose: "Technical stack and development environment".to_string(),
                    criteria: vec![
                        Criterion {
                            name: "Architecture".to_string(),
                            description: "High-level system design".to_string(),
                        },
                        Criterion {
                            name: "Frontend".to_string(),
                            description: "Frameworks, libraries, build tools (if applicable)"
                                .to_string(),
                        },
                        Criterion {
                            name: "Backend".to_string(),
                            description: "Language, framework, server technology (if applicable)"
                                .to_string(),
                        },
                        Criterion {
                            name: "Development Environment".to_string(),
                            description: "Required tools and setup".to_string(),
                        },
                        Criterion {
                            name: "Common Commands".to_string(),
                            description: "Frequently used development commands".to_string(),
                        },
                        Criterion {
                            name: "Environment Variables".to_string(),
                            description: "Key configuration variables".to_string(),
                        },
                        Criterion {
                            name: "Port Configuration".to_string(),
                            description: "Standard ports used by services".to_string(),
                        },
                    ],
                },
                SteeringType {
                    name: "structure".to_string(),
                    purpose: "Code organization and project structure patterns".to_string(),
                    criteria: vec![
                        Criterion {
                            name: "Root Directory Organization".to_string(),
                            description: "Top-level structure with descriptions".to_string(),
                        },
                        Criterion {
                            name: "Subdirectory Structures".to_string(),
                            description: "Detailed breakdown of key directories".to_string(),
                        },
                        Criterion {
                            name: "Code Organization Patterns".to_string(),
                            description: "How code is structured".to_string(),
                        },
                        Criterion {
                            name: "File Naming Conventions".to_string(),
                            description: "Standards for naming files and directories".to_string(),
                        },
                        Criterion {
                            name: "Import Organization".to_string(),
                            description: "How imports/dependencies are organized".to_string(),
                        },
                        Criterion {
                            name: "Key Architectural Principles".to_string(),
                            description: "Core design decisions and patterns".to_string(),
                        },
                    ],
                },
            ],
        }
    }

    /// Parse SteeringConfig from criterion strings in "Name: Description" format
    pub fn from_criterion_strings(
        name: &str,
        purpose: &str,
        criterion_strings: Vec<String>,
    ) -> Result<SteeringType, String> {
        let mut criteria = Vec::new();

        for criterion_str in criterion_strings {
            if let Some(criterion) = Criterion::parse_from_string(&criterion_str) {
                criteria.push(criterion);
            } else {
                return Err(format!("Invalid criterion format: {}", criterion_str));
            }
        }

        Ok(SteeringType {
            name: name.to_string(),
            purpose: purpose.to_string(),
            criteria,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_criterion_parse_from_string_valid() {
        let criterion = Criterion::parse_from_string(
            "Product Overview: Brief description of what the product is",
        );

        assert!(criterion.is_some());
        let criterion = criterion.unwrap();
        assert_eq!(criterion.name, "Product Overview");
        assert_eq!(
            criterion.description,
            "Brief description of what the product is"
        );
    }

    #[test]
    fn test_criterion_parse_from_string_with_colon_in_description() {
        let criterion =
            Criterion::parse_from_string("API Endpoint: GET /api/users: Returns user list");

        assert!(criterion.is_some());
        let criterion = criterion.unwrap();
        assert_eq!(criterion.name, "API Endpoint");
        assert_eq!(criterion.description, "GET /api/users: Returns user list");
    }

    #[test]
    fn test_criterion_parse_from_string_with_whitespace() {
        let criterion = Criterion::parse_from_string("  Name  :  Description with spaces  ");

        assert!(criterion.is_some());
        let criterion = criterion.unwrap();
        assert_eq!(criterion.name, "Name");
        assert_eq!(criterion.description, "Description with spaces");
    }

    #[test]
    fn test_criterion_parse_from_string_invalid() {
        let criterion = Criterion::parse_from_string("Invalid format without colon");
        assert!(criterion.is_none());

        let criterion = Criterion::parse_from_string("Name:");
        assert!(criterion.is_some());
        let criterion = criterion.unwrap();
        assert_eq!(criterion.description, "");
    }

    #[test]
    fn test_criterion_to_string() {
        let criterion = Criterion {
            name: "Test Name".to_string(),
            description: "Test Description".to_string(),
        };

        assert_eq!(criterion.to_string(), "Test Name: Test Description");
    }

    #[test]
    fn test_steering_config_default_for_new_project() {
        let config = SteeringConfig::default_for_new_project();

        assert_eq!(config.types.len(), 3);

        // Test product type
        let product = &config.types[0];
        assert_eq!(product.name, "product");
        assert_eq!(product.purpose, "Product overview and value proposition");
        assert_eq!(product.criteria.len(), 4);
        assert_eq!(product.criteria[0].name, "Product Overview");

        // Test tech type
        let tech = &config.types[1];
        assert_eq!(tech.name, "tech");
        assert_eq!(tech.purpose, "Technical stack and development environment");
        assert_eq!(tech.criteria.len(), 7);
        assert_eq!(tech.criteria[0].name, "Architecture");

        // Test structure type
        let structure = &config.types[2];
        assert_eq!(structure.name, "structure");
        assert_eq!(
            structure.purpose,
            "Code organization and project structure patterns"
        );
        assert_eq!(structure.criteria.len(), 6);
        assert_eq!(structure.criteria[0].name, "Root Directory Organization");
    }

    #[test]
    fn test_from_criterion_strings_valid() {
        let criterion_strings = vec![
            "Name1: Description1".to_string(),
            "Name2: Description2".to_string(),
        ];

        let result =
            SteeringConfig::from_criterion_strings("test", "Test purpose", criterion_strings);

        assert!(result.is_ok());
        let steering_type = result.unwrap();
        assert_eq!(steering_type.name, "test");
        assert_eq!(steering_type.purpose, "Test purpose");
        assert_eq!(steering_type.criteria.len(), 2);
        assert_eq!(steering_type.criteria[0].name, "Name1");
        assert_eq!(steering_type.criteria[0].description, "Description1");
    }

    #[test]
    fn test_from_criterion_strings_invalid() {
        let criterion_strings = vec![
            "Name1: Description1".to_string(),
            "Invalid format".to_string(),
        ];

        let result =
            SteeringConfig::from_criterion_strings("test", "Test purpose", criterion_strings);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Invalid criterion format: Invalid format")
        );
    }

    #[test]
    fn test_steering_type_clone_and_debug() {
        let steering_type = SteeringType {
            name: "test".to_string(),
            purpose: "test purpose".to_string(),
            criteria: vec![Criterion {
                name: "test criterion".to_string(),
                description: "test description".to_string(),
            }],
        };

        let cloned = steering_type.clone();
        assert_eq!(steering_type, cloned);

        let debug_str = format!("{:?}", steering_type);
        assert!(debug_str.contains("SteeringType"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_criterion_clone_and_debug() {
        let criterion = Criterion {
            name: "test".to_string(),
            description: "description".to_string(),
        };

        let cloned = criterion.clone();
        assert_eq!(criterion, cloned);

        let debug_str = format!("{:?}", criterion);
        assert!(debug_str.contains("Criterion"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_steering_config_clone_and_debug() {
        let config = SteeringConfig {
            backup: SteeringBackupConfig::default(),
            types: vec![SteeringType {
                name: "test".to_string(),
                purpose: "test purpose".to_string(),
                criteria: vec![],
            }],
        };

        let cloned = config.clone();
        assert_eq!(config, cloned);

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("SteeringConfig"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_steering_backup_config_default() {
        let config = SteeringBackupConfig::default();
        assert_eq!(config.max, 10);
    }

    #[test]
    fn test_steering_backup_config_clone() {
        let config = SteeringBackupConfig { max: 5 };
        let cloned = config.clone();
        assert_eq!(config, cloned);
        assert_eq!(cloned.max, 5);
    }

    #[test]
    fn test_steering_backup_config_debug() {
        let config = SteeringBackupConfig { max: 15 };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("SteeringBackupConfig"));
        assert!(debug_str.contains("15"));
    }

    #[test]
    fn test_steering_backup_config_partial_eq() {
        let config1 = SteeringBackupConfig { max: 10 };
        let config2 = SteeringBackupConfig { max: 10 };
        let config3 = SteeringBackupConfig { max: 20 };

        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }
}
