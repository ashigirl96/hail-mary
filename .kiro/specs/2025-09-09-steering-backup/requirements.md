# Requirements: Steering Backup Command

## Overview
`hail-mary steering backup` コマンドを追加し、steeringディレクトリのマークダウンファイルをタイムスタンプ付きバックアップとして保存する機能を提供する。自動ローテーション機能により、設定された上限数を超えたバックアップは古いものから自動削除される。

## Purpose
- **データ保護**: steering情報の誤削除・誤編集からの保護
- **履歴管理**: steering情報の変更履歴を追跡可能に
- **自動ローテーション**: ディスク容量を考慮した古いバックアップの自動削除

## User Stories
- As a developer, I want to backup current steering files so that I can protect against accidental data loss
- As a developer, I want to limit the number of backups so that disk space is managed efficiently
- As a developer, I want timestamped backups so that I can restore steering files to a specific point in time

## Acceptance Criteria
- [ ] `hail-mary steering backup` コマンドでsteeringファイルがバックアップされる
- [ ] バックアップは `.kiro/steering/backup/<YYYY-MM-dd-HH-mm>/` に保存される
- [ ] バックアップ対象は `.kiro/steering/*.md`（backup/とdraft/ディレクトリは除外）
- [ ] バックアップ数が設定値を超えた場合、最古のバックアップが自動削除される
- [ ] `hail-mary init` 実行時に `[steering.backup]` セクションが自動追加される（既存設定は保持）
- [ ] デフォルトの最大バックアップ数は10
- [ ] エラーハンドリングが適切に実装されている

## Technical Requirements

### Directory Structure
```
.kiro/
├── steering/
│   ├── product.md
│   ├── tech.md
│   ├── structure.md
│   ├── backup/
│   │   ├── 2025-09-09-14-30/
│   │   │   ├── product.md
│   │   │   ├── tech.md
│   │   │   └── structure.md
│   │   └── 2025-09-09-15-45/
│   │       └── ...
│   └── draft/
└── config.toml
```

### Configuration Format
```toml
[steering.backup]
max = 10  # Maximum number of backups to retain
```

### Implementation Components
- **CLI Command**: New `backup` subcommand under steering commands
- **Use Case**: `backup_steering` function in application layer
- **Repository**: Extend `ProjectRepository` trait with backup operations
- **Domain Entity**: Add backup configuration to `SteeringConfig`
- **Error Handling**: New error types for backup operations

## Priority
1. **P0**: Basic backup command functionality
2. **P0**: Automatic rotation based on max configuration
3. **P0**: Configuration management during init
4. **P1**: List backups command (future enhancement)
5. **P2**: Restore from backup command (future enhancement)

## Risks and Mitigations
- **Risk**: Data loss during backup process
  - **Mitigation**: Transactional processing, rollback on error
- **Risk**: Configuration file corruption
  - **Mitigation**: Backup before update, validation of TOML structure
- **Risk**: Filesystem permission issues
  - **Mitigation**: Proper error handling and user-friendly error messages
