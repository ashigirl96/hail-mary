use crate::application::errors::ApplicationError;
use crate::application::repositories::MemoryRepository;

#[derive(Default)]
pub struct ReindexStats {
    pub deleted_entries: usize,
    pub index_rebuilt: bool,
    pub database_optimized: bool,
}

pub fn reindex_memories(
    repository: &mut impl MemoryRepository,
    verbose: bool,
) -> Result<ReindexStats, ApplicationError> {
    let mut stats = ReindexStats::default();

    if verbose {
        println!("Starting database reindex...");
    }

    // Remove logically deleted entries
    stats.deleted_entries = repository.cleanup_deleted()?;

    if verbose {
        println!("Removed {} deleted entries", stats.deleted_entries);
    }

    // Rebuild FTS5 index
    repository.rebuild_fts_index()?;
    stats.index_rebuilt = true;

    if verbose {
        println!("FTS5 index rebuilt successfully");
    }

    // Optimize database
    repository.vacuum()?;
    stats.database_optimized = true;

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::MockMemoryRepository;

    #[test]
    fn test_reindex_memories_basic_operation() {
        let mut repo = MockMemoryRepository::new();
        repo.set_deleted_count(5);

        let result = reindex_memories(&mut repo, false);

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.deleted_entries, 5);
        assert!(stats.index_rebuilt);
        assert!(stats.database_optimized);

        assert!(repo.is_cleanup_called());
        assert!(repo.is_rebuild_called());
        assert!(repo.is_vacuum_called());
    }

    #[test]
    fn test_reindex_memories_verbose_mode() {
        let mut repo = MockMemoryRepository::new();
        repo.set_deleted_count(3);

        let result = reindex_memories(&mut repo, true);

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.deleted_entries, 3);
        assert!(stats.index_rebuilt);
        assert!(stats.database_optimized);
    }

    #[test]
    fn test_reindex_memories_no_deleted_entries() {
        let mut repo = MockMemoryRepository::new();
        repo.set_deleted_count(0);

        let result = reindex_memories(&mut repo, false);

        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.deleted_entries, 0);
        assert!(stats.index_rebuilt);
        assert!(stats.database_optimized);
    }

    #[test]
    fn test_reindex_stats_default() {
        let stats = ReindexStats::default();
        assert_eq!(stats.deleted_entries, 0);
        assert!(!stats.index_rebuilt);
        assert!(!stats.database_optimized);
    }

    #[test]
    fn test_reindex_memories_cleanup_failure() {
        let mut repo = MockMemoryRepository::new();
        repo.set_fail_cleanup(true);

        let result = reindex_memories(&mut repo, false);

        assert!(result.is_err());
        if let Err(ApplicationError::DatabaseError(msg)) = result {
            assert_eq!(msg, "Cleanup failed");
        } else {
            panic!("Expected DatabaseError");
        }
    }
}
