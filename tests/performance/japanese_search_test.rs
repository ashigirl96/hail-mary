use anyhow::{Context, Result};
use hail_mary::models::kiro::KiroConfig;
use hail_mary::models::memory::MemoryType;
use hail_mary::repositories::memory::{MemoryRepository, SqliteMemoryRepository};
use hail_mary::services::memory::{MemoryInput, MemoryService};
use std::collections::HashMap;
use tempfile::TempDir;

/// Tests for Japanese text search quality and FTS5 tokenization
/// Validates that FTS5 with 'porter unicode61' tokenizer properly handles Japanese content

#[tokio::test]
async fn test_japanese_search_precision() -> Result<()> {
    let (_temp_dir, mut service) = setup_japanese_search_service().await?;
    
    // Populate with Japanese content
    populate_japanese_memories(&mut service).await?;
    
    // Test Japanese term searches with expected precision
    let test_cases = vec![
        ("Rust", vec!["Rustの非同期プログラミング"], "Programming language search"),
        ("非同期", vec!["Rustの非同期プログラミング", "JavaScript非同期処理"], "Japanese hiragana search"),
        ("プログラミング", vec!["Rustの非同期プログラミング"], "Japanese katakana search"),
        ("データベース", vec!["SQLite データベース設計"], "Database term search"),
        ("SQLite", vec!["SQLite データベース設計"], "English term in Japanese content"),
    ];
    
    for (query, expected_titles, description) in test_cases {
        let results = service.recall(query, 10, None, vec![]).await?;
        
        // Parse markdown results to extract titles
        let found_titles = extract_titles_from_markdown(&results);
        
        // Check precision: all expected titles should be found
        let precision = calculate_precision(&found_titles, &expected_titles);
        assert!(
            precision >= 0.8,
            "{}: precision {:.2} (found: {:?}, expected: {:?})",
            description,
            precision,
            found_titles,
            expected_titles
        );
        
        println!("{}: precision {:.2}, found {} results", description, precision, found_titles.len());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_mixed_language_search() -> Result<()> {
    let (_temp_dir, mut service) = setup_japanese_search_service().await?;
    
    populate_mixed_language_memories(&mut service).await?;
    
    // Test searches that span multiple languages
    let mixed_queries = vec![
        ("async", "Should find both English and Japanese async content"),
        ("database", "Should find database content in both languages"),
        ("プログラミング async", "Mixed Japanese-English query"),
        ("Rust 非同期", "Mixed English-Japanese query"),
    ];
    
    for (query, description) in mixed_queries {
        let results = service.recall(query, 10, None, vec![]).await?;
        let found_titles = extract_titles_from_markdown(&results);
        
        assert!(
            !found_titles.is_empty(),
            "{}: should find results for mixed language query '{}'",
            description,
            query
        );
        
        // Verify that results contain relevant content
        let has_relevant = found_titles.iter().any(|title| {
            title.to_lowercase().contains("async") || 
            title.contains("非同期") ||
            title.to_lowercase().contains("rust") ||
            title.contains("プログラミング")
        });
        
        assert!(
            has_relevant,
            "{}: results should contain relevant content for '{}'",
            description,
            query
        );
        
        println!("{}: found {} relevant results", description, found_titles.len());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_japanese_character_types() -> Result<()> {
    let (_temp_dir, mut service) = setup_japanese_search_service().await?;
    
    // Create memories with different Japanese character types
    let character_type_memories = vec![
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "ひらがな検索テスト".to_string(),
            content: "ひらがなのみのコンテンツです。あいうえお、かきくけこ。".to_string(),
            tags: vec!["ひらがな".to_string(), "テスト".to_string()],
            confidence: Some(0.9),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "カタカナ検索テスト".to_string(),
            content: "カタカナコンテンツ：プログラミング、データベース、アルゴリズム。".to_string(),
            tags: vec!["カタカナ".to_string(), "プログラミング".to_string()],
            confidence: Some(0.9),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "漢字検索試験".to_string(),
            content: "漢字を含む文章。技術文書、設計仕様、開発手順。".to_string(),
            tags: vec!["漢字".to_string(), "技術".to_string()],
            confidence: Some(0.9),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "混合文字検索Test".to_string(),
            content: "ひらがな、カタカナ、漢字、English混合のコンテンツです。Rustプログラミング技術文書。".to_string(),
            tags: vec!["混合".to_string(), "mixed".to_string()],
            confidence: Some(0.9),
        },
    ];
    
    service.remember_batch(character_type_memories).await?;
    
    // Test search for each character type
    let character_tests = vec![
        ("ひらがな", "Hiragana search"),
        ("カタカナ", "Katakana search"),
        ("プログラミング", "Katakana programming term"),
        ("漢字", "Kanji search"),
        ("技術", "Kanji technical term"),
        ("混合", "Mixed content search"),
        ("Rust", "English in mixed content"),
    ];
    
    for (query, description) in character_tests {
        let results = service.recall(query, 10, None, vec![]).await?;
        let found_titles = extract_titles_from_markdown(&results);
        
        assert!(
            !found_titles.is_empty(),
            "{}: should find results for '{}'",
            description,
            query
        );
        
        println!("{}: found {} results for '{}'", description, found_titles.len(), query);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_japanese_search_recall_accuracy() -> Result<()> {
    let (_temp_dir, mut service) = setup_japanese_search_service().await?;
    
    // Create a comprehensive set of Japanese memories
    populate_comprehensive_japanese_content(&mut service).await?;
    
    // Test recall (how many relevant documents are retrieved)
    let recall_tests = vec![
        ("Rust", 3, "Should find all Rust-related content"),
        ("非同期", 2, "Should find all async-related content"),
        ("データベース", 2, "Should find all database content"),
        ("プログラミング", 3, "Should find all programming content"),
    ];
    
    for (query, expected_min, description) in recall_tests {
        let results = service.recall(query, 20, None, vec![]).await?; // Increase limit to catch all
        let found_titles = extract_titles_from_markdown(&results);
        
        assert!(
            found_titles.len() >= expected_min,
            "{}: found {} results, expected at least {}",
            description,
            found_titles.len(),
            expected_min
        );
        
        // Calculate recall percentage
        let recall = found_titles.len() as f64 / expected_min as f64;
        println!("{}: recall {:.2} ({}/{})", description, recall, found_titles.len(), expected_min);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_fts5_japanese_tokenization_edge_cases() -> Result<()> {
    let (_temp_dir, mut service) = setup_japanese_search_service().await?;
    
    // Test edge cases for Japanese tokenization
    let edge_case_memories = vec![
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "句読点テスト、検索。".to_string(),
            content: "句読点（、。）を含む日本語テキスト。検索精度への影響を検証。".to_string(),
            tags: vec!["句読点".to_string(), "検索".to_string()],
            confidence: Some(0.9),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "記号!@#$%^&*()含有テスト".to_string(),
            content: "特殊記号を含むテキスト!@#$%。検索時の処理確認。".to_string(),
            tags: vec!["記号".to_string(), "特殊".to_string()],
            confidence: Some(0.9),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "数字123混合テスト456".to_string(),
            content: "数字123と日本語456の混合テキスト。789検索テスト。".to_string(),
            tags: vec!["数字".to_string(), "混合".to_string()],
            confidence: Some(0.9),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "長い文章検索テスト".to_string(),
            content: "非常に長い日本語の文章でのFTS5検索性能をテストします。複数の段落にわたる内容で、様々なキーワードが含まれています。プログラミング言語としてのRustの特徴、非同期プログラミングの概念、データベース設計の原則、ユーザーインターフェースの設計思想、パフォーマンス最適化の手法、セキュリティ対策の重要性について説明します。".to_string(),
            tags: vec!["長文".to_string(), "テスト".to_string(), "複合".to_string()],
            confidence: Some(0.9),
        },
    ];
    
    service.remember_batch(edge_case_memories).await?;
    
    // Test edge case searches
    let edge_case_queries = vec![
        ("句読点", "Punctuation in Japanese"),
        ("検索", "Common word with punctuation"),
        ("記号", "Special characters"),
        ("数字", "Numbers in Japanese"),
        ("長い", "Long content search"),
        ("プログラミング", "Word in long content"),
    ];
    
    for (query, description) in edge_case_queries {
        let results = service.recall(query, 10, None, vec![]).await?;
        let found_titles = extract_titles_from_markdown(&results);
        
        // Should find at least one result for each edge case
        assert!(
            !found_titles.is_empty(),
            "{}: should handle edge case for '{}'",
            description,
            query
        );
        
        println!("{}: found {} results", description, found_titles.len());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_search_performance_japanese_vs_english() -> Result<()> {
    let (_temp_dir, mut service) = setup_japanese_search_service().await?;
    
    // Populate with mixed content
    populate_mixed_language_memories(&mut service).await?;
    populate_japanese_memories(&mut service).await?;
    
    // Compare search performance between Japanese and English queries
    let performance_tests = vec![
        ("rust", "English single term"),
        ("programming", "English multi-syllable"),
        ("ラスト", "Japanese katakana"),
        ("プログラミング", "Japanese katakana long"),
        ("非同期", "Japanese hiragana"),
        ("技術文書", "Japanese kanji compound"),
    ];
    
    for (query, description) in performance_tests {
        let start = std::time::Instant::now();
        let results = service.recall(query, 10, None, vec![]).await?;
        let elapsed = start.elapsed();
        
        // Search should be fast regardless of language
        assert!(
            elapsed.as_millis() < 200,
            "{} search took {}ms, expected < 200ms",
            description,
            elapsed.as_millis()
        );
        
        let found_titles = extract_titles_from_markdown(&results);
        println!(
            "{}: {}ms, {} results",
            description,
            elapsed.as_millis(),
            found_titles.len()
        );
    }
    
    Ok(())
}

// Helper functions

async fn setup_japanese_search_service() -> Result<(TempDir, MemoryService<SqliteMemoryRepository>)> {
    let temp_dir = tempfile::tempdir()?;
    let config = create_test_config(temp_dir.path())?;
    
    let repository = SqliteMemoryRepository::new(&config)?;
    let service = MemoryService::new(repository, config);
    
    Ok((temp_dir, service))
}

fn create_test_config(temp_path: &std::path::Path) -> Result<KiroConfig> {
    let db_path = temp_path.join("db.sqlite3");
    
    Ok(KiroConfig {
        root_dir: temp_path.to_path_buf(),
        memory: hail_mary::models::kiro::MemoryConfig {
            types: vec![
                "tech".to_string(),
                "project-tech".to_string(),
                "domain".to_string(),
            ],
            instructions: "Japanese search test instructions".to_string(),
            document: hail_mary::models::kiro::DocumentConfig {
                output_dir: temp_path.to_path_buf(),
                format: "markdown".to_string(),
            },
            database: hail_mary::models::kiro::DatabaseConfig {
                path: db_path,
            },
        },
    })
}

async fn populate_japanese_memories(service: &mut MemoryService<SqliteMemoryRepository>) -> Result<()> {
    let japanese_memories = vec![
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "Rustの非同期プログラミング".to_string(),
            content: "Rustでは async/await 構文を使用して非同期プログラミングを行います。tokio ランタイムが最も一般的です。".to_string(),
            tags: vec!["rust".to_string(), "非同期".to_string(), "プログラミング".to_string()],
            confidence: Some(0.95),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "JavaScript非同期処理".to_string(),
            content: "JavaScriptの非同期処理にはPromiseとasync/awaitを使用します。".to_string(),
            tags: vec!["javascript".to_string(), "非同期".to_string(), "処理".to_string()],
            confidence: Some(0.88),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "SQLite データベース設計".to_string(),
            content: "SQLiteを使用してメモリシステムを実装します。FTS5による全文検索機能を提供します。".to_string(),
            tags: vec!["sqlite".to_string(), "データベース".to_string(), "設計".to_string()],
            confidence: Some(0.92),
        },
    ];
    
    service.remember_batch(japanese_memories).await
}

async fn populate_mixed_language_memories(service: &mut MemoryService<SqliteMemoryRepository>) -> Result<()> {
    let mixed_memories = vec![
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "Mixed Content with 日本語 and English".to_string(),
            content: "This memory contains both English and 日本語 content for testing multilingual search capabilities. プログラミング languages like Rust provide excellent performance.".to_string(),
            tags: vec!["multilingual".to_string(), "mixed".to_string(), "programming".to_string()],
            confidence: Some(0.85),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "Database Performance with データベース最適化".to_string(),
            content: "Database performance optimization techniques include proper indexing and query tuning. データベースの最適化は重要な技術です。".to_string(),
            tags: vec!["database".to_string(), "performance".to_string(), "最適化".to_string()],
            confidence: Some(0.89),
        },
        MemoryInput {
            memory_type: MemoryType::Domain,
            title: "International Development Team Communication".to_string(),
            content: "Working with international teams requires clear communication. 国際的なチームワークには明確なコミュニケーションが必要です。Code comments should be in English, but domain knowledge can be in local languages.".to_string(),
            tags: vec!["international".to_string(), "communication".to_string(), "team".to_string()],
            confidence: Some(0.82),
        },
    ];
    
    service.remember_batch(mixed_memories).await
}

async fn populate_comprehensive_japanese_content(service: &mut MemoryService<SqliteMemoryRepository>) -> Result<()> {
    let comprehensive_memories = vec![
        // More Rust content
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "Rust所有権システム".to_string(),
            content: "Rustの所有権システムはメモリ安全性を保証します。ガベージコレクションなしでメモリリークを防ぎます。".to_string(),
            tags: vec!["rust".to_string(), "所有権".to_string(), "メモリ".to_string()],
            confidence: Some(0.93),
        },
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "Rustパフォーマンス最適化".to_string(),
            content: "Rustのパフォーマンス最適化技術について。ゼロコスト抽象化とコンパイル時最適化。".to_string(),
            tags: vec!["rust".to_string(), "パフォーマンス".to_string(), "最適化".to_string()],
            confidence: Some(0.91),
        },
        // More async content
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "非同期プログラミングパターン".to_string(),
            content: "非同期プログラミングの基本パターンとベストプラクティス。ノンブロッキングI/Oの活用。".to_string(),
            tags: vec!["非同期".to_string(), "パターン".to_string(), "プログラミング".to_string()],
            confidence: Some(0.87),
        },
        // More database content
        MemoryInput {
            memory_type: MemoryType::Tech,
            title: "データベース設計原則".to_string(),
            content: "効果的なデータベース設計の原則。正規化、インデックス設計、パフォーマンス考慮事項。".to_string(),
            tags: vec!["データベース".to_string(), "設計".to_string(), "原則".to_string()],
            confidence: Some(0.94),
        },
        // More programming content
        MemoryInput {
            memory_type: MemoryType::Domain,
            title: "プログラミング教育方法論".to_string(),
            content: "効果的なプログラミング教育のアプローチ。実践的な学習と理論的基礎のバランス。".to_string(),
            tags: vec!["プログラミング".to_string(), "教育".to_string(), "方法論".to_string()],
            confidence: Some(0.86),
        },
    ];
    
    service.remember_batch(comprehensive_memories).await
}

fn extract_titles_from_markdown(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .filter(|line| line.starts_with("## "))
        .map(|line| line.trim_start_matches("## ").to_string())
        .collect()
}

fn calculate_precision(found: &[String], expected: &[String]) -> f64 {
    if found.is_empty() {
        return 0.0;
    }
    
    let relevant_found = found
        .iter()
        .filter(|found_title| {
            expected.iter().any(|expected_title| {
                found_title.contains(expected_title) || expected_title.contains(found_title)
            })
        })
        .count();
    
    relevant_found as f64 / found.len() as f64
}

#[cfg(test)]
mod japanese_search_tests {
    use super::*;
    
    #[test]
    fn test_title_extraction() {
        let markdown = r#"
## Title One
Some content here

## Title Two
More content

## Title Three
Final content
"#;
        
        let titles = extract_titles_from_markdown(markdown);
        assert_eq!(titles.len(), 3);
        assert_eq!(titles[0], "Title One");
        assert_eq!(titles[1], "Title Two");
        assert_eq!(titles[2], "Title Three");
    }
    
    #[test]
    fn test_precision_calculation() {
        let found = vec!["Title A".to_string(), "Title B".to_string(), "Title C".to_string()];
        let expected = vec!["Title A".to_string(), "Title B".to_string()];
        
        let precision = calculate_precision(&found, &expected);
        assert!((precision - 0.667).abs() < 0.01); // 2/3 ≈ 0.667
    }
}