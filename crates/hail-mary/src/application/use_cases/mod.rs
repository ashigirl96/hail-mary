pub mod complete_features;
pub mod create_feature;
pub mod generate_document;
pub mod initialize_project;
pub mod launch_claude_with_spec;
pub mod recall_memory;
pub mod reindex_memories;
pub mod remember_memory;

// Re-export use case functions for easier access
pub use complete_features::complete_features;
pub use create_feature::create_feature;
pub use generate_document::generate_document;
pub use initialize_project::initialize_project;
pub use launch_claude_with_spec::launch_claude_with_spec;
pub use recall_memory::recall_memory;
pub use reindex_memories::{ReindexStats, reindex_memories};
pub use remember_memory::{RememberRequest, remember_memory};
