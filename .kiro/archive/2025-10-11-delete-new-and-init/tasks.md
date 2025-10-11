# Tasks

## Required Investigations
- [x] init-command-usage → investigation.md#init-command-usage
- [x] new-command-usage → investigation.md#new-command-usage
- [x] code-command-coverage → investigation.md#code-command-coverage
- [x] cli-structure → investigation.md#cli-structure
- [x] test-cleanup → investigation.md#test-cleanup

## State Tracking

| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | complete | - | - |
| investigation.md | complete | 5/5 (100%) | - |
| design.md | complete | - | - |
| tasks.md#Timeline | complete | 5/5 phases (100%) | - |

## Timeline

- [x] Spec created → delete-new-and-init
- [x] Requirements defined → requirements.md#概要
- [x] All investigations complete → investigation.md
- [x] Design completed → design.md#概要
- [x] Implementation plan agreed with user
- [x] phase1: ファイル削除 → design.md#実装順序
  - [x] Delete crates/hail-mary/src/cli/commands/init.rs
  - [x] Delete crates/hail-mary/src/cli/commands/new.rs
  - [x] Delete crates/hail-mary/src/application/use_cases/create_spec.rs
- [x] phase2: CLI層更新 → design.md#実装詳細
  - [x] Update crates/hail-mary/src/cli/args.rs (Commands列挙型とヘルパー削除)
  - [x] Update crates/hail-mary/src/cli/commands/mod.rs (モジュール削除)
  - [x] Update crates/hail-mary/src/cli/mod.rs (エクスポート削除)
  - [x] Update crates/hail-mary/src/main.rs (ルーティング削除)
  - [x] Update crates/hail-mary/src/application/use_cases/mod.rs (ユースケース削除)
- [x] phase3: 依存箇所更新 → design.md#実装詳細
  - [x] Update crates/hail-mary/src/cli/commands/complete.rs (エラーメッセージ)
  - [x] Update crates/hail-mary/tests/steering_integration_test.rs (create_spec置き換え)
- [x] phase4: ドキュメント更新 → design.md#実装詳細
  - [x] Update README.md (ユーザー向けガイド)
  - [x] Update .kiro/steering/tech.md (コマンドリファレンス)
- [x] phase5: 検証とテスト → design.md#ビルドと検証
  - [x] Run just test (全検証項目クリア)
  - [x] Verify 受け入れ基準 (requirements.md#受け入れ基準)
