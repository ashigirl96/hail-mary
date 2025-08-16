use anyhow::Result;
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

/// データベースマイグレーション定義
pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // 初期スキーマ作成
        M::up(include_str!("../../migrations/001_initial_schema.sql")),
        // ベクトルストレージ追加
        M::up(include_str!("../../migrations/002_vector_storage.sql")),
        // エンベディングインデックス追加
        M::up(include_str!("../../migrations/003_embedding_index.sql")),
    ])
}

/// データベース初期化
pub fn initialize_database(conn: &mut Connection) -> Result<()> {
    migrations().to_latest(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();
        initialize_database(&mut conn).unwrap();

        // テーブルが作成されたことを確認
        let table_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='memories'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 1);

        // FTS5テーブルが作成されたことを確認
        let fts_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name LIKE 'memories_fts%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(fts_count > 0);
    }
}
