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
Application Layer (Traits/Interfaces)
    ├── ConfigRepositoryInterface trait
    ├── SpecRepositoryInterface trait
    └── SteeringRepositoryInterface trait
    
Infrastructure Layer (Implementations)
    ├── ConfigRepository
    ├── SpecRepository
    └── SteeringRepository
```

### 設計方針
- **ProjectRepository（Coordinator）は作成しない** - 不要な抽象層を避ける
- **Use Caseが必要なリポジトリを直接受け取る** - 明示的な依存関係
- **単一責任の徹底** - 各リポジトリは完全に独立

## 詳細設計

### 1. ConfigRepositoryInterface / ConfigRepository

#### 責任範囲
- 設定ファイル(.kiro/config.toml)の読み込み・保存・更新
- TOMLパース処理の一元化
- 設定の部分更新（ensure系メソッド）

#### トレイト定義
```rust
// application/repositories/config_repository.rs
pub trait ConfigRepositoryInterface: Send + Sync {
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
pub struct ConfigRepository {
    path_manager: PathManager,
}

impl ConfigRepository {
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

### 2. SpecRepositoryInterface / SpecRepository

#### 責任範囲
- Spec（Feature）の作成・一覧・完了・アーカイブ
- テンプレートファイルの生成
- Spec名のバリデーション

#### トレイト定義
```rust
// application/repositories/spec_repository.rs
pub trait SpecRepositoryInterface: Send + Sync {
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
pub struct SpecRepository {
    path_manager: PathManager,
}

impl SpecRepository {
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

### 3. SteeringRepositoryInterface / SteeringRepository

#### 責任範囲
- Steeringファイルの作成・一覧・管理
- Steeringバックアップの作成・削除・管理
- Slash commandsのデプロイ

#### トレイト定義
```rust
// application/repositories/steering_repository.rs
pub trait SteeringRepositoryInterface: Send + Sync {
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
pub struct SteeringRepository {
    path_manager: PathManager,
}

impl SteeringRepository {
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

### 4. ProjectRepository削除の理由

#### なぜProjectRepositoryを削除するか
- **不要な抽象層の排除**: CoordinatorパターンはYAGNI原則に反する
- **明示的な依存関係**: Use Caseレベルで必要なリポジトリが明確になる
- **柔軟性の向上**: Use Caseごとに必要なリポジトリの組み合わせを自由に選択可能
- **テストの簡潔性**: モックするリポジトリが最小限で済む

#### 移行される責任
旧ProjectRepositoryの責任は以下のように分散されます：

1. **プロジェクト初期化** → `initialize_project` Use Caseで複数リポジトリを組み合わせる
2. **プロジェクト存在確認** → SteeringRepositoryまたはConfigRepositoryの責任として統合
3. **gitignore更新** → 新しい`GitignoreRepository`を作成するか、適切なリポジトリに移動

## 依存性注入の更新

### main.rsでの組み立て
```rust
// main.rs
let path_manager = PathManager::new(project_root);

// 各リポジトリを個別に作成
let config_repo = Arc::new(ConfigRepository::new(path_manager.clone()));
let spec_repo = Arc::new(SpecRepository::new(path_manager.clone()));
let steering_repo = Arc::new(SteeringRepository::new(path_manager.clone()));

// Use caseに必要なリポジトリを直接渡す（ProjectRepository経由ではない）
match args.command {
    Commands::Init { force } => {
        // 複数のリポジトリを直接渡す
        initialize_project(
            config_repo.as_ref(),
            spec_repo.as_ref(),
            steering_repo.as_ref(),
            force
        )?;
    }
    Commands::New { name } => {
        // 単一のリポジトリだけを渡す
        create_feature(spec_repo.as_ref(), &name)?;
    }
    Commands::Complete => {
        // 必要なリポジトリを渡す
        complete_features(
            spec_repo.as_ref(),
            config_repo.as_ref()
        )?;
    }
    Commands::Code { no_danger } => {
        launch_claude_with_spec(
            spec_repo.as_ref(),
            config_repo.as_ref(),
            !no_danger
        )?;
    }
    // ...
}
```

## Use Case層の更新

各Use Caseは必要なリポジトリを直接受け取るように更新：

```rust
// application/use_cases/initialize_project.rs
pub fn initialize_project(
    config_repo: &dyn ConfigRepositoryInterface,
    spec_repo: &dyn SpecRepositoryInterface,
    steering_repo: &dyn SteeringRepositoryInterface,
    force: bool,
) -> Result<(), ApplicationError> {
    // プロジェクトが既に存在するかチェック
    if !force && steering_repo.exists()? {
        return Err(ApplicationError::ProjectAlreadyExists);
    }
    
    // 1. .kiroディレクトリとサブディレクトリを作成
    steering_repo.initialize_directories()?;
    spec_repo.initialize_directories()?;
    
    // 2. steering初期化
    steering_repo.initialize_steering()?;
    
    // 3. デフォルト設定を保存
    let config = ProjectConfig::default_for_new_project();
    config_repo.save_config(&config)?;
    
    // 4. steeringファイル作成
    steering_repo.create_steering_files(&config.steering)?;
    
    // 5. slash commands deploy
    steering_repo.deploy_slash_commands()?;
    
    // 6. gitignore更新
    steering_repo.update_gitignore()?;
    
    Ok(())
}

// application/use_cases/create_feature.rs
pub fn create_feature(
    spec_repo: &dyn SpecRepositoryInterface,
    name: &str,
) -> Result<(), ApplicationError> {
    // SpecRepositoryInterfaceのみ使用
    spec_repo.create_feature(name)
}

// application/use_cases/complete_features.rs
pub fn complete_features(
    spec_repo: &dyn SpecRepositoryInterface,
    config_repo: &dyn ConfigRepositoryInterface,
) -> Result<(), ApplicationError> {
    // TUIを表示してspecを選択
    let selected_specs = show_spec_selector(spec_repo)?;
    
    // 選択されたspecをアーカイブ
    for spec_name in selected_specs {
        spec_repo.mark_spec_complete(&spec_name)?;
    }
    
    Ok(())
}

// application/use_cases/launch_claude_with_spec.rs
pub fn launch_claude_with_spec(
    spec_repo: &dyn SpecRepositoryInterface,
    config_repo: &dyn ConfigRepositoryInterface,
    danger_mode: bool,
) -> Result<(), ApplicationError> {
    // 既存のspecを選択または新規作成
    let spec_name = select_or_create_spec(spec_repo)?;
    let spec_path = spec_repo.get_spec_path(&spec_name)?;
    
    // system promptを生成
    let system_prompt = generate_system_prompt(&spec_name, &spec_path)?;
    
    // Claude Codeを起動
    launch_claude_process(&system_prompt, danger_mode)?;
    
    Ok(())
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
        let repo = ConfigRepository::new(path_manager);
        
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
    let config_repo = Arc::new(ConfigRepository::new(path_manager.clone()));
    let spec_repo = Arc::new(SpecRepository::new(path_manager.clone()));
    let steering_repo = Arc::new(SteeringRepository::new(path_manager.clone()));
    
    // Use Caseを通じて初期化実行（複数リポジトリを渡す）
    initialize_project(
        config_repo.as_ref(),
        spec_repo.as_ref(),
        steering_repo.as_ref(),
        false
    ).unwrap();
    
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

### Phase 2: 既存ProjectRepositoryの削除と移行
1. 既存メソッドを各リポジトリに適切に分散
2. ProjectRepositoryトレイトと実装を削除
3. 必要に応じて新しいヘルパー関数をUse Case層に追加

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
7. **明示的な依存関係**: Use Caseレベルで必要なリポジトリが明確
8. **不要な抽象層の排除**: ProjectRepository Coordinatorを削除してシンプル化

## 潜在的な課題と対策

### 課題1: Use Caseの複雑化
- **問題**: 一部のUse Caseが複数のリポジトリを扱う必要がある
- **対策**: Use Case層で適切に調整、明示的な依存関係のメリットが上回る

### 課題2: パフォーマンス
- **問題**: ConfigRepositoryが頻繁に呼ばれる場合のI/O
- **対策**: 必要に応じてキャッシュ層を導入

### 課題3: 後方互換性
- **問題**: 既存のconfig.tomlとの互換性
- **対策**: Serdeのdefault属性で対応、マイグレーション処理を実装

### 課題4: gitignore更新の配置
- **問題**: gitignore更新がどのリポジトリに属するか不明確
- **対策**: SteeringRepositoryに含めるか、必要に応じて小さなGitignoreRepositoryを作成
