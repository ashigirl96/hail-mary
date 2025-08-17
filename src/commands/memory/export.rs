use crate::mcp::server::get_default_db_path;
use crate::memory::{
    models::{Memory, MemoryType},
    repository::{MemoryRepository, SqliteMemoryRepository},
};
use crate::utils::error::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Export memories to JSON or CSV format
#[derive(Args)]
pub struct ExportCommand {
    /// Output format
    #[arg(long, value_enum, default_value = "json")]
    pub format: ExportFormat,

    /// Output file path (defaults to stdout)
    #[arg(long, short, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Filter by memory type
    #[arg(long, value_enum)]
    pub r#type: Option<MemoryType>,

    /// Filter by tags (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,

    /// Minimum confidence score (0.0 to 1.0)
    #[arg(long, value_name = "SCORE")]
    pub min_confidence: Option<f32>,

    /// Maximum age in days
    #[arg(long, value_name = "DAYS")]
    pub max_age_days: Option<i64>,

    /// Include deleted memories
    #[arg(long)]
    pub include_deleted: bool,

    /// Pretty print JSON (ignored for CSV)
    #[arg(long)]
    pub pretty: bool,

    /// Include metadata (reference count, timestamps, etc.)
    #[arg(long)]
    pub include_metadata: bool,

    /// Fields to include in export (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub fields: Option<Vec<String>>,

    /// CSV delimiter (only for CSV format)
    #[arg(long, default_value = ",")]
    pub csv_delimiter: String,

    /// Enable verbose output
    #[arg(long, short)]
    pub verbose: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    Json,
    Csv,
}

/// Export-specific memory representation for JSON output
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMemory {
    pub id: String,
    #[serde(rename = "type")]
    pub memory_type: String,
    pub topic: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_accessed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,
}

impl ExportCommand {
    /// Execute the export command
    pub fn execute(self) -> Result<()> {
        // Validate input
        if let Some(confidence) = self.min_confidence
            && (!(0.0..=1.0).contains(&confidence))
        {
            eprintln!("Error: Confidence score must be between 0.0 and 1.0");
            return Ok(());
        }

        if self.csv_delimiter.len() != 1 {
            eprintln!("Error: CSV delimiter must be a single character");
            return Ok(());
        }

        // Determine database path
        let db_path = self
            .db_path
            .clone()
            .unwrap_or_else(|| get_default_db_path().expect("Failed to get default database path"));

        // Check if database exists
        if !db_path.exists() {
            eprintln!("Error: Database not found at {:?}", db_path);
            eprintln!("Please run 'hail-mary memory serve' first to create the database.");
            return Ok(());
        }

        let repository = SqliteMemoryRepository::new(&db_path)?;

        if self.verbose {
            println!("📦 Export Configuration:");
            println!("  Format: {:?}", self.format);
            if let Some(ref output) = self.output {
                println!("  Output: {:?}", output);
            } else {
                println!("  Output: stdout");
            }
            if let Some(ref t) = self.r#type {
                println!("  Type filter: {:?}", t);
            }
            if let Some(ref tags) = self.tags {
                println!("  Tag filter: {}", tags.join(", "));
            }
            println!("  Include metadata: {}", self.include_metadata);
            println!("  Include deleted: {}", self.include_deleted);
            println!();
        }

        // Load memories
        let memories = self.load_memories(&repository)?;

        // Filter memories
        let filtered_memories = self.apply_filters(memories)?;

        if self.verbose {
            println!("📊 Loaded {} memories for export", filtered_memories.len());
        }

        // Convert to export format
        let export_memories = self.convert_to_export_format(&filtered_memories);

        // Export
        match self.format {
            ExportFormat::Json => self.export_json(&export_memories)?,
            ExportFormat::Csv => self.export_csv(&export_memories)?,
        }

        if self.verbose {
            println!("✅ Export completed successfully!");
        }

        Ok(())
    }

    fn load_memories(&self, repository: &SqliteMemoryRepository) -> Result<Vec<Memory>> {
        let memories = if let Some(ref memory_type) = self.r#type {
            // Load specific type
            repository.browse_by_type(memory_type, usize::MAX)?
        } else {
            // Load all types
            if self.include_deleted {
                repository.browse_all(usize::MAX)?
            } else {
                let mut all = Vec::new();
                for memory_type in &[
                    MemoryType::Tech,
                    MemoryType::ProjectTech,
                    MemoryType::Domain,
                ] {
                    let memories = repository.browse_by_type(memory_type, usize::MAX)?;
                    all.extend(memories);
                }
                all
            }
        };

        Ok(memories)
    }

    fn apply_filters(&self, mut memories: Vec<Memory>) -> Result<Vec<Memory>> {
        // Filter by tags
        if let Some(ref filter_tags) = self.tags {
            memories.retain(|m| {
                filter_tags.iter().all(|tag| {
                    m.tags
                        .iter()
                        .any(|mem_tag| mem_tag.to_lowercase().contains(&tag.to_lowercase()))
                })
            });
        }

        // Filter by confidence
        if let Some(min_conf) = self.min_confidence {
            memories.retain(|m| m.confidence >= min_conf);
        }

        // Filter by age
        if let Some(max_age) = self.max_age_days {
            let cutoff_time = chrono::Utc::now().timestamp() - (max_age * 24 * 60 * 60);
            memories.retain(|m| m.created_at >= cutoff_time);
        }

        Ok(memories)
    }

    fn convert_to_export_format(&self, memories: &[Memory]) -> Vec<ExportMemory> {
        memories
            .iter()
            .map(|m| {
                let created_at = if self.include_metadata {
                    Some(format_timestamp(m.created_at))
                } else {
                    None
                };

                let last_accessed = if self.include_metadata && m.last_accessed.is_some() {
                    Some(format_timestamp(m.last_accessed.unwrap()))
                } else {
                    None
                };

                ExportMemory {
                    id: m.id.clone(),
                    memory_type: m.memory_type.to_string(),
                    topic: m.topic.clone(),
                    tags: m.tags.clone(),
                    content: m.content.clone(),
                    examples: m.examples.clone(),
                    reference_count: if self.include_metadata {
                        Some(m.reference_count)
                    } else {
                        None
                    },
                    confidence: if self.include_metadata {
                        Some(m.confidence)
                    } else {
                        None
                    },
                    created_at,
                    last_accessed,
                    source: if self.include_metadata {
                        m.source.clone()
                    } else {
                        None
                    },
                    deleted: if self.include_deleted {
                        Some(m.deleted)
                    } else {
                        None
                    },
                }
            })
            .collect()
    }

    fn export_json(&self, memories: &[ExportMemory]) -> Result<()> {
        let output = if self.pretty {
            serde_json::to_string_pretty(memories)?
        } else {
            serde_json::to_string(memories)?
        };

        self.write_output(&output)?;
        Ok(())
    }

    fn export_csv(&self, memories: &[ExportMemory]) -> Result<()> {
        let mut output = String::new();
        let delimiter = &self.csv_delimiter;

        // Determine fields to include
        let available_fields = [
            "id",
            "type",
            "topic",
            "tags",
            "content",
            "examples",
            "reference_count",
            "confidence",
            "created_at",
            "last_accessed",
            "source",
            "deleted",
        ];

        let fields = if let Some(ref custom_fields) = self.fields {
            custom_fields.clone()
        } else {
            // Default fields based on settings
            let mut default_fields = vec!["id", "type", "topic", "tags", "content"];
            if !memories.is_empty() && !memories[0].examples.is_empty() {
                default_fields.push("examples");
            }
            if self.include_metadata {
                default_fields.extend_from_slice(&["reference_count", "confidence", "created_at"]);
            }
            if self.include_deleted {
                default_fields.push("deleted");
            }
            default_fields.into_iter().map(|s| s.to_string()).collect()
        };

        // Validate fields
        for field in &fields {
            if !available_fields.contains(&field.as_str()) {
                eprintln!(
                    "Warning: Unknown field '{}' - available fields: {}",
                    field,
                    available_fields.join(", ")
                );
            }
        }

        // Write header
        output.push_str(&fields.join(delimiter));
        output.push('\n');

        // Write data
        for memory in memories {
            let mut row = Vec::new();

            for field in &fields {
                let value = match field.as_str() {
                    "id" => memory.id.clone(),
                    "type" => memory.memory_type.clone(),
                    "topic" => escape_csv_field(&memory.topic, delimiter),
                    "tags" => escape_csv_field(&memory.tags.join("; "), delimiter),
                    "content" => escape_csv_field(&memory.content, delimiter),
                    "examples" => escape_csv_field(&memory.examples.join("; "), delimiter),
                    "reference_count" => memory
                        .reference_count
                        .map_or("".to_string(), |v| v.to_string()),
                    "confidence" => memory
                        .confidence
                        .map_or("".to_string(), |v| format!("{:.2}", v)),
                    "created_at" => memory.created_at.clone().unwrap_or_default(),
                    "last_accessed" => memory.last_accessed.clone().unwrap_or_default(),
                    "source" => memory.source.clone().unwrap_or_default(),
                    "deleted" => memory.deleted.map_or("".to_string(), |v| v.to_string()),
                    _ => "".to_string(),
                };
                row.push(value);
            }

            output.push_str(&row.join(delimiter));
            output.push('\n');
        }

        self.write_output(&output)?;
        Ok(())
    }

    fn write_output(&self, content: &str) -> Result<()> {
        match &self.output {
            Some(path) => {
                let mut file = File::create(path)?;
                file.write_all(content.as_bytes())?;
                if self.verbose {
                    println!("📁 Output written to: {:?}", path);
                }
            }
            None => {
                print!("{}", content);
            }
        }
        Ok(())
    }
}

fn format_timestamp(timestamp: i64) -> String {
    match chrono::DateTime::from_timestamp(timestamp, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => "Invalid timestamp".to_string(),
    }
}

fn escape_csv_field(field: &str, delimiter: &str) -> String {
    if field.contains(delimiter)
        || field.contains('"')
        || field.contains('\n')
        || field.contains('\r')
    {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_field() {
        assert_eq!(escape_csv_field("simple", ","), "simple");
        assert_eq!(escape_csv_field("has, comma", ","), "\"has, comma\"");
        assert_eq!(
            escape_csv_field("has \"quotes\"", ","),
            "\"has \"\"quotes\"\"\""
        );
        assert_eq!(escape_csv_field("has\nnewline", ","), "\"has\nnewline\"");
    }

    #[test]
    fn test_export_format_enum() {
        let formats = [ExportFormat::Json, ExportFormat::Csv];
        assert_eq!(formats.len(), 2);
    }

    #[test]
    fn test_export_memory_serialization() {
        let export_memory = ExportMemory {
            id: "test-id".to_string(),
            memory_type: "tech".to_string(),
            topic: "Test Topic".to_string(),
            tags: vec!["rust".to_string()],
            content: "Test content".to_string(),
            examples: vec!["example".to_string()],
            reference_count: Some(1),
            confidence: Some(0.9),
            created_at: Some("2024-01-01 12:00:00".to_string()),
            last_accessed: None,
            source: None,
            deleted: Some(false),
        };

        let json = serde_json::to_string(&export_memory).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("tech"));
        assert!(json.contains("Test Topic"));
    }
}
