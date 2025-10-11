# 設計書: InitとNewコマンドの削除

## メタ情報
- **完成度**: 100%
- **要件**: `init`と`new`コマンドをCLIから削除（requirements.md#概要）
- **アーキテクチャスコープ**: CLI層の簡素化、コマンドルーティングの整理

## 概要

### 現状（As-Is）
- 3つのコマンド（`init`, `new`, `code`）が存在し、機能が重複
- `init`: プロジェクト初期化（267行のコード）
- `new`: スペック作成（509行のコード）
- 合計776行のコード + 23個のテスト

### 目標（To-Be）
- `code`コマンドのみを残し、すべての機能を提供
- CLIインターフェースを簡素化（3コマンド → 1コマンド）
- 508行のコード削除
- 23個のテスト削除（162テスト → 139テスト）

## 設計

### 削除対象の特定（investigation.md#cli-structure、#test-cleanup）

**完全削除するファイル（2ファイル）**:
1. `crates/hail-mary/src/cli/commands/init.rs` (267行)
2. `crates/hail-mary/src/cli/commands/new.rs` (241行)

**完全削除するユースケース（1ファイル）**:
1. `crates/hail-mary/src/application/use_cases/create_spec.rs` (280行)

理由: `new`コマンド専用で、`code`コマンドは`SpecValidator`を直接使用（investigation.md#new-command-usage）

---

## 実装詳細

### ファイル1: crates/hail-mary/src/cli/args.rs（更新）

**目的**: Commands列挙型から`Init`と`New`バリアントを削除（requirements.md#技術目標）

**変更内容**:

**削除する行**:
```rust
// 行14-21: Init と New バリアント削除
Init,
New {
    /// Spec name in kebab-case
    name: String,
},
```

**削除するヘルパーメソッド**:
```rust
// 行86-92: is_init() と is_new() 削除
pub fn is_init(&self) -> bool {
    matches!(self, Commands::Init)
}

pub fn is_new(&self) -> bool {
    matches!(self, Commands::New { .. })
}

// 行108-113: get_new_name() 削除
pub fn get_new_name(&self) -> Option<&str> {
    match self {
        Commands::New { name } => Some(name.as_str()),
        _ => None,
    }
}
```

**削除するテスト**:
```rust
// 行145-155: test_cli_parse_init_command 削除
// 行151-155: test_cli_parse_new_command 削除
```

---

### ファイル2: crates/hail-mary/src/cli/commands/mod.rs（更新）

**目的**: モジュール宣言と再エクスポートを削除

**削除する行**:
```rust
// 行4: mod init; 削除
// 行5: mod new; 削除
// 行12: pub use init::InitCommand; 削除
// 行13: pub use new::NewCommand; 削除
```

---

### ファイル3: crates/hail-mary/src/cli/mod.rs（更新）

**目的**: 公開APIから削除

**変更前**:
```rust
pub use commands::{InitCommand, NewCommand, ...};
```

**変更後**:
```rust
pub use commands::{...};  // InitCommand, NewCommand を削除
```

---

### ファイル4: crates/hail-mary/src/main.rs（更新）

**目的**: コマンドルーティングから削除

**削除するインポート**:
```rust
// 行4-5: InitCommand, NewCommand を削除
use hail_mary::cli::commands::{
    CodeCommand, CompleteCommand, // InitCommand, NewCommand 削除
    SteeringBackupCommand, SteeringRemindCommand, completion,
};
```

**削除するマッチアーム**:
```rust
// 行23-30: 削除
Commands::Init => {
    let command = InitCommand::new();
    command.execute()?;
}
Commands::New { name } => {
    let command = NewCommand::new(name);
    command.execute()?;
}
```

**削除するテスト**:
```rust
// 行111-165: test_main_init_command と test_main_init_command_idempotent 削除
// 行168-254: test_main_new_command、test_main_new_command_invalid_name、test_run_function_error_handling 削除
```

---

### ファイル5: crates/hail-mary/src/application/use_cases/mod.rs（更新）

**目的**: create_specユースケースを削除

**削除する行**:
```rust
// 行3: pub mod create_spec; 削除
// 行11: pub use create_spec::create_spec; 削除
```

---

### ファイル6: crates/hail-mary/src/cli/commands/complete.rs（更新）

**目的**: エラーメッセージを更新

**変更前**:
```rust
// 行30
format_error("Not in a project directory. Run 'hail-mary init' first.")
```

**変更後**:
```rust
format_error("Not in a project directory. Run 'hail-mary code' to initialize.")
```

---

### ファイル7: crates/hail-mary/tests/steering_integration_test.rs（更新）

**目的**: create_spec使用箇所を置き換え

**変更前**:
```rust
// 行5: インポート削除
use hail_mary::application::use_cases::{initialize_project, create_spec};

// 行44, 95: create_spec呼び出し削除
create_spec(&spec_repo, "test-feature").unwrap();
```

**変更後**:
```rust
// インポートからcreate_specを削除
use hail_mary::application::use_cases::initialize_project;

// 直接リポジトリメソッドを使用
spec_repo.create_spec("test-feature").unwrap();
```

---

### ファイル8: README.md（更新）

**目的**: ドキュメントの更新（requirements.md#技術目標）

**更新箇所**:

**1. Quick Start（行76-77）**:

**変更前**:
```markdown
# Initialize project
hail-mary init

# Create specification
hail-mary new user-auth
```

**変更後**:
```markdown
# Initialize and work with specs (unified command)
hail-mary code
```

**2. Project Initialization（行110-114）**:

**変更前**:
```markdown
## Project Initialization

Initialize a new Kiro project:
```bash
hail-mary init
```
```

**変更後**:
```markdown
## Project Initialization

Initialize a new Kiro project:
```bash
hail-mary code
```

The `code` command automatically initializes the project if needed (idempotent).
```

**3. Feature Specification Management（行130-141）**:

**変更前**:
```markdown
### Create New Specification

```bash
hail-mary new user-authentication-system
hail-mary new api-rate-limiting-v2
```
```

**変更後**:
```markdown
### Create New Specification

```bash
hail-mary code
# Interactive TUI: Navigate to "Create New" → Enter spec name
```
```

**4. Workflow Overview（行335-336）**:

**変更前**:
```markdown
## Specification Management Workflow

1. Initialize: `hail-mary init`
2. Create spec: `hail-mary new <feature-name>`
3. Work on spec
4. Complete: `hail-mary complete`
```

**変更後**:
```markdown
## Specification Management Workflow

1. Initialize and work: `hail-mary code` (unified command)
2. Complete: `hail-mary complete`
```

---

### ファイル9: .kiro/steering/tech.md（更新）

**目的**: Steering documentationの更新

**変更前**:
```markdown
### Application Usage
```bash
hail-mary init                           # Initialize project
hail-mary new <feature-name>             # Create feature specification
hail-mary code                           # Launch Claude Code
```
```

**変更後**:
```markdown
### Application Usage
```bash
hail-mary code                           # Initialize + Create/Select spec + Launch Claude
hail-mary complete                       # Mark specifications as complete
hail-mary steering backup                # Backup steering files
hail-mary steering remind <type> <topic> <content>  # Remind steering content
hail-mary shell-completions <shell>      # Generate shell completions
```
```

---

## テスト戦略

### 削除するテスト（23個）

**init.rs内のテスト（13個）**:
- `test_init_command_new`
- `test_init_command_execute_success`
- `test_init_command_is_idempotent`
- `test_init_command_partial_initialization`
- `test_init_command_creates_gitignore`
- `test_init_command_appends_to_existing_gitignore`
- `test_init_command_directory_structure`
- `test_init_command_config_content`
- `test_init_command_deploys_slash_commands`
- `test_init_command_overwrites_slash_commands`

**new.rs内のテスト（8個）**:
- `test_new_command_new`
- `test_new_command_execute_success`
- `test_new_command_execute_without_project`
- `test_new_command_execute_invalid_name`
- `test_new_command_execute_valid_names`
- `test_new_command_execute_duplicate_feature`
- `test_new_command_spec_path_format`
- `test_new_command_edge_cases`

**args.rs内のテスト（2個）**:
- `test_cli_parse_init_command`
- `test_cli_parse_new_command`

### 保持するテスト（139個）

**initialize_project.rs（9個）**: `code`コマンドが使用（investigation.md#test-cleanup）
**steering_integration_test.rs（7個）**: initialize_project関数を直接使用
**その他（123個）**: 影響なし

### テスト実行

```bash
# 完全テストスイート
just test

# 個別確認
cargo test --lib
cargo test --test steering_integration_test
```

**期待される結果**:
- 139テストが合格
- コンパイルエラーなし
- `just test`成功（fmt + lint + tests）

---

## ビルドと検証

### ビルドコマンド

```bash
just fmt      # フォーマット
just lint     # Lint実行
just build    # ビルド
just test     # 完全検証（推奨）
```

### 検証項目（requirements.md#受け入れ基準）

- [ ] `hail-mary init`コマンドがCLIから削除されている
- [ ] `hail-mary new`コマンドがCLIから削除されている
- [ ] CLIヘルプテキストに`init`や`new`コマンドが表示されない
- [ ] 削除されたコマンドを除いて全てのテストが通過
- [ ] `just test`が成功（fmt + lint + tests）
- [ ] コードベース内に非推奨コマンドへの参照がない（アーカイブドキュメントを除く）
- [ ] `hail-mary code`が以前の`init`/`new`のユースケースを全て処理できることを確認
- [ ] シェル補完が更新され、削除されたコマンドを提案しない

**シェル補完の自動更新**:

Clapの`clap_complete`が`Commands`列挙型から自動生成するため、手動更新は不要（investigation.md#cli-structure）

---

## 実装順序

1. **ファイル削除** (3ファイル):
   - `crates/hail-mary/src/cli/commands/init.rs`
   - `crates/hail-mary/src/cli/commands/new.rs`
   - `crates/hail-mary/src/application/use_cases/create_spec.rs`

2. **コア更新** (5ファイル):
   - `crates/hail-mary/src/cli/args.rs` - 列挙型とヘルパー削除
   - `crates/hail-mary/src/cli/commands/mod.rs` - モジュール削除
   - `crates/hail-mary/src/cli/mod.rs` - エクスポート削除
   - `crates/hail-mary/src/main.rs` - ルーティング削除
   - `crates/hail-mary/src/application/use_cases/mod.rs` - ユースケース削除

3. **依存更新** (2ファイル):
   - `crates/hail-mary/src/cli/commands/complete.rs` - エラーメッセージ更新
   - `crates/hail-mary/tests/steering_integration_test.rs` - create_spec置き換え

4. **ドキュメント更新** (2ファイル):
   - `README.md` - ユーザー向けドキュメント
   - `.kiro/steering/tech.md` - Steering documentation

5. **検証**:
   - `just test` - すべての検証項目をクリア

---

## リスク管理

### 低リスク
- ファイルは独立しており、他への依存なし（investigation.md#init-command-usage、#new-command-usage）
- `initialize_project`と`SpecValidator`は保持（`code`コマンドが使用）
- テストヘルパーは全保持（他のテストが使用）
- シェル補完は自動更新

### 破壊的変更
- 既存ユーザーの`init`/`new`コマンド使用が不可能になる
- 自動化スクリプトが動作しなくなる

### 軽減策（requirements.md#リスク評価）
- リリースノートで明示
- `code`コマンドの使用方法を明確に案内
- ロールバック計画: git revert可能

---

## 成功基準

**完了条件**:
- [ ] 全ファイルが更新済み（合計12ファイル）
- [ ] 508行のコード削除
- [ ] 23テスト削除、139テスト合格
- [ ] `just test`成功
- [ ] ドキュメント更新完了
- [ ] `hail-mary --help`に`init`/`new`が表示されない

---

## 参考資料

- requirements.md#概要: 技術要件の定義
- requirements.md#受け入れ基準: 完成基準
- investigation.md#init-command-usage: Init command分析
- investigation.md#new-command-usage: New command分析
- investigation.md#cli-structure: CLI構造と更新箇所
- investigation.md#test-cleanup: テストクリーンアップ戦略
