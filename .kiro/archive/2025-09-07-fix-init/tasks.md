# Tasks

## References
- design.md: 冪等な init コマンドの設計
- investigation.md: 現在の実装の調査結果

## Phase 1: CLI層の変更
- [ ] `cli/args.rs` から `--force` フラグを削除
  - `Commands::Init { force: bool }` → `Commands::Init` に変更
  - `is_init()` メソッドの更新
  - `get_force_flag()` メソッドの削除または修正
- [ ] `cli/commands/init.rs` から force フィールドを削除
  - `InitCommand` 構造体から `force: bool` フィールドを削除
  - `new(force: bool)` → `new()` に変更
  - `execute()` メソッドの更新（force パラメータを渡さない）

## Phase 2: Application層の変更
- [ ] `application/use_cases/initialize_project.rs` を冪等に修正
  - 関数シグネチャから `force: bool` パラメータを削除
  - `ProjectAlreadyExists` エラーチェックを削除
  - シンプルな実装に変更（5行程度）
- [ ] `application/errors.rs` から `ProjectAlreadyExists` エラーを削除（不要になった場合）

## Phase 3: Infrastructure層の変更（冪等化）
- [ ] `infrastructure/repositories/project.rs` の `initialize()` を冪等に修正
  - `.kiro` ディレクトリが存在してもエラーにしない
  - `.kiro/specs` ディレクトリが存在してもエラーにしない
- [ ] `save_config()` を冪等に修正（既に実装済み - 確認のみ）
  - 既存の config.toml があれば何もしない（既に実装済み）
- [ ] `initialize_steering()` を冪等に修正（確認）
  - 既存のディレクトリがあっても続行
- [ ] `create_steering_files()` を冪等に修正（既に実装済み - 確認のみ）
  - 既存のファイルがあればスキップ（既に実装済み）
- [ ] `update_gitignore()` を冪等に修正（確認）
  - 既存の .gitignore があっても続行
- [ ] `ensure_steering_config()` メソッドを削除または簡略化
  - 不要になる可能性があるため確認

## Phase 4: テストの更新
- [ ] `cli/commands/init.rs` のテストを更新
  - `test_init_command_new()` を修正（force パラメータ削除）
  - `test_init_command_execute_already_exists` を削除
  - `test_init_command_execute_with_force` を削除
- [ ] 新しいテスト `test_init_is_idempotent` を追加
  - 複数回実行してもエラーにならないことを確認
- [ ] 新しいテスト `test_init_partial_initialization` を追加
  - 部分的に初期化されたプロジェクトを修復できることを確認
- [ ] `application/use_cases/initialize_project.rs` のテストを更新
  - force パラメータ関連のテストを削除
  - MockProjectRepository の更新
- [ ] `infrastructure/repositories/project.rs` のテストを更新
  - 冪等性を確認するテストを追加

## Phase 5: 出力メッセージの改善（オプション）
- [ ] Created/Already exists の表示を実装するか決定
  - 案1: インラインで println! を追加
  - 案2: シンプルに "Initialization complete." のみ
- [ ] 選択した案を実装

## Phase 6: 動作確認とドキュメント更新
- [ ] `just fix` でコードフォーマットとlintを実行
- [ ] `just ci` でフォーマットチェック、lint、テストをすべて実行
- [ ] `just build-release` でリリースビルドの確認
- [ ] 手動で `hail-mary init` を複数回実行して冪等性を確認
- [ ] README.md やヘルプメッセージから `--force` の記述を削除（もしあれば）
- [ ] CLAUDE.md の更新（もし必要なら）

## 完了条件
- `hail-mary init` が何度実行してもエラーにならない
- 既存のファイルが上書きされない
- 部分的に初期化されたプロジェクトを修復できる
- すべてのテストが通る
