# Tasks: Steering Backup Command Implementation

## 実装方針
- **TDD (Test-Driven Development)**: Red-Green-Refactor サイクルを厳守
- **Inside-Out アプローチ**: ドメイン層から外側へ向かって実装
- **継続的検証**: 各ステップで `just fix` (フォーマット) と `just ci` (フルテスト) を実行

## Phase 1: Domain Layer - SteeringBackupConfig (30分)

### RED: テスト作成
- [ ] `domain/entities/steering.rs` にSteeringBackupConfigのテストを追加
  - [ ] `test_steering_backup_config_default()` - Default値が10であることを確認
  - [ ] `test_steering_backup_config_clone()` - Cloneトレイトの動作確認
  - [ ] `test_steering_backup_config_debug()` - Debugトレイトの動作確認
  - [ ] `test_steering_backup_config_partial_eq()` - PartialEqトレイトの動作確認

### GREEN: 実装
- [ ] SteeringBackupConfig構造体を追加
  ```rust
  #[derive(Debug, Clone, PartialEq)]
  pub struct SteeringBackupConfig {
      pub max: usize,
  }
  ```
- [ ] Default実装を追加
- [ ] SteeringConfigにbackupフィールドを追加
- [ ] `just ci` で全テストがパスすることを確認

### REFACTOR
- [ ] コードの重複を除去
- [ ] `just fix` でフォーマット

## Phase 2: Application Layer - Repository Trait (20分)

### RED: テスト作成
- [ ] `application/repositories/project_repository.rs` のテストモジュールを更新
  - [ ] `test_list_steering_files()` - MockRepositoryでのテスト
  - [ ] `test_create_steering_backup()` - バックアップ作成のモックテスト
  - [ ] `test_list_steering_backups()` - バックアップリストのモックテスト
  - [ ] `test_delete_oldest_steering_backups()` - 削除のモックテスト
  - [ ] `test_load_steering_backup_config()` - 設定読み込みのモックテスト

### GREEN: 実装
- [ ] ProjectRepositoryトレイトに5つの新メソッドを追加
- [ ] BackupInfo構造体を定義
- [ ] MockProjectRepositoryに対応するモック実装を追加
- [ ] `just ci` でテスト確認

## Phase 3: Infrastructure Layer - Repository実装 (60分)

### RED: 統合テスト作成
- [ ] `tests/steering_backup_test.rs` を新規作成
- [ ] `test_list_steering_files_excludes_subdirs()` - backup/draft除外確認
- [ ] `test_create_steering_backup_with_timestamp()` - タイムスタンプディレクトリ作成
- [ ] `test_list_steering_backups_sorted_by_time()` - 作成時刻順ソート確認
- [ ] `test_delete_oldest_steering_backups()` - 古い順に削除
- [ ] `test_load_steering_backup_config_from_toml()` - TOML解析
- [ ] `test_load_steering_backup_config_returns_default()` - 設定なし時のデフォルト値

### GREEN: 実装
- [ ] `infrastructure/repositories/project.rs` に各メソッドを実装
  - [ ] `list_steering_files()` - *.mdファイルのみ、ディレクトリ除外
  - [ ] `create_steering_backup()` - タイムスタンプディレクトリ作成とファイルコピー
  - [ ] `list_steering_backups()` - backup/内のディレクトリを作成時刻順にリスト
  - [ ] `delete_oldest_steering_backups()` - 指定数の古いバックアップを削除
  - [ ] `load_steering_backup_config()` - config.tomlから設定読み込み
- [ ] PathManagerに`steering_backup_dir()`メソッドを追加
- [ ] TOML構造体（BackupSection）を定義
- [ ] `just ci` で統合テストがパスすることを確認

## Phase 4: Application Layer - Use Case (40分)

### RED: Use Caseテスト作成
- [ ] `application/use_cases/mod.rs` にbackup_steeringモジュールを追加
- [ ] `application/use_cases/backup_steering.rs` を新規作成
- [ ] テストモジュールを作成
  - [ ] `test_backup_steering_success()` - 正常系バックアップ作成
  - [ ] `test_backup_steering_with_rotation()` - max超過時の自動削除
  - [ ] `test_backup_steering_no_files()` - ファイルなしエラー
  - [ ] `test_backup_steering_filesystem_error()` - ファイルシステムエラー

### GREEN: 実装
- [ ] backup_steering関数を実装
  1. 設定読み込み
  2. steeringファイルリスト取得
  3. タイムスタンプ生成
  4. バックアップディレクトリ作成
  5. ローテーション処理
- [ ] ApplicationErrorに新しいエラー型を追加
- [ ] `just fix` と `just ci` で確認

## Phase 5: CLI Layer - コマンド実装 (30分)

### RED: CLIテスト作成
- [ ] `cli/args.rs` のテストモジュールを更新
  - [ ] `test_steering_command_parsing()` - コマンドパースのテスト
  - [ ] `test_steering_backup_subcommand()` - backupサブコマンドのテスト
- [ ] `cli/commands/steering_backup.rs` のテスト作成
  - [ ] `test_execute_backup_command()` - コマンド実行のテスト

### GREEN: 実装
- [ ] `cli/args.rs` にSteeringサブコマンドを追加
  ```rust
  Steering {
      #[command(subcommand)]
      command: SteeringCommands,
  }
  ```
- [ ] SteeringCommands列挙型を定義
- [ ] `cli/commands/steering_backup.rs` を新規作成
- [ ] `cli/commands/mod.rs` でモジュールをエクスポート
- [ ] `main.rs` でコマンドルーティングを追加
- [ ] `just ci` で全体テスト確認

## Phase 6: Init Command更新 (30分)

### RED: Init更新テスト
- [ ] `tests/init_steering_backup_test.rs` を新規作成
- [ ] `test_init_adds_backup_config()` - 新規作成時のデフォルト値テスト
- [ ] `test_init_preserves_existing_backup_config()` - 既存設定保持テスト
- [ ] `test_ensure_steering_backup_config()` - 既存config.tomlへの追加テスト

### GREEN: 実装
- [ ] `infrastructure/repositories/project.rs` にensure_steering_backup_configメソッドを実装
- [ ] ProjectRepositoryトレイトにメソッドを追加
- [ ] `application/use_cases/initialize_project.rs` から呼び出し
- [ ] `just fix` と `just ci` で確認

## Phase 7: E2Eテスト (40分)

### 統合動作確認
- [ ] `tests/e2e_steering_backup.rs` を新規作成
- [ ] 実際のコマンド実行テスト
  - [ ] `test_init_creates_backup_config()` - initでconfig.tomlに[steering.backup]追加
  - [ ] `test_backup_command_creates_directory()` - バックアップディレクトリ作成
  - [ ] `test_backup_rotation()` - 複数回実行でローテーション動作
  - [ ] `test_backup_preserves_file_contents()` - ファイル内容が保持される
- [ ] `just ci` で全テストパス確認

### 手動テスト
- [ ] 実際のプロジェクトでコマンド実行
- [ ] バックアップディレクトリの確認
- [ ] ローテーション動作の確認

## Phase 8: ドキュメント更新 (20分)

- [ ] `CLAUDE.md` を更新
  - [ ] CLI Commandsセクションに`hail-mary steering backup`を追加
  - [ ] Steering System Workflowにバックアップ機能を追加
- [ ] `README.md` を更新（必要に応じて）
  - [ ] 使用例を追加
  - [ ] コマンド一覧を更新
- [ ] `just fix` で最終フォーマット
- [ ] `just ci` で最終確認

## 検証ポイント

### 各フェーズ共通
- [ ] `just ci` を実行して既存テストが壊れていないことを確認
- [ ] テストカバレッジが低下していないことを確認
- [ ] エラーメッセージが適切でユーザーフレンドリーであることを確認

### 最終確認
- [ ] すべてのテストがグリーン
- [ ] ドキュメントが最新
- [ ] コードフォーマットが統一されている
- [ ] 不要なコメントやデバッグコードが削除されている

## 所要時間
- 合計: 約4時間（各フェーズのバッファ含む）
- 実装: 3時間20分
- テスト・検証: 40分

## 注意事項
- `cargo` コマンドは使用せず、必ず `just fix` と `just ci` を使用する
- 各フェーズでRed-Green-Refactorサイクルを守る
- テストが失敗している状態でも、まずコミットしてからGreenフェーズに進む（TDDの原則）