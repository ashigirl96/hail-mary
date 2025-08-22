use anyhow::Result;
use refinery::embed_migrations;
use rusqlite::Connection;

// Embed all migrations from ./migrations directory
embed_migrations!("./migrations");

/// Run database migrations
pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    migrations::runner().run(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::TestDirectory;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_can_be_embedded() {
        // Test that migrations are successfully embedded
        let runner = migrations::runner();

        // Check that we have the expected migrations
        let applied_migrations = runner.get_migrations();
        assert!(
            !applied_migrations.is_empty(),
            "Should have embedded migrations"
        );

        // Verify we have 3 migration files
        assert_eq!(
            applied_migrations.len(),
            3,
            "Should have exactly 3 migrations"
        );

        // Check migration names match expected pattern
        let migration_names: Vec<_> = applied_migrations.iter().map(|m| m.name()).collect();

        assert!(
            migration_names
                .iter()
                .any(|name| name.contains("initial_schema"))
        );
        assert!(
            migration_names
                .iter()
                .any(|name| name.contains("add_fts5_index"))
        );
        assert!(
            migration_names
                .iter()
                .any(|name| name.contains("add_triggers"))
        );
    }

    #[test]
    fn test_run_migrations_creates_tables() {
        // Create temporary database
        let test_dir = TestDirectory::new_no_cd();
        let db_path = test_dir.path().join("test_migrations.db");
        let mut conn = Connection::open(&db_path).unwrap();

        // Run migrations
        run_migrations(&mut conn).expect("Migrations should succeed");

        // Check that memories table exists with correct schema
        let table_info = conn
            .prepare("PRAGMA table_info(memories)")
            .unwrap()
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let type_name: String = row.get(2)?;
                Ok((name, type_name))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(!table_info.is_empty(), "memories table should exist");

        // Check for expected columns
        let column_names: Vec<String> = table_info.iter().map(|(name, _)| name.clone()).collect();

        assert!(column_names.contains(&"id".to_string()));
        assert!(column_names.contains(&"type".to_string()));
        assert!(column_names.contains(&"title".to_string()));
        assert!(column_names.contains(&"content".to_string()));
        assert!(column_names.contains(&"tags".to_string()));
        assert!(column_names.contains(&"confidence".to_string()));
        assert!(column_names.contains(&"reference_count".to_string()));
        assert!(column_names.contains(&"created_at".to_string()));
        assert!(column_names.contains(&"last_accessed".to_string()));
        assert!(column_names.contains(&"deleted".to_string()));

        // Check that memories_fts virtual table exists
        let fts_check = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='memories_fts'")
            .unwrap()
            .query_map([], |row| {
                let name: String = row.get(0)?;
                Ok(name)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(!fts_check.is_empty(), "memories_fts table should exist");

        // Check that indexes exist
        let index_check = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='memories'")
            .unwrap()
            .query_map([], |row| {
                let name: String = row.get(0)?;
                Ok(name)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(
            !index_check.is_empty(),
            "Should have indexes on memories table"
        );

        // Verify specific indexes exist
        let index_names: Vec<String> = index_check;
        assert!(
            index_names
                .iter()
                .any(|name| name.contains("idx_memories_type"))
        );
        assert!(
            index_names
                .iter()
                .any(|name| name.contains("idx_memories_ref_count"))
        );
        assert!(
            index_names
                .iter()
                .any(|name| name.contains("idx_memories_created"))
        );
    }

    #[test]
    fn test_migrations_are_idempotent() {
        // Create temporary database
        let test_dir = TestDirectory::new_no_cd();
        let db_path = test_dir.path().join("test_idempotent.db");
        let mut conn = Connection::open(&db_path).unwrap();

        // Run migrations first time
        run_migrations(&mut conn).expect("First migration run should succeed");

        // Run migrations second time - should not fail
        run_migrations(&mut conn).expect("Second migration run should succeed");

        // Verify table still exists and is functional
        let count: i32 = conn
            .prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='memories'")
            .unwrap()
            .query_row([], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 1, "memories table should exist exactly once");
    }

    #[test]
    fn test_migration_creates_triggers() {
        // Create temporary database
        let test_dir = TestDirectory::new_no_cd();
        let db_path = test_dir.path().join("test_triggers.db");
        let mut conn = Connection::open(&db_path).unwrap();

        // Run migrations
        run_migrations(&mut conn).expect("Migrations should succeed");

        // Check that triggers exist
        let trigger_check = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='trigger'")
            .unwrap()
            .query_map([], |row| {
                let name: String = row.get(0)?;
                Ok(name)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(!trigger_check.is_empty(), "Should have triggers");

        // Check for specific triggers from V003__add_triggers.sql
        let trigger_names: Vec<String> = trigger_check;
        assert!(
            trigger_names
                .iter()
                .any(|name| name.contains("memories_ai"))
        );
        assert!(
            trigger_names
                .iter()
                .any(|name| name.contains("memories_au"))
        );
        assert!(
            trigger_names
                .iter()
                .any(|name| name.contains("memories_soft_delete"))
        );
    }

    #[test]
    fn test_fts_functionality() {
        // Create temporary database
        let test_dir = TestDirectory::new_no_cd();
        let db_path = test_dir.path().join("test_fts.db");
        let mut conn = Connection::open(&db_path).unwrap();

        // Run migrations
        run_migrations(&mut conn).expect("Migrations should succeed");

        // Insert test data
        conn.execute(
            "INSERT INTO memories (id, type, title, content, tags) VALUES (?, ?, ?, ?, ?)",
            &[
                "test-uuid",
                "tech",
                "Test Memory",
                "This is test content",
                "test,memory",
            ],
        )
        .unwrap();

        // Test FTS search
        let search_result: i32 = conn
            .prepare("SELECT COUNT(*) FROM memories_fts WHERE memories_fts MATCH ?")
            .unwrap()
            .query_row(&["test"], |row| row.get(0))
            .unwrap();

        assert!(search_result > 0, "FTS search should find the test memory");
    }
}
