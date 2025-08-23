pub mod memory_repository;
pub mod project_repository;

// Re-export traits for easier access
pub use memory_repository::MemoryRepository;
pub use project_repository::ProjectRepository;
