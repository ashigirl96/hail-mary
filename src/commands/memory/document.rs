use crate::mcp::server::get_default_db_path;
use crate::memory::{
    models::{Memory, MemoryType},
    repository::SqliteMemoryRepository,
    service::MemoryService,
};
use crate::utils::error::Result;
use clap::Args;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Generate documentation from stored memories
#[derive(Args)]
pub struct DocumentCommand {
    /// Output directory for generated documentation (defaults to ./memory-docs)
    #[arg(long, short, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Generate documentation only for specific memory type
    #[arg(long, value_name = "TYPE")]
    pub r#type: Option<String>,

    /// Path to the database file (defaults to ~/.local/share/hail-mary/memory.db)
    #[arg(long, value_name = "PATH")]
    pub db_path: Option<PathBuf>,

    /// Enable verbose output
    #[arg(long, short)]
    pub verbose: bool,
}

impl DocumentCommand {
    /// Execute the document command
    pub fn execute(self) -> Result<()> {
        // Determine database path
        let db_path = self
            .db_path
            .unwrap_or_else(|| get_default_db_path().expect("Failed to get default database path"));

        // If database doesn't exist, create it
        if !db_path.exists() {
            if self.verbose {
                println!("Database not found, creating new one at {:?}", db_path);
            }
            // Ensure parent directory exists
            if let Some(parent) = db_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }

        // Create output directory
        let output_dir = self.output.unwrap_or_else(|| PathBuf::from("memory-docs"));
        fs::create_dir_all(&output_dir)?;

        if self.verbose {
            println!("Using database: {:?}", db_path);
            println!("Output directory: {:?}", output_dir);
        }

        // Create runtime for async operations
        let runtime = tokio::runtime::Runtime::new()?;

        runtime.block_on(async {
            // Initialize repository and service
            let repository = SqliteMemoryRepository::new(&db_path)?;
            let service = MemoryService::new(repository);

            // Determine which types to generate
            let types_to_generate = if let Some(type_str) = self.r#type {
                match MemoryType::from_str(&type_str) {
                    Some(memory_type) => vec![memory_type],
                    None => {
                        eprintln!(
                            "Invalid memory type: {}. Valid types are: tech, project-tech, domain",
                            type_str
                        );
                        return Ok(());
                    }
                }
            } else {
                vec![
                    MemoryType::Tech,
                    MemoryType::ProjectTech,
                    MemoryType::Domain,
                ]
            };

            // Generate documentation for each type
            for memory_type in types_to_generate {
                let filename = format!("{}.md", memory_type);
                let file_path = output_dir.join(&filename);

                if self.verbose {
                    println!("Generating {}...", filename);
                }

                // Get all memories of this type
                let mut memories = service.get_all_by_type(&memory_type).await?;

                // Sort by confidence and reference_count
                memories.sort_by(|a, b| {
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap()
                        .then(b.reference_count.cmp(&a.reference_count))
                });

                // Generate markdown content
                let content = generate_markdown(&memory_type, &memories);

                // Write to file
                let mut file = fs::File::create(&file_path)?;
                file.write_all(content.as_bytes())?;

                println!("✅ Generated {}", file_path.display());
            }

            println!("\n📚 Documentation generation complete!");
            println!("Files are available in: {}", output_dir.display());

            Ok(())
        })
    }
}

/// Generate markdown content for a memory type
fn generate_markdown(memory_type: &MemoryType, memories: &[Memory]) -> String {
    let mut content = String::new();

    // Add header
    let title = match memory_type {
        MemoryType::Tech => "Technical Knowledge",
        MemoryType::ProjectTech => "Project Technical Standards",
        MemoryType::Domain => "Domain Knowledge",
    };

    content.push_str(&format!("# {}\n\n", title));

    if memories.is_empty() {
        content.push_str("*No memories recorded yet.*\n");
        return content;
    }

    content.push_str(&format!("*Total memories: {}*\n\n", memories.len()));
    content.push_str("---\n\n");

    // Add each memory
    for memory in memories {
        // Topic header
        content.push_str(&format!("## {}\n", memory.title));

        // Metadata
        let tags = if memory.tags.is_empty() {
            String::from("none")
        } else {
            memory.tags.join(", ")
        };

        content.push_str(&format!(
            "*Tags: {} | References: {} | Confidence: {:.2}*\n\n",
            tags, memory.reference_count, memory.confidence
        ));

        // Content
        content.push_str(&memory.content);
        content.push_str("\n\n");

        // Examples (if any)
        if !memory.examples.is_empty() {
            content.push_str("### Examples:\n");
            for example in &memory.examples {
                // Detect if it's code (simple heuristic)
                if example.contains('\n') || example.contains('{') || example.contains('(') {
                    content.push_str("```\n");
                    content.push_str(example);
                    if !example.ends_with('\n') {
                        content.push('\n');
                    }
                    content.push_str("```\n");
                } else {
                    content.push_str(&format!("- {}\n", example));
                }
            }
            content.push('\n');
        }

        // Source field removed

        // Separator
        content.push_str("---\n\n");
    }

    content
}
