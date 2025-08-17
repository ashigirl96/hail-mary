use crate::mcp::server::get_default_db_path;
use crate::memory::{
    models::{Memory, MemoryType},
    repository::{MemoryRepository, SqliteMemoryRepository},
};
use crate::utils::error::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Import memories from JSON or CSV format
#[derive(Args)]
pub struct ImportCommand {
    /// Input file path
    #[arg(value_name = "PATH")]
    pub input: PathBuf,

    /// Input format (auto-detected from file extension if not specified)
    #[arg(long, value_enum)]
    pub format: Option<ImportFormat>,

    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Dry run - show what would be imported without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Update existing memories if ID matches
    #[arg(long)]
    pub update_existing: bool,

    /// Skip memories with duplicate content
    #[arg(long)]
    pub skip_duplicates: bool,

    /// CSV delimiter (only for CSV format)
    #[arg(long, default_value = ",")]
    pub csv_delimiter: String,

    /// Set default confidence for imported memories without confidence
    #[arg(long, value_name = "SCORE", default_value = "0.8")]
    pub default_confidence: f32,

    /// Set default memory type for imported memories without type
    #[arg(long, value_enum, default_value = "tech")]
    pub default_type: MemoryType,

    /// Enable verbose output
    #[arg(long, short)]
    pub verbose: bool,

    /// Force import even with validation warnings
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ImportFormat {
    Json,
    Csv,
}

/// Import-specific memory representation for JSON input
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportMemory {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type", default)]
    pub memory_type: Option<String>,
    pub topic: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub content: String,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub reference_count: Option<u32>,
    #[serde(default)]
    pub confidence: Option<f32>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_accessed: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub deleted: Option<bool>,
}

#[derive(Debug)]
pub struct ImportResult {
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

impl ImportCommand {
    /// Execute the import command
    pub fn execute(self) -> Result<()> {
        // Validate input
        if !self.input.exists() {
            eprintln!("Error: Input file does not exist: {:?}", self.input);
            return Ok(());
        }

        if self.default_confidence < 0.0 || self.default_confidence > 1.0 {
            eprintln!("Error: Default confidence score must be between 0.0 and 1.0");
            return Ok(());
        }

        if self.csv_delimiter.len() != 1 {
            eprintln!("Error: CSV delimiter must be a single character");
            return Ok(());
        }

        // Determine format
        let format = self.determine_format()?;

        if self.verbose {
            println!("📥 Import Configuration:");
            println!("  Input: {:?}", self.input);
            println!("  Format: {:?}", format);
            println!("  Dry run: {}", self.dry_run);
            println!("  Update existing: {}", self.update_existing);
            println!("  Skip duplicates: {}", self.skip_duplicates);
            println!("  Default confidence: {:.2}", self.default_confidence);
            println!("  Default type: {:?}", self.default_type);
            println!();
        }

        // Parse input file
        let import_memories = match format {
            ImportFormat::Json => self.parse_json()?,
            ImportFormat::Csv => self.parse_csv()?,
        };

        if self.verbose {
            println!(
                "📄 Parsed {} memories from input file",
                import_memories.len()
            );
        }

        // Validate memories
        let (valid_memories, validation_warnings) = self.validate_memories(import_memories)?;

        if !validation_warnings.is_empty() {
            println!("⚠️  Validation Warnings:");
            for warning in &validation_warnings {
                println!("  - {}", warning);
            }

            if !self.force {
                println!();
                println!("Use --force to import despite warnings, or fix the issues above.");
                return Ok(());
            }
            println!();
        }

        if self.dry_run {
            println!("🔍 Dry Run - No changes will be made:");
            self.preview_import(&valid_memories)?;
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

        let mut repository = SqliteMemoryRepository::new(&db_path)?;

        // Import memories
        let result = self.import_memories(&mut repository, valid_memories)?;

        // Display results
        self.display_results(&result)?;

        Ok(())
    }

    fn determine_format(&self) -> Result<ImportFormat> {
        if let Some(format) = &self.format {
            return Ok(format.clone());
        }

        // Auto-detect from file extension
        match self.input.extension().and_then(|ext| ext.to_str()) {
            Some("json") => Ok(ImportFormat::Json),
            Some("csv") => Ok(ImportFormat::Csv),
            _ => {
                eprintln!(
                    "Error: Cannot auto-detect format. Please specify --format json or --format csv"
                );
                Ok(ImportFormat::Json) // Default fallback
            }
        }
    }

    fn parse_json(&self) -> Result<Vec<ImportMemory>> {
        let mut file = File::open(&self.input)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let memories: Vec<ImportMemory> = serde_json::from_str(&contents)?;
        Ok(memories)
    }

    fn parse_csv(&self) -> Result<Vec<ImportMemory>> {
        let mut file = File::open(&self.input)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut memories = Vec::new();
        let lines: Vec<&str> = contents.lines().collect();

        if lines.is_empty() {
            return Ok(memories);
        }

        // Parse header
        let header_line = lines[0];
        let headers: Vec<&str> = header_line.split(&self.csv_delimiter).collect();

        // Validate required columns
        let required_fields = ["topic", "content"];
        for field in &required_fields {
            if !headers.iter().any(|h| h.trim() == *field) {
                eprintln!("Error: CSV missing required field: {}", field);
                return Ok(Vec::new());
            }
        }

        // Parse data rows
        for line in lines.iter().skip(1) {
            if line.trim().is_empty() {
                continue;
            }

            let values: Vec<&str> = line.split(&self.csv_delimiter).collect();
            if values.len() != headers.len() {
                eprintln!("Warning: Skipping malformed CSV row: {}", line);
                continue;
            }

            let mut memory = ImportMemory {
                id: None,
                memory_type: None,
                topic: String::new(),
                tags: Vec::new(),
                content: String::new(),
                examples: Vec::new(),
                reference_count: None,
                confidence: None,
                created_at: None,
                last_accessed: None,
                source: None,
                deleted: None,
            };

            // Map CSV columns to memory fields
            for (i, header) in headers.iter().enumerate() {
                if i >= values.len() {
                    continue;
                }

                let value = values[i].trim();
                // Remove quotes if present
                let value = if value.starts_with('"') && value.ends_with('"') && value.len() > 1 {
                    &value[1..value.len() - 1].replace("\"\"", "\"")
                } else {
                    value
                };

                match header.trim() {
                    "id" => {
                        memory.id = if value.is_empty() {
                            None
                        } else {
                            Some(value.to_string())
                        }
                    }
                    "type" => {
                        memory.memory_type = if value.is_empty() {
                            None
                        } else {
                            Some(value.to_string())
                        }
                    }
                    "topic" => memory.topic = value.to_string(),
                    "content" => memory.content = value.to_string(),
                    "tags" => {
                        if !value.is_empty() {
                            memory.tags = value.split(';').map(|s| s.trim().to_string()).collect();
                        }
                    }
                    "examples" => {
                        if !value.is_empty() {
                            memory.examples =
                                value.split(';').map(|s| s.trim().to_string()).collect();
                        }
                    }
                    "reference_count" => {
                        memory.reference_count = value.parse().ok();
                    }
                    "confidence" => {
                        memory.confidence = value.parse().ok();
                    }
                    "source" => {
                        memory.source = if value.is_empty() {
                            None
                        } else {
                            Some(value.to_string())
                        }
                    }
                    _ => {} // Ignore unknown columns
                }
            }

            memories.push(memory);
        }

        Ok(memories)
    }

    fn validate_memories(&self, memories: Vec<ImportMemory>) -> Result<(Vec<Memory>, Vec<String>)> {
        let mut valid_memories = Vec::new();
        let mut warnings = Vec::new();

        for (i, import_memory) in memories.into_iter().enumerate() {
            // Validate required fields
            if import_memory.topic.trim().is_empty() {
                warnings.push(format!("Memory #{}: Missing topic", i + 1));
                continue;
            }

            if import_memory.content.trim().is_empty() {
                warnings.push(format!("Memory #{}: Missing content", i + 1));
                continue;
            }

            // Determine memory type
            let memory_type = if let Some(type_str) = &import_memory.memory_type {
                MemoryType::from_str(type_str).unwrap_or_else(|| {
                    warnings.push(format!(
                        "Memory #{}: Invalid type '{}', using default",
                        i + 1,
                        type_str
                    ));
                    self.default_type.clone()
                })
            } else {
                self.default_type.clone()
            };

            // Validate confidence
            let confidence = import_memory.confidence.unwrap_or(self.default_confidence);
            if !(0.0..=1.0).contains(&confidence) {
                warnings.push(format!(
                    "Memory #{}: Invalid confidence {:.2}, using default",
                    i + 1,
                    confidence
                ));
            }

            // Create Memory from ImportMemory
            let memory = Memory {
                id: import_memory
                    .id
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                memory_type,
                topic: import_memory.topic,
                tags: import_memory.tags,
                content: import_memory.content,
                examples: import_memory.examples,
                reference_count: import_memory.reference_count.unwrap_or(0),
                confidence: confidence.clamp(0.0, 1.0),
                created_at: chrono::Utc::now().timestamp(),
                last_accessed: None,
                source: import_memory.source,
                deleted: import_memory.deleted.unwrap_or(false),
            };

            valid_memories.push(memory);
        }

        Ok((valid_memories, warnings))
    }

    fn preview_import(&self, memories: &[Memory]) -> Result<()> {
        println!("Would import {} memories:", memories.len());

        let mut type_counts = std::collections::HashMap::new();
        for memory in memories {
            *type_counts.entry(&memory.memory_type).or_insert(0) += 1;
        }

        for (mem_type, count) in &type_counts {
            println!("  - {}: {} memories", mem_type, count);
        }

        if self.verbose && memories.len() <= 10 {
            println!("\nMemory previews:");
            for (i, memory) in memories.iter().enumerate() {
                println!("{}. {} [{}]", i + 1, memory.topic, memory.memory_type);
                if !memory.tags.is_empty() {
                    println!("   Tags: {}", memory.tags.join(", "));
                }
                let preview = if memory.content.len() > 100 {
                    format!("{}...", &memory.content[..100])
                } else {
                    memory.content.clone()
                };
                println!("   Content: {}", preview);
                println!();
            }
        }

        Ok(())
    }

    fn import_memories(
        &self,
        repository: &mut SqliteMemoryRepository,
        memories: Vec<Memory>,
    ) -> Result<ImportResult> {
        let mut result = ImportResult {
            created: 0,
            updated: 0,
            skipped: 0,
            errors: Vec::new(),
        };

        for memory in memories {
            if self.skip_duplicates {
                // Check for duplicate content
                match repository.search(&memory.content, 10) {
                    Ok(existing) => {
                        if existing.iter().any(|m| m.content == memory.content) {
                            if self.verbose {
                                println!("Skipping duplicate: {}", memory.topic);
                            }
                            result.skipped += 1;
                            continue;
                        }
                    }
                    Err(_) => {
                        // Continue with import if search fails
                    }
                }
            }

            // Check if memory exists (by ID)
            if self.update_existing {
                match repository.get_by_id(&memory.id) {
                    Ok(Some(_)) => {
                        // Update existing memory
                        match repository.update_memory(&memory) {
                            Ok(_) => {
                                if self.verbose {
                                    println!("Updated: {}", memory.topic);
                                }
                                result.updated += 1;
                            }
                            Err(e) => {
                                let error_msg =
                                    format!("Failed to update '{}': {}", memory.topic, e);
                                result.errors.push(error_msg);
                            }
                        }
                        continue;
                    }
                    Ok(None) => {
                        // Memory doesn't exist, create new
                    }
                    Err(e) => {
                        let error_msg =
                            format!("Failed to check existing memory '{}': {}", memory.topic, e);
                        result.errors.push(error_msg);
                        continue;
                    }
                }
            }

            // Create new memory
            match repository.create_memory(&memory) {
                Ok(_) => {
                    if self.verbose {
                        println!("Created: {}", memory.topic);
                    }
                    result.created += 1;
                }
                Err(e) => {
                    let error_msg = format!("Failed to create '{}': {}", memory.topic, e);
                    result.errors.push(error_msg);
                }
            }
        }

        Ok(result)
    }

    fn display_results(&self, result: &ImportResult) -> Result<()> {
        println!("✅ Import completed!");
        println!("  Created: {}", result.created);
        println!("  Updated: {}", result.updated);
        println!("  Skipped: {}", result.skipped);

        if !result.errors.is_empty() {
            println!("  Errors: {}", result.errors.len());
            if self.verbose {
                println!("\nError details:");
                for error in &result.errors {
                    println!("  - {}", error);
                }
            }
        }

        println!(
            "\nTotal processed: {}",
            result.created + result.updated + result.skipped + result.errors.len()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_import_format_detection() {
        let cmd = ImportCommand {
            input: PathBuf::from("test.json"),
            format: None,
            db_path: None,
            dry_run: true,
            update_existing: false,
            skip_duplicates: false,
            csv_delimiter: ",".to_string(),
            default_confidence: 0.8,
            default_type: MemoryType::Tech,
            verbose: false,
            force: false,
        };

        let format = cmd.determine_format().unwrap();
        assert!(matches!(format, ImportFormat::Json));
    }

    #[test]
    fn test_import_memory_validation() {
        let cmd = ImportCommand {
            input: PathBuf::from("test.json"),
            format: Some(ImportFormat::Json),
            db_path: None,
            dry_run: true,
            update_existing: false,
            skip_duplicates: false,
            csv_delimiter: ",".to_string(),
            default_confidence: 0.8,
            default_type: MemoryType::Tech,
            verbose: false,
            force: false,
        };

        let import_memories = vec![ImportMemory {
            id: None,
            memory_type: Some("tech".to_string()),
            topic: "Test Topic".to_string(),
            tags: vec!["rust".to_string()],
            content: "Test content".to_string(),
            examples: vec![],
            reference_count: None,
            confidence: Some(0.9),
            created_at: None,
            last_accessed: None,
            source: None,
            deleted: None,
        }];

        let (valid_memories, warnings) = cmd.validate_memories(import_memories).unwrap();
        assert_eq!(valid_memories.len(), 1);
        assert!(warnings.is_empty());
        assert_eq!(valid_memories[0].topic, "Test Topic");
        assert_eq!(valid_memories[0].confidence, 0.9);
    }

    #[test]
    fn test_csv_parsing() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.csv");

        let csv_content = r#"id,type,topic,tags,content,confidence
test-1,tech,"Rust Async","rust;async","Async programming in Rust",0.9
test-2,domain,"Database Design","database;sql","Database design principles",0.8"#;

        let mut file = File::create(&file_path).unwrap();
        file.write_all(csv_content.as_bytes()).unwrap();

        let cmd = ImportCommand {
            input: file_path,
            format: Some(ImportFormat::Csv),
            db_path: None,
            dry_run: true,
            update_existing: false,
            skip_duplicates: false,
            csv_delimiter: ",".to_string(),
            default_confidence: 0.8,
            default_type: MemoryType::Tech,
            verbose: false,
            force: false,
        };

        let memories = cmd.parse_csv().unwrap();
        assert_eq!(memories.len(), 2);
        assert_eq!(memories[0].topic, "Rust Async");
        assert_eq!(memories[0].tags, vec!["rust", "async"]);
        assert_eq!(memories[0].confidence, Some(0.9));
    }
}
