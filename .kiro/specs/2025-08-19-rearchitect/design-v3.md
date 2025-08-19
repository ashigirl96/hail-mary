# Clean Architecture リファクタリング設計仕様書 V3

## 概要

hail-maryプロジェクトをClean Architectureに準拠させるための最終設計仕様書です。V2の過度な抽象化を見直し、シンプルで実用的なアーキテクチャを採用します。

### 設計方針

1. **YAGNI (You Aren't Gonna Need It)**: 現時点で不要な抽象化は避ける
2. **KISS (Keep It Simple, Stupid)**: シンプルで理解しやすい設計
3. **既存パターンの踏襲**: 成功しているMemoryRepositoryパターンを参考に
4. **Stupid Helper回避**: 価値のない薄いラッパーは作らない

### V2からの主な変更点

- **削除**: KiroProjectService（ドメインサービス） - 不要な抽象化
- **削除**: FileSystemトレイト - Stupid Helperアンチパターン
- **簡略化**: 3層構造に集約（Domain, Application, Infrastructure）

## アーキテクチャ概要

```
┌─────────────────────────────────────────────────────────┐
│                    Interface Layer                       │
│                  Commands (CLI入力)                      │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────┐
│                  Application Layer                       │
│              ProjectService (ユースケース)               │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────┐
│                     Domain Layer                         │
│         KiroConfig, KiroFeature (ビジネスルール)          │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────┐
│                 Infrastructure Layer                     │
│      ProjectRepository trait & 実装 (永続化抽象)          │
│         FileProjectRepository (std::fs直接使用)          │
│         InMemoryProjectRepository (テスト用)             │
└─────────────────────────────────────────────────────────┘
```

## 詳細設計

### 1. Domain Layer - ビジネスルールとエンティティ

#### 1.1 KiroConfig - 設定とビジネスルール

```rust
// src/models/kiro_config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::Utc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KiroConfig {
    pub root_dir: PathBuf,
    pub memory: MemoryConfig,
}

impl KiroConfig {
    /// デフォルト設定を生成
    pub fn default() -> Self {
        Self {
            root_dir: PathBuf::from(".kiro"),
            memory: MemoryConfig {
                types: vec![
                    "tech".to_string(),
                    "project-tech".to_string(),
                    "domain".to_string(),
                ],
                instructions: "Default memory types for hail-mary".to_string(),
                document: DocumentConfig {
                    output_dir: PathBuf::from(".kiro/memory"),
                    format: "markdown".to_string(),
                },
                database: DatabaseConfig {
                    path: PathBuf::from(".kiro/memory/db.sqlite3"),
                },
            },
        }
    }
    
    // ===== ビジネスルール: 設定関連 =====
    
    /// メモリタイプの妥当性検証
    pub fn validate_memory_type(&self, memory_type: &str) -> bool {
        self.memory.types.contains(&memory_type.to_string())
    }
    
    /// メモリドキュメントディレクトリ取得
    pub fn memory_docs_dir(&self) -> PathBuf {
        self.memory.document.output_dir.clone()
    }
    
    /// データベースパス取得
    pub fn memory_database_path(&self) -> PathBuf {
        self.memory.database.path.clone()
    }
    
    // ===== ビジネスルール: プロジェクト構造 =====
    
    /// 機能ディレクトリの命名規則（YYYY-MM-dd-feature-name）
    pub fn generate_feature_dir_name(&self, feature_name: &str) -> String {
        format!("{}-{}", Utc::now().format("%Y-%m-%d"), feature_name)
    }
    
    /// デフォルトの機能ファイル構成
    pub fn default_feature_files() -> Vec<&'static str> {
        vec![
            "requirements.md",
            "design.md", 
            "tasks.md",
            "spec.json",
        ]
    }
    
    /// プロジェクトに必要なディレクトリ構造
    pub fn required_directories(&self) -> Vec<PathBuf> {
        vec![
            self.root_dir.clone(),
            self.root_dir.join("memory"),
            self.root_dir.join("specs"),
        ]
    }
    
    /// .gitignoreに追加すべきエントリ
    pub fn default_gitignore_entries() -> Vec<&'static str> {
        vec![
            "# hail-mary memory database",
            ".kiro/memory/db.sqlite3",
            ".kiro/memory/*.sqlite3-*",
        ]
    }
    
    /// 設定テンプレートの生成
    pub fn config_template() -> &'static str {
        r#"# .kiro/config.toml
# hail-mary Memory MCP project configuration

[memory]
# Memory types for categorization (customize for your project)
types = [
    "tech",           # General technical knowledge
    "project-tech",   # Project-specific technical details
    "domain",         # Business domain knowledge
    "workflow",       # Development workflows and processes
    "decision",       # Architecture decisions and rationale
]

# Instructions for MCP server
instructions = """
Available memory types:
- tech: General technical knowledge (languages, frameworks, algorithms)
- project-tech: This project's specific technical implementation
- domain: Business domain knowledge and requirements
- workflow: Development workflows and processes
- decision: Architecture decisions and their rationale
"""

# Document generation settings
[memory.document]
output_dir = ".kiro/memory"
format = "markdown"

# Database configuration
[memory.database]
path = ".kiro/memory/db.sqlite3"
"#
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    pub types: Vec<String>,
    pub instructions: String,
    pub document: DocumentConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentConfig {
    pub output_dir: PathBuf,
    pub format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}
```

#### 1.2 KiroFeature - エンティティ

```rust
// src/models/kiro_feature.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KiroFeature {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub directory_name: String,
    pub path: Option<PathBuf>,
}

impl KiroFeature {
    /// 新しいKiroFeatureを作成
    pub fn new(name: String, directory_name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: Utc::now(),
            directory_name,
            path: None,
        }
    }
    
    /// 既存のKiroFeatureを復元（永続化からの読み込み時）
    pub fn restore(
        id: String,
        name: String,
        created_at: DateTime<Utc>,
        directory_name: String,
        path: Option<PathBuf>,
    ) -> Self {
        Self {
            id,
            name,
            created_at,
            directory_name,
            path,
        }
    }
    
    /// パスを設定
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }
    
    // ===== ビジネスルール: バリデーション =====
    
    /// 機能名のバリデーション（kebab-case）
    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 50
            && name.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c.is_ascii_digit())
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.contains("--")
    }
    
    /// 既存機能との重複チェック
    pub fn can_create(name: &str, existing_features: &[KiroFeature]) -> bool {
        !existing_features.iter().any(|f| f.name == name)
    }
}
```

### 2. Infrastructure Layer - 永続化の抽象化（FileSystemトレイトなし）

#### 2.1 ProjectRepository トレイト

```rust
// src/repositories/project.rs
use crate::models::error::Result;
use crate::models::kiro_config::KiroConfig;
use crate::models::kiro_feature::KiroFeature;
use std::path::PathBuf;

/// プロジェクト関連の永続化を抽象化
/// FileSystemトレイトは使用せず、直接実装で対応
pub trait ProjectRepository {
    // 初期化関連
    fn initialize_structure(&self, config: &KiroConfig) -> Result<()>;
    
    // 機能関連
    fn save_feature(&self, feature: &KiroFeature) -> Result<PathBuf>;
    fn find_feature_by_name(&self, name: &str) -> Result<Option<KiroFeature>>;
    fn list_all_features(&self) -> Result<Vec<KiroFeature>>;
    
    // 設定関連
    fn save_config(&self, config: &KiroConfig) -> Result<()>;
    fn load_config(&self) -> Result<KiroConfig>;
    fn find_kiro_root(&self) -> Result<PathBuf>;
    
    // その他
    fn update_gitignore(&self, entries: &[String]) -> Result<()>;
}
```

#### 2.2 FileProjectRepository - 本番実装

```rust
// src/repositories/project_file.rs
use crate::models::error::{MemoryError, Result};
use crate::models::kiro_config::KiroConfig;
use crate::models::kiro_feature::KiroFeature;
use crate::repositories::project::ProjectRepository;
use std::fs;
use std::path::{Path, PathBuf};

/// ファイルシステムベースのProjectRepository実装
/// FileSystemトレイトを使わず、std::fsを直接使用
pub struct FileProjectRepository {
    base_path: PathBuf,
}

impl FileProjectRepository {
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from(".kiro"),
        }
    }
    
    pub fn with_base_path(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl ProjectRepository for FileProjectRepository {
    fn initialize_structure(&self, config: &KiroConfig) -> Result<()> {
        // 必要なディレクトリを作成（std::fs直接使用）
        for dir in config.required_directories() {
            fs::create_dir_all(&dir).map_err(MemoryError::Io)?;
        }
        
        // 設定ファイルを作成
        let config_path = config.root_dir.join("config.toml");
        fs::write(&config_path, KiroConfig::config_template())
            .map_err(MemoryError::Io)?;
        
        // .gitignore更新
        self.update_gitignore(&KiroConfig::default_gitignore_entries()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>())?;
        
        Ok(())
    }
    
    fn save_feature(&self, feature: &KiroFeature) -> Result<PathBuf> {
        let feature_path = self.base_path.join("specs").join(&feature.directory_name);
        
        // ディレクトリ作成（std::fs直接使用）
        fs::create_dir_all(&feature_path).map_err(MemoryError::Io)?;
        
        // 必要ファイルを作成
        for filename in KiroConfig::default_feature_files() {
            let file_path = feature_path.join(filename);
            let content = match filename {
                "spec.json" => serde_json::to_string_pretty(feature)?,
                _ => String::new(),
            };
            fs::write(&file_path, content).map_err(MemoryError::Io)?;
        }
        
        Ok(feature_path)
    }
    
    fn find_feature_by_name(&self, name: &str) -> Result<Option<KiroFeature>> {
        let features = self.list_all_features()?;
        Ok(features.into_iter().find(|f| f.name == name))
    }
    
    fn list_all_features(&self) -> Result<Vec<KiroFeature>> {
        let specs_dir = self.base_path.join("specs");
        
        if !specs_dir.exists() {
            return Ok(vec![]);
        }
        
        let mut features = Vec::new();
        
        for entry in fs::read_dir(&specs_dir).map_err(MemoryError::Io)? {
            let entry = entry.map_err(MemoryError::Io)?;
            let path = entry.path();
            
            if path.is_dir() {
                // spec.jsonから機能情報を読み込み
                let spec_file = path.join("spec.json");
                if spec_file.exists() {
                    let content = fs::read_to_string(&spec_file)
                        .map_err(MemoryError::Io)?;
                    if let Ok(mut feature) = serde_json::from_str::<KiroFeature>(&content) {
                        feature.path = Some(path);
                        features.push(feature);
                    }
                }
            }
        }
        
        features.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(features)
    }
    
    fn save_config(&self, config: &KiroConfig) -> Result<()> {
        let config_path = config.root_dir.join("config.toml");
        let content = toml::to_string_pretty(config)?;
        fs::write(&config_path, content).map_err(MemoryError::Io)?;
        Ok(())
    }
    
    fn load_config(&self) -> Result<KiroConfig> {
        let kiro_root = self.find_kiro_root()?;
        let config_path = kiro_root.join("config.toml");
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .map_err(MemoryError::Io)?;
            let config: KiroConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(KiroConfig::default())
        }
    }
    
    fn find_kiro_root(&self) -> Result<PathBuf> {
        let mut current_dir = std::env::current_dir()
            .map_err(MemoryError::Io)?;
        
        loop {
            let kiro_dir = current_dir.join(".kiro");
            if kiro_dir.exists() && kiro_dir.is_dir() {
                return Ok(kiro_dir);
            }
            
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                return Err(MemoryError::NotFound(
                    ".kiro directory not found".to_string()
                ));
            }
        }
    }
    
    fn update_gitignore(&self, entries: &[String]) -> Result<()> {
        let gitignore_path = Path::new(".gitignore");
        
        if gitignore_path.exists() {
            let content = fs::read_to_string(gitignore_path)
                .map_err(MemoryError::Io)?;
            
            // 既存エントリの確認
            let mut needs_update = false;
            let mut new_entries = Vec::new();
            
            for entry in entries {
                if !content.contains(entry) {
                    needs_update = true;
                    new_entries.push(entry.clone());
                }
            }
            
            if needs_update {
                use std::fs::OpenOptions;
                use std::io::Write;
                
                let mut file = OpenOptions::new()
                    .append(true)
                    .open(gitignore_path)
                    .map_err(MemoryError::Io)?;
                
                writeln!(file)?;
                for entry in new_entries {
                    writeln!(file, "{}", entry)?;
                }
            }
        } else {
            // 新規作成
            let content = entries.join("\n") + "\n";
            fs::write(gitignore_path, content).map_err(MemoryError::Io)?;
        }
        
        Ok(())
    }
}
```

#### 2.3 InMemoryProjectRepository - テスト実装

```rust
// src/repositories/project_memory.rs
use crate::models::error::{MemoryError, Result};
use crate::models::kiro_config::KiroConfig;
use crate::models::kiro_feature::KiroFeature;
use crate::repositories::project::ProjectRepository;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// インメモリのProjectRepository実装（テスト用）
/// FileSystemトレイトなしで実装
pub struct InMemoryProjectRepository {
    features: Arc<Mutex<HashMap<String, KiroFeature>>>,
    files: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
    config: Arc<Mutex<Option<KiroConfig>>>,
}

impl InMemoryProjectRepository {
    pub fn new() -> Self {
        Self {
            features: Arc::new(Mutex::new(HashMap::new())),
            files: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(Mutex::new(None)),
        }
    }
}

impl ProjectRepository for InMemoryProjectRepository {
    fn initialize_structure(&self, config: &KiroConfig) -> Result<()> {
        let mut config_store = self.config.lock().unwrap();
        *config_store = Some(config.clone());
        
        // ディレクトリ構造をメモリ上に記録
        let mut files = self.files.lock().unwrap();
        for dir in config.required_directories() {
            files.insert(dir, Vec::new());
        }
        
        Ok(())
    }
    
    fn save_feature(&self, feature: &KiroFeature) -> Result<PathBuf> {
        let mut features = self.features.lock().unwrap();
        features.insert(feature.name.clone(), feature.clone());
        
        let feature_path = PathBuf::from(".kiro/specs").join(&feature.directory_name);
        
        // ファイル構造をメモリ上に記録
        let mut files = self.files.lock().unwrap();
        for filename in KiroConfig::default_feature_files() {
            let file_path = feature_path.join(filename);
            let content = match filename {
                "spec.json" => serde_json::to_vec(feature)?,
                _ => Vec::new(),
            };
            files.insert(file_path, content);
        }
        
        Ok(feature_path)
    }
    
    fn find_feature_by_name(&self, name: &str) -> Result<Option<KiroFeature>> {
        let features = self.features.lock().unwrap();
        Ok(features.get(name).cloned())
    }
    
    fn list_all_features(&self) -> Result<Vec<KiroFeature>> {
        let features = self.features.lock().unwrap();
        let mut feature_list: Vec<_> = features.values().cloned().collect();
        feature_list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(feature_list)
    }
    
    fn save_config(&self, config: &KiroConfig) -> Result<()> {
        let mut config_store = self.config.lock().unwrap();
        *config_store = Some(config.clone());
        Ok(())
    }
    
    fn load_config(&self) -> Result<KiroConfig> {
        let config_store = self.config.lock().unwrap();
        Ok(config_store.clone().unwrap_or_else(KiroConfig::default))
    }
    
    fn find_kiro_root(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(".kiro"))
    }
    
    fn update_gitignore(&self, entries: &[String]) -> Result<()> {
        let mut files = self.files.lock().unwrap();
        let gitignore_path = PathBuf::from(".gitignore");
        
        let existing = files.get(&gitignore_path)
            .map(|v| String::from_utf8_lossy(v).to_string())
            .unwrap_or_default();
        
        let mut content = existing;
        for entry in entries {
            if !content.contains(entry) {
                if !content.is_empty() && !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push_str(entry);
                content.push('\n');
            }
        }
        
        files.insert(gitignore_path, content.into_bytes());
        Ok(())
    }
}
```

### 3. Application Layer - ユースケースの実装

```rust
// src/services/project.rs
use crate::models::error::{MemoryError, Result};
use crate::models::kiro_config::KiroConfig;
use crate::models::kiro_feature::KiroFeature;
use crate::repositories::project::ProjectRepository;
use std::path::PathBuf;

/// プロジェクト関連のユースケースを実装
/// ドメインサービスは使用せず、直接ビジネスロジックを実装
pub struct ProjectService<R: ProjectRepository> {
    repository: R,
    config: KiroConfig,
}

impl<R: ProjectRepository> ProjectService<R> {
    pub fn new(repository: R) -> Result<Self> {
        let config = repository.load_config()?;
        Ok(Self { repository, config })
    }
    
    pub fn with_config(repository: R, config: KiroConfig) -> Self {
        Self { repository, config }
    }
    
    /// プロジェクト初期化のユースケース
    pub fn initialize_project(&self, force: bool) -> Result<()> {
        // 既存チェック
        if !force {
            if let Ok(_) = self.repository.find_kiro_root() {
                return Err(MemoryError::InvalidInput(
                    ".kiro directory already exists. Use --force to overwrite.".to_string()
                ));
            }
        }
        
        // 初期化実行
        self.repository.initialize_structure(&self.config)?;
        
        Ok(())
    }
    
    /// 新機能作成のユースケース
    pub fn create_new_feature(&self, name: &str) -> Result<PathBuf> {
        // バリデーション（ドメインルール）
        if !KiroFeature::is_valid_name(name) {
            return Err(MemoryError::InvalidInput(
                format!("Invalid feature name: {}. Must be kebab-case.", name)
            ));
        }
        
        // 重複チェック（ドメインルール）
        let existing = self.repository.list_all_features()?;
        if !KiroFeature::can_create(name, &existing) {
            return Err(MemoryError::InvalidInput(
                format!("Feature '{}' already exists", name)
            ));
        }
        
        // Feature作成（ドメインルール適用）
        let directory_name = self.config.generate_feature_dir_name(name);
        let feature = KiroFeature::new(name.to_string(), directory_name);
        
        // 永続化
        let feature_path = self.repository.save_feature(&feature)?;
        
        Ok(feature_path)
    }
    
    /// 機能一覧取得のユースケース
    pub fn list_features(&self) -> Result<Vec<KiroFeature>> {
        self.repository.list_all_features()
    }
    
    /// 機能検索のユースケース
    pub fn find_feature(&self, name: &str) -> Result<Option<KiroFeature>> {
        self.repository.find_feature_by_name(name)
    }
}
```

### 4. Interface Layer - CLIコマンド

```rust
// src/commands/init.rs
use crate::services::project::ProjectService;
use crate::repositories::project_file::FileProjectRepository;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct InitCommand {
    /// Force overwrite existing configuration
    #[arg(long)]
    pub force: bool,
}

impl InitCommand {
    pub fn execute(&self) -> Result<()> {
        // 依存性の構築（DI）
        let repository = FileProjectRepository::new();
        let service = ProjectService::new(repository)?;
        
        // ユースケースの実行
        service.initialize_project(self.force)?;
        
        // 成功メッセージ
        println!("✅ Initialized .kiro directory structure");
        println!("  - Created .kiro/");
        println!("  - Created .kiro/config.toml");
        println!("  - Created .kiro/memory/");
        println!("  - Created .kiro/specs/");
        println!("  - Updated .gitignore");
        
        Ok(())
    }
}

// src/commands/new.rs
use crate::services::project::ProjectService;
use crate::repositories::project_file::FileProjectRepository;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct NewCommand {
    /// Name of the new feature (must be in kebab-case)
    pub feature_name: String,
}

impl NewCommand {
    pub fn execute(&self) -> Result<()> {
        // 依存性の構築（DI）
        let repository = FileProjectRepository::new();
        let service = ProjectService::new(repository)?;
        
        // ユースケースの実行
        let feature_path = service.create_new_feature(&self.feature_name)?;
        
        // 成功メッセージ
        println!("✅ Feature '{}' created successfully!", self.feature_name);
        println!("📁 Location: {}", feature_path.display());
        println!("📝 Files created:");
        println!("   - requirements.md");
        println!("   - design.md");
        println!("   - tasks.md");
        println!("   - spec.json");
        
        Ok(())
    }
}
```

## テスト戦略

### ユニットテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::project_memory::InMemoryProjectRepository;
    
    #[test]
    fn test_create_feature_success() {
        // InMemoryRepositoryで高速テスト（FileSystemトレイトなし）
        let repository = InMemoryProjectRepository::new();
        let config = KiroConfig::default();
        let service = ProjectService::with_config(repository, config);
        
        let result = service.create_new_feature("test-feature");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_duplicate_feature_fails() {
        let repository = InMemoryProjectRepository::new();
        let config = KiroConfig::default();
        let service = ProjectService::with_config(repository, config);
        
        // 最初の作成は成功
        assert!(service.create_new_feature("test-feature").is_ok());
        
        // 重複作成は失敗
        let result = service.create_new_feature("test-feature");
        assert!(result.is_err());
    }
}
```

### 統合テスト

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_with_real_filesystem() {
        // 実ファイルシステムでテスト
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        let repository = FileProjectRepository::new();
        let service = ProjectService::new(repository).unwrap();
        
        // 初期化
        assert!(service.initialize_project(false).is_ok());
        
        // 機能作成
        let result = service.create_new_feature("test-feature");
        assert!(result.is_ok());
        
        // ファイル確認
        let feature_path = result.unwrap();
        assert!(feature_path.exists());
        assert!(feature_path.join("requirements.md").exists());
    }
}
```

## 移行計画

### Phase 1: Domain Layer（1-2日） ✅ 完了
1. ✅ `KiroConfig`にビジネスルールメソッド追加
2. ✅ `KiroFeature`エンティティ作成

### Phase 2: Infrastructure Layer（2-3日） ✅ 完了
1. ✅ `ProjectRepository`トレイト定義
2. ✅ `FileProjectRepository`実装（std::fs直接使用）
3. ✅ `InMemoryProjectRepository`実装

### Phase 3: Application Layer（1-2日）
1. `ProjectService`実装
2. 既存`ProjectManager`からロジック移行

### Phase 4: テスト追加（2-3日）
1. ユニットテスト作成
2. 統合テスト作成
3. E2Eテスト作成

### Phase 5: 既存コード置換（1日）
1. Commands層の更新
2. 既存`ProjectManager`の削除

## まとめ

### 採用した設計

✅ **シンプルな3層アーキテクチャ**
- Domain Layer: ビジネスルールを含むモデル
- Application Layer: ユースケース実装
- Infrastructure Layer: 永続化の抽象化

✅ **既存パターンの踏襲**
- MemoryRepositoryパターンと同じアプローチ
- 成功実績のある設計

### 削除した設計

❌ **KiroProjectService（ドメインサービス）**
- 不要な抽象化
- ビジネスルールはドメインモデルで十分

❌ **FileSystemトレイト**
- Stupid Helperアンチパターン
- ProjectRepositoryだけで十分な抽象化

### 期待される効果

1. **シンプルで理解しやすい**: 層が少なく直感的
2. **テスタブル**: Repository層でモック化可能
3. **保守しやすい**: 責務が明確に分離
4. **YAGNI/KISS準拠**: 必要十分な設計
5. **拡張可能**: 将来の要件にも対応可能

この設計により、Clean Architectureの原則を守りながら、実用的でシンプルなアーキテクチャを実現します。