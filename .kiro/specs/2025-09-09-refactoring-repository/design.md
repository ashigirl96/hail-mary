# Design: Repository層リファクタリング設計書

## 概要
現在の`ProjectRepository`は多くの責任を持ちすぎているため、より明確な責任分担を実現するために3つの専門的なリポジトリに分離します。

## 現状の問題点

### 1. 単一責任原則の違反
現在の`ProjectRepository`は以下の多岐にわたる責任を持っています：
- プロジェクト初期化
- Spec管理（作成、一覧、完了、アーカイブ）
- Config管理（読み込み、保存、更新）
- Steering管理（ファイル、バックアップ）
- Gitignore更新
- Slash commands deployment

### 2. コードの重複
- 各load系関数が独立してTOMLをパースしている
- ensure系関数も独立してファイル読み込み・書き込みをしている
- `load_config()`が実際のTOML読み込みをせず、デフォルト値を返すだけ

## 新しいアーキテクチャ

### レイヤー構造
```
Application Layer (Traits)
    ├── ConfigRepository trait
    ├── SpecRepository trait 
    ├── SteeringRepository trait
    └── ProjectRepository trait (Coordinator)
    
Infrastructure Layer (Implementations)
    ├── ConfigRepositoryImpl
    ├── SpecRepositoryImpl
    ├── SteeringRepositoryImpl
    └── ProjectRepositoryImpl (uses other repositories)
```

## 詳細設計

### 1. ConfigRepository

#### 責任範囲
- 設定ファイル(.kiro/config.toml)の読み込み・保存・更新
- TOMLパース処理の一元化
- 設定の部分更新（ensure系メソッド）

#### トレイト定義
```rust
// application/repositories/config_repository.rs
pub trait ConfigRepository: Send + Sync {
    /// TOMLファイルを読み込んでProjectConfigを返す
    fn load_config(&self) -> Result<ProjectConfig, ApplicationError>;
    
    /// ProjectConfig全体を保存
    fn save_config(&self, config: &ProjectConfig) -> Result<(), ApplicationError>;
    
    /// SteeringConfigのみを取得
    fn load_steering_config(&self) -> Result<SteeringConfig, ApplicationError>;
    
    /// SteeringBackupConfigのみを取得
    fn load_steering_backup_config(&self) -> Result<SteeringBackupConfig, ApplicationError>;
    
    /// Steering設定が不足している場合、デフォルト値で補完
    fn ensure_steering_config(&self) -> Result<(), ApplicationError>;
    
    /// Steeringバックアップ設定が不足している場合、デフォルト値で補完
    fn ensure_steering_backup_config(&self) -> Result<(), ApplicationError>;
}
```

#### 実装詳細
```rust
// infrastructure/repositories/config.rs
pub struct ConfigRepositoryImpl {
    path_manager: PathManager,
}

impl ConfigRepositoryImpl {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }
    
    fn load_toml(&self) -> Result<toml::Value, ApplicationError> {
        let config_path = self.path_manager.config_path(true);
        
        if !config_path.exists() {
            return Ok(toml::Value::Table(toml::map::Map::new()));
        }
        
        let content = fs::read_to_string(&config_path)?;
        toml::from_str(&content).map_err(|e| 
            ApplicationError::ConfigurationError(format!("Failed to parse TOML: {}", e))
        )
    }
    
    fn save_toml(&self, value: &toml::Value) -> Result<(), ApplicationError> {
        let config_path = self.path_manager.config_path(true);
        let content = toml::to_string_pretty(value)?;
        fs::write(config_path, content)?;
        Ok(())
    }
}
```

### 2. SpecRepository

#### 責任範囲
- Spec（Feature）の作成・一覧・完了・アーカイブ
- テンプレートファイルの生成
- Spec名のバリデーション

#### トレイト定義
```rust
// application/repositories/spec_repository.rs
pub trait SpecRepository: Send + Sync {
    /// 新しいfeature specを作成
    fn create_feature(&self, name: &str) -> Result<(), ApplicationError>;
    
    /// specディレクトリ一覧を取得
    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError>;
    
    /// specを完了としてアーカイブに移動
    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError>;
    
    /// specディレクトリのパスを取得
    fn get_spec_path(&self, name: &str) -> Result<PathBuf, ApplicationError>;
    
    /// アーカイブされたspec一覧を取得
    fn list_archived_specs(&self) -> Result<Vec<String>, ApplicationError>;
}
```

#### 実装詳細
```rust
// infrastructure/repositories/spec.rs
pub struct SpecRepositoryImpl {
    path_manager: PathManager,
}

impl SpecRepositoryImpl {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }
    
    fn validate_feature_name(&self, name: &str) -> Result<(), ApplicationError> {
        // kebab-case validation
        if !name.chars().all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
            || name.starts_with('-')
            || name.ends_with('-')
            || name.contains("--")
        {
            return Err(ApplicationError::InvalidFeatureName(name.to_string()));
        }
        Ok(())
    }
    
    fn create_template_files(&self, feature_dir: &Path, name: &str) -> Result<(), ApplicationError> {
        // requirements.md, design.md, tasks.md, memo.md, investigation.md, spec.json
        // の作成処理
    }
}
```

### 3. SteeringRepository

#### 責任範囲
- Steeringファイルの作成・一覧・管理
- Steeringバックアップの作成・削除・管理
- Slash commandsのデプロイ

#### トレイト定義
```rust
// application/repositories/steering_repository.rs
pub trait SteeringRepository: Send + Sync {
    /// steeringディレクトリを初期化
    fn initialize_steering(&self) -> Result<(), ApplicationError>;
    
    /// steering設定からmarkdownファイルを作成
    fn create_steering_files(&self, config: &SteeringConfig) -> Result<(), ApplicationError>;
    
    /// steeringファイル一覧を取得
    fn list_steering_files(&self) -> Result<Vec<PathBuf>, ApplicationError>;
    
    /// steeringファイルのバックアップを作成
    fn create_steering_backup(&self, timestamp: &str, files: &[PathBuf]) -> Result<(), ApplicationError>;
    
    /// バックアップ一覧を取得
    fn list_steering_backups(&self) -> Result<Vec<BackupInfo>, ApplicationError>;
    
    /// 古いバックアップを削除
    fn delete_oldest_steering_backups(&self, count: usize) -> Result<(), ApplicationError>;
    
    /// slash commandsをデプロイ
    fn deploy_slash_commands(&self) -> Result<(), ApplicationError>;
}
```

#### 実装詳細
```rust
// infrastructure/repositories/steering.rs
pub struct SteeringRepositoryImpl {
    path_manager: PathManager,
}

impl SteeringRepositoryImpl {
    pub fn new(path_manager: PathManager) -> Self {
        Self { path_manager }
    }
    
    fn steering_dir(&self) -> PathBuf {
        self.path_manager.kiro_dir(true).join("steering")
    }
    
    fn backup_dir(&self) -> PathBuf {
        self.steering_dir().join("backup")
    }
}
```

### 4. ProjectRepository（Coordinator）

#### 責任範囲
- 各リポジトリの統合・調整
- プロジェクト全体の初期化
- gitignore更新などプロジェクト全体の管理

#### トレイト定義（更新版）
```rust
// application/repositories/project_repository.rs
pub trait ProjectRepository: Send + Sync {
    /// プロジェクトを初期化（各リポジトリの初期化を調整）
    fn initialize(&self) -> Result<(), ApplicationError>;
    
    /// プロジェクトが存在するか確認
    fn exists(&self) -> Result<bool, ApplicationError>;
    
    /// gitignoreを更新
    fn update_gitignore(&self) -> Result<(), ApplicationError>;
}
```

#### 実装詳細
```rust
// infrastructure/repositories/project.rs
pub struct ProjectRepositoryImpl {
    path_manager: PathManager,
    config_repository: Arc<dyn ConfigRepository>,
    spec_repository: Arc<dyn SpecRepository>,
    steering_repository: Arc<dyn SteeringRepository>,
}

impl ProjectRepositoryImpl {
    pub fn new(
        path_manager: PathManager,
        config_repository: Arc<dyn ConfigRepository>,
        spec_repository: Arc<dyn SpecRepository>,
        steering_repository: Arc<dyn SteeringRepository>,
    ) -> Self {
        Self {
            path_manager,
            config_repository,
            spec_repository,
            steering_repository,
        }
    }
}

impl ProjectRepository for ProjectRepositoryImpl {
    fn initialize(&self) -> Result<(), ApplicationError> {
        // 1. .kiroディレクトリ作成
        let kiro_dir = self.path_manager.kiro_dir(true);
        fs::create_dir_all(&kiro_dir)?;
        
        // 2. specsディレクトリ作成
        let specs_dir = self.path_manager.specs_dir(true);
        fs::create_dir_all(&specs_dir)?;
        
        // 3. steering初期化を委譲
        self.steering_repository.initialize_steering()?;
        
        // 4. config初期化を委譲
        let config = ProjectConfig::default_for_new_project();
        self.config_repository.save_config(&config)?;
        
        // 5. steeringファイル作成を委譲
        self.steering_repository.create_steering_files(&config.steering)?;
        
        // 6. slash commands deployを委譲
        self.steering_repository.deploy_slash_commands()?;
        
        Ok(())
    }
}
```

## 依存性注入の更新

### main.rsでの組み立て
```rust
// main.rs
let path_manager = PathManager::new(project_root);

// 各リポジトリを個別に作成
let config_repo = Arc::new(ConfigRepositoryImpl::new(path_manager.clone()));
let spec_repo = Arc::new(SpecRepositoryImpl::new(path_manager.clone()));
let steering_repo = Arc::new(SteeringRepositoryImpl::new(path_manager.clone()));

// ProjectRepositoryに注入
let project_repo = Arc::new(ProjectRepositoryImpl::new(
    path_manager.clone(),
    config_repo.clone(),
    spec_repo.clone(),
    steering_repo.clone(),
));

// Use caseに必要なリポジトリを渡す
match args.command {
    Commands::Init { force } => {
        initialize_project(project_repo.as_ref(), force)?;
    }
    Commands::New { name } => {
        create_feature(spec_repo.as_ref(), &name)?;
    }
    // ...
}
```

## Use Case層の更新

各Use Caseは必要なリポジトリのみを受け取るように更新：

```rust
// application/use_cases/initialize_project.rs
pub fn initialize_project(
    project_repo: &dyn ProjectRepository,
    force: bool,
) -> Result<(), ApplicationError> {
    // ProjectRepositoryのみ使用
}

// application/use_cases/create_feature.rs
pub fn create_feature(
    spec_repo: &dyn SpecRepository,
    name: &str,
) -> Result<(), ApplicationError> {
    // SpecRepositoryのみ使用
}

// application/use_cases/backup_steering.rs
pub fn backup_steering(
    config_repo: &dyn ConfigRepository,
    steering_repo: &dyn SteeringRepository,
) -> Result<(), ApplicationError> {
    // ConfigとSteeringの両方を使用
}
```

## ProjectConfig構造の改善

```rust
// domain/entities/project.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_instructions")]
    pub instructions: String,
    
    #[serde(default)]
    pub document_format: DocumentFormat,
    
    #[serde(default)]
    pub steering: SteeringSection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SteeringSection {
    #[serde(default)]
    pub types: Vec<SteeringTypeToml>,
    
    #[serde(default)]
    pub backup: Option<SteeringBackupToml>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringTypeToml {
    pub name: String,
    pub purpose: String,
    pub criteria: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteeringBackupToml {
    #[serde(default = "default_backup_max")]
    pub max: usize,
}

fn default_backup_max() -> usize { 10 }
```

## テスト戦略

### 単体テスト
各リポジトリは独立してテスト可能：

```rust
// infrastructure/repositories/config.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::TestDirectory;
    
    #[test]
    fn test_load_config_when_file_exists() {
        let test_dir = TestDirectory::new_no_cd();
        let path_manager = PathManager::new(test_dir.path().to_path_buf());
        let repo = ConfigRepositoryImpl::new(path_manager);
        
        // TOMLファイルを作成
        // ...
        
        let config = repo.load_config().unwrap();
        // アサーション
    }
    
    #[test]
    fn test_ensure_steering_config_adds_missing_section() {
        // ...
    }
}
```

### 統合テスト
ProjectRepositoryを通じた統合動作をテスト：

```rust
// tests/repository_integration.rs
#[test]
fn test_project_initialization_creates_all_structure() {
    let test_dir = TestDirectory::new_no_cd();
    let path_manager = PathManager::new(test_dir.path().to_path_buf());
    
    // 全リポジトリを組み立て
    let config_repo = Arc::new(ConfigRepositoryImpl::new(path_manager.clone()));
    let spec_repo = Arc::new(SpecRepositoryImpl::new(path_manager.clone()));
    let steering_repo = Arc::new(SteeringRepositoryImpl::new(path_manager.clone()));
    let project_repo = ProjectRepositoryImpl::new(
        path_manager.clone(),
        config_repo,
        spec_repo,
        steering_repo,
    );
    
    // 初期化実行
    project_repo.initialize().unwrap();
    
    // 全構造が作成されたことを確認
    assert!(test_dir.path().join(".kiro").exists());
    assert!(test_dir.path().join(".kiro/specs").exists());
    assert!(test_dir.path().join(".kiro/steering").exists());
    assert!(test_dir.path().join(".kiro/config.toml").exists());
}
```

## 移行計画

### Phase 1: 新リポジトリの作成
1. ConfigRepositoryの実装とテスト
2. SpecRepositoryの実装とテスト
3. SteeringRepositoryの実装とテスト

### Phase 2: ProjectRepositoryのリファクタリング
1. 既存メソッドを各リポジトリに移動
2. ProjectRepositoryをCoordinatorに変更
3. 依存性注入の更新

### Phase 3: Use Case層の更新
1. 各Use Caseが適切なリポジトリを使用するよう更新
2. main.rsでの組み立て更新

### Phase 4: テストの更新と検証
1. 単体テストの追加
2. 統合テストの更新
3. 既存テストが全てパスすることを確認

## メリット

1. **単一責任の原則**: 各リポジトリが明確な責任を持つ
2. **テスタビリティ向上**: 独立したテストが可能
3. **保守性向上**: 変更の影響範囲が限定的
4. **拡張性向上**: 新機能追加時の影響が局所化
5. **コード重複の削減**: TOMLパース処理の一元化
6. **型安全性向上**: Serdeによる自動シリアライズ/デシリアライズ

## 潜在的な課題と対策

### 課題1: リポジトリ間の依存
- **問題**: 一部の操作で複数リポジトリが必要
- **対策**: Use Case層で適切に調整、必要に応じてサービス層を導入

### 課題2: パフォーマンス
- **問題**: ConfigRepositoryが頻繁に呼ばれる場合のI/O
- **対策**: 必要に応じてキャッシュ層を導入

### 課題3: 後方互換性
- **問題**: 既存のconfig.tomlとの互換性
- **対策**: Serdeのdefault属性で対応、マイグレーション処理を実装
