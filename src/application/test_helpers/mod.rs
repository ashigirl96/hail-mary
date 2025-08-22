//! Test helper modules for application layer testing
//!
//! This module provides mock implementations of repositories for testing purposes.

#[cfg(test)]
pub mod mock_memory_repository;

#[cfg(test)]
pub mod mock_project_repository;

#[cfg(test)]
pub use mock_memory_repository::MockMemoryRepository;

#[cfg(test)]
pub use mock_project_repository::MockProjectRepository;
