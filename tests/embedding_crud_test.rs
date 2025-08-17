use hail_mary::memory::{
    embeddings::EmbeddingService,
    models::{Memory, MemoryType, RememberParams},
    repository::{MemoryRepository, SqliteMemoryRepository},
    service::MemoryService,
};
use tempfile::TempDir;

#[tokio::test]
async fn test_auto_embedding_on_create() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create service with embeddings enabled
    let repo = SqliteMemoryRepository::new(&db_path).unwrap();
    let mut service = MemoryService::with_embeddings(repo).unwrap();

    // Create a memory
    let params = RememberParams {
        memory_type: MemoryType::Tech,
        title: "Rust Programming".to_string(),
        content: "Rust is a systems programming language focused on safety and performance"
            .to_string(),
        tags: Some(vec!["rust".to_string(), "programming".to_string()]),
        examples: None,
    };

    let response = service.remember(params).await.unwrap();
    assert_eq!(response.action, "created");

    // Verify embedding was generated and stored
    let repo = SqliteMemoryRepository::new(&db_path).unwrap();
    let embedding = repo.get_embedding(&response.memory_id).unwrap();
    assert!(embedding.is_some());

    let embedding = embedding.unwrap();
    assert_eq!(embedding.len(), 384); // Default dimension
}

#[tokio::test]
async fn test_auto_embedding_on_update() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create service with embeddings enabled
    let repo = SqliteMemoryRepository::new(&db_path).unwrap();
    let mut service = MemoryService::with_embeddings(repo).unwrap();

    // Create a memory
    let params = RememberParams {
        memory_type: MemoryType::Tech,
        title: "Python".to_string(),
        content: "Python is a high-level language".to_string(),
        tags: None,
        examples: None,
    };

    let response = service.remember(params).await.unwrap();
    let memory_id = response.memory_id.clone();

    // Get the original embedding
    let repo = SqliteMemoryRepository::new(&db_path).unwrap();
    let original_embedding = repo.get_embedding(&memory_id).unwrap().unwrap();

    // Update the memory with same topic but different content
    let update_params = RememberParams {
        memory_type: MemoryType::Tech,
        title: "Python".to_string(),
        content: "Python is a versatile programming language used for web development, data science, and more".to_string(),
        tags: Some(vec!["python".to_string(), "programming".to_string()]),
        examples: None,
    };

    let response = service.remember(update_params).await.unwrap();
    assert_eq!(response.action, "updated");

    // Verify embedding was regenerated
    let repo = SqliteMemoryRepository::new(&db_path).unwrap();
    let new_embedding = repo.get_embedding(&memory_id).unwrap().unwrap();

    // The embeddings should be different since content changed
    let similarity = EmbeddingService::cosine_similarity(&original_embedding, &new_embedding);
    assert!(similarity < 1.0); // Not identical
    assert!(similarity > 0.5); // But still somewhat similar (same topic)
}

#[tokio::test]
async fn test_semantic_search() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create service with embeddings
    let repo = SqliteMemoryRepository::new(&db_path).unwrap();
    let mut service = MemoryService::with_embeddings(repo).unwrap();

    // Create several memories
    let memories = vec![
        (
            "Machine Learning",
            "Neural networks and deep learning algorithms",
        ),
        (
            "Artificial Intelligence",
            "AI systems that can learn and adapt",
        ),
        (
            "Web Development",
            "Building websites with HTML, CSS, and JavaScript",
        ),
        (
            "Database Systems",
            "Storing and retrieving data efficiently",
        ),
    ];

    for (topic, content) in memories {
        let params = RememberParams {
            memory_type: MemoryType::Tech,
            title: topic.to_string(),
            content: content.to_string(),
            tags: None,
            examples: None,
        };
        service.remember(params).await.unwrap();
    }

    // Search semantically for "deep learning"
    let results = service
        .recall_semantic("deep learning", 2, 0.3)
        .await
        .unwrap();

    assert!(!results.is_empty());
    // The first result should be about machine learning
    assert!(
        results[0].0.title.contains("Machine Learning")
            || results[0].0.title.contains("Artificial Intelligence")
    );
}

#[tokio::test]
async fn test_find_related_memories() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create service with embeddings
    let repo = SqliteMemoryRepository::new(&db_path).unwrap();
    let mut service = MemoryService::with_embeddings(repo).unwrap();

    // Create related memories
    let rust_params = RememberParams {
        memory_type: MemoryType::Tech,
        title: "Rust Language".to_string(),
        content: "Rust is a systems programming language with memory safety".to_string(),
        tags: None,
        examples: None,
    };
    let rust_response = service.remember(rust_params).await.unwrap();

    let cpp_params = RememberParams {
        memory_type: MemoryType::Tech,
        title: "C++ Language".to_string(),
        content: "C++ is a systems programming language with manual memory management".to_string(),
        tags: None,
        examples: None,
    };
    service.remember(cpp_params).await.unwrap();

    let python_params = RememberParams {
        memory_type: MemoryType::Tech,
        title: "Python Language".to_string(),
        content: "Python is a high-level interpreted language".to_string(),
        tags: None,
        examples: None,
    };
    service.remember(python_params).await.unwrap();

    // Find memories related to Rust
    let related = service
        .find_related(&rust_response.memory_id, 2, 0.3)
        .await
        .unwrap();

    assert!(!related.is_empty());
    // C++ should be more related to Rust than Python (both are systems languages)
    let cpp_found = related.iter().any(|(m, _)| m.title.contains("C++"));
    assert!(cpp_found);
}

#[tokio::test]
async fn test_reindex_with_embeddings() {
    use hail_mary::memory::reindex::{ReindexConfig, ReindexService};

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create repository and add memories
    let mut repo = SqliteMemoryRepository::new(&db_path).unwrap();

    for i in 0..5 {
        let memory = Memory::new(
            MemoryType::Tech,
            format!("Memory {}", i),
            format!("Content for memory {}", i),
        );
        repo.save(&memory).unwrap();
    }

    // Run reindex with embedding generation
    let config = ReindexConfig {
        generate_embeddings: true,
        embedding_batch_size: 2,
        backup_enabled: false,
        verbose: true,
        ..Default::default()
    };

    let service = ReindexService::new(config).unwrap();
    let result = service.reindex_with_embeddings(&db_path).await.unwrap();

    assert_eq!(result.total_memories, 5);

    // Verify all memories have embeddings
    let repo = SqliteMemoryRepository::new(&db_path).unwrap();
    let memories_without_embeddings = repo.get_memories_without_embeddings(10).unwrap();
    assert_eq!(memories_without_embeddings.len(), 0);
}
