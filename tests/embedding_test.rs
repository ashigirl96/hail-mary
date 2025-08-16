use hail_mary::memory::embeddings::{EmbeddingConfig, EmbeddingService};
use hail_mary::memory::models::{Memory, MemoryType};
use hail_mary::memory::repository::{MemoryRepository, SqliteMemoryRepository};
use tempfile::TempDir;

#[tokio::test]
async fn test_embedding_generation() {
    let service = EmbeddingService::new().unwrap();

    let texts = vec![
        "Rust is a systems programming language".to_string(),
        "Python is great for data science".to_string(),
        "JavaScript runs in the browser".to_string(),
    ];

    let embeddings = service.embed_texts(texts).await.unwrap();

    assert_eq!(embeddings.len(), 3);
    assert_eq!(embeddings[0].len(), 384); // Default dimension

    // Check that embeddings are normalized (L2 norm = 1)
    for embedding in &embeddings {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }
}

#[tokio::test]
async fn test_similarity_calculation() {
    let service = EmbeddingService::new().unwrap();

    // Similar texts should have high similarity
    let text1 = "Machine learning is a subset of artificial intelligence".to_string();
    let text2 = "AI and machine learning are related fields".to_string();
    let text3 = "The weather is nice today".to_string();

    let embedding1 = service.embed_text(&text1).await.unwrap();
    let embedding2 = service.embed_text(&text2).await.unwrap();
    let embedding3 = service.embed_text(&text3).await.unwrap();

    let similarity_12 = EmbeddingService::cosine_similarity(&embedding1, &embedding2);
    let similarity_13 = EmbeddingService::cosine_similarity(&embedding1, &embedding3);

    // Similar texts should have higher similarity
    assert!(similarity_12 > similarity_13);
}

#[test]
fn test_embedding_storage() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut repo = SqliteMemoryRepository::new(&db_path).unwrap();

    // Create a memory
    let memory = Memory::new(
        MemoryType::Tech,
        "Test Memory".to_string(),
        "Test content for embedding".to_string(),
    );
    repo.save(&memory).unwrap();

    // Generate and store embedding
    let rt = tokio::runtime::Runtime::new().unwrap();
    let service = EmbeddingService::new().unwrap();
    let embedding = rt.block_on(service.embed_text(&memory.content)).unwrap();

    repo.store_embedding(&memory.id, &embedding, "test-model")
        .unwrap();

    // Retrieve and verify
    let retrieved = repo.get_embedding(&memory.id).unwrap();
    assert!(retrieved.is_some());

    let retrieved_embedding = retrieved.unwrap();
    assert_eq!(retrieved_embedding.len(), embedding.len());

    // Check values match (with small tolerance for float precision)
    for (original, retrieved) in embedding.iter().zip(retrieved_embedding.iter()) {
        assert!((original - retrieved).abs() < 0.0001);
    }
}

#[test]
fn test_find_duplicates() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut repo = SqliteMemoryRepository::new(&db_path).unwrap();

    // Create similar memories
    let memory1 = Memory::new(
        MemoryType::Tech,
        "Rust Programming".to_string(),
        "Rust is a systems programming language focused on safety".to_string(),
    );
    let memory2 = Memory::new(
        MemoryType::Tech,
        "Rust Language".to_string(),
        "Rust programming language is designed for safety and performance".to_string(),
    );
    let memory3 = Memory::new(
        MemoryType::Tech,
        "Python".to_string(),
        "Python is a high-level interpreted language".to_string(),
    );

    repo.save(&memory1).unwrap();
    repo.save(&memory2).unwrap();
    repo.save(&memory3).unwrap();

    // Generate and store embeddings
    let rt = tokio::runtime::Runtime::new().unwrap();
    let service = EmbeddingService::new().unwrap();

    let embedding1 = rt.block_on(service.embed_text(&memory1.content)).unwrap();
    let embedding2 = rt.block_on(service.embed_text(&memory2.content)).unwrap();
    let embedding3 = rt.block_on(service.embed_text(&memory3.content)).unwrap();

    repo.store_embedding(&memory1.id, &embedding1, "test-model")
        .unwrap();
    repo.store_embedding(&memory2.id, &embedding2, "test-model")
        .unwrap();
    repo.store_embedding(&memory3.id, &embedding3, "test-model")
        .unwrap();

    // Find duplicates with a lower threshold (since we're using simple embeddings)
    let duplicates = repo.find_duplicates(0.5).unwrap();

    // We should find that memory1 and memory2 are similar
    assert!(!duplicates.is_empty());

    // The most similar pair should be memory1 and memory2
    let (id1, id2, _similarity) = &duplicates[0];
    assert!(
        (id1 == &memory1.id && id2 == &memory2.id) || (id1 == &memory2.id && id2 == &memory1.id)
    );
}

#[test]
fn test_get_memories_without_embeddings() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let mut repo = SqliteMemoryRepository::new(&db_path).unwrap();

    // Create memories
    let memory_with_embedding = Memory::new(
        MemoryType::Tech,
        "With Embedding".to_string(),
        "This memory has an embedding".to_string(),
    );
    let memory_without_embedding = Memory::new(
        MemoryType::Tech,
        "Without Embedding".to_string(),
        "This memory lacks an embedding".to_string(),
    );

    repo.save(&memory_with_embedding).unwrap();
    repo.save(&memory_without_embedding).unwrap();

    // Generate and store embedding for only one memory
    let rt = tokio::runtime::Runtime::new().unwrap();
    let service = EmbeddingService::new().unwrap();
    let embedding = rt
        .block_on(service.embed_text(&memory_with_embedding.content))
        .unwrap();
    repo.store_embedding(&memory_with_embedding.id, &embedding, "test-model")
        .unwrap();

    // Get memories without embeddings
    let memories_without = repo.get_memories_without_embeddings(10).unwrap();

    assert_eq!(memories_without.len(), 1);
    assert_eq!(memories_without[0].id, memory_without_embedding.id);
}
