use crate::models::kiro::KiroConfig;
use crate::repositories::memory::SqliteMemoryRepository;
use crate::services::memory::MemoryService;
use anyhow::{Context, Result};

pub async fn execute(type_filter: Option<String>) -> Result<()> {
    // Load configuration
    let config = KiroConfig::load()
        .context("Failed to load configuration. Did you run 'hail-mary init'?")?;

    // Create repository and service
    let repository =
        SqliteMemoryRepository::new(&config).context("Failed to initialize database")?;

    let service = MemoryService::new(repository, config.clone());

    // Validate type filter if provided
    if let Some(ref type_str) = type_filter
        && !config.memory.types.contains(type_str)
    {
        anyhow::bail!(
            "Invalid memory type '{}'. Available types: {}",
            type_str,
            config.memory.types.join(", ")
        );
    }

    println!("Generating memory documentation...");

    // Generate documents
    service
        .generate_documents(&config)
        .await
        .context("Failed to generate documents")?;

    let output_dir = config.memory_docs_dir();

    if let Some(type_filter) = type_filter {
        println!(
            "✅ Generated document for type '{}' in: {}",
            type_filter,
            output_dir.display()
        );
        let file_path = output_dir.join(format!("{}.md", type_filter));
        if file_path.exists() {
            println!("  - {}", file_path.display());
        }
    } else {
        println!("✅ Generated memory documents in: {}", output_dir.display());

        // List generated files
        for memory_type in &config.memory.types {
            let file_path = output_dir.join(format!("{}.md", memory_type));
            if file_path.exists() {
                println!("  - {}", file_path.display());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::memory::{Memory, MemoryType};
    use crate::tests::common::TestDirectory;
    use std::fs;

    async fn setup_service_with_test_data() -> (
        KiroConfig,
        MemoryService<SqliteMemoryRepository>,
        TestDirectory,
    ) {
        let test_dir = TestDirectory::new();
        let kiro_dir = test_dir.path().join(".kiro");
        let memory_dir = kiro_dir.join("memory");
        fs::create_dir_all(&memory_dir).unwrap();

        // Create config
        let config_content = format!(
            r#"
[memory]
types = ["tech", "project-tech", "domain"]
instructions = "Test instructions"

[memory.document]
output_dir = "{}"
format = "markdown"

[memory.database]
path = "{}"
"#,
            memory_dir.join("docs").display(),
            memory_dir.join("test.sqlite3").display()
        );
        fs::write(kiro_dir.join("config.toml"), config_content).unwrap();

        let config = KiroConfig::load().unwrap();
        let repository = SqliteMemoryRepository::new(&config).unwrap();
        let mut service = MemoryService::new(repository, config.clone());

        // Add some test memories
        let memories = vec![
            Memory::new(
                MemoryType::Tech,
                "Rust Programming".to_string(),
                "Rust is a systems programming language.".to_string(),
            )
            .with_tags(vec!["rust".to_string(), "programming".to_string()]),
            Memory::new(
                MemoryType::ProjectTech,
                "Project Setup".to_string(),
                "This project uses Cargo for dependency management.".to_string(),
            )
            .with_tags(vec!["cargo".to_string(), "setup".to_string()]),
            Memory::new(
                MemoryType::Domain,
                "Business Logic".to_string(),
                "The memory system stores technical knowledge.".to_string(),
            )
            .with_tags(vec!["domain".to_string(), "business".to_string()]),
        ];

        let memory_inputs: Vec<_> = memories
            .into_iter()
            .map(|m| crate::services::memory::MemoryInput {
                memory_type: m.memory_type,
                title: m.title,
                content: m.content,
                tags: m.tags,
                confidence: Some(m.confidence),
            })
            .collect();

        service.remember_batch(memory_inputs).await.unwrap();

        (config, service, test_dir)
    }

    #[tokio::test]
    async fn test_document_generates_markdown_files() {
        let (_config, _service, _test_dir) = setup_service_with_test_data().await;

        // Execute document generation
        let result = execute(None).await;

        assert!(
            result.is_ok(),
            "Document generation should succeed: {:?}",
            result
        );

        // Note: The test directory and working directory are automatically managed
        // by TestDirectory's RAII pattern
    }

    #[tokio::test]
    async fn test_document_type_filter() {
        let (_config, _service, _test_dir) = setup_service_with_test_data().await;

        // Test with valid type filter
        let result = execute(Some("tech".to_string())).await;

        assert!(
            result.is_ok(),
            "Document generation with type filter should succeed"
        );
    }

    #[tokio::test]
    async fn test_document_invalid_type_filter() {
        let (_config, _service, _test_dir) = setup_service_with_test_data().await;

        // Test with invalid type filter
        let result = execute(Some("invalid-type".to_string())).await;

        assert!(result.is_err(), "Should fail with invalid type filter");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Invalid memory type"),
            "Should mention invalid type"
        );
    }

    #[tokio::test]
    async fn test_document_fails_without_config() {
        let _test_dir = TestDirectory::new();
        // No .kiro directory created

        let result = execute(None).await;

        assert!(result.is_err(), "Should fail without config");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to load configuration"),
            "Should mention config failure"
        );
    }
}
