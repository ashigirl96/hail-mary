# Acceptance Criteria for Clean Architecture Redesign

## Overview
This document defines the acceptance criteria for the Clean Architecture redesign of the hail-mary project as specified in design-v2.md.

## 🏗️ Architecture Structure
- [ ] **Layer Separation**: Domain, Application, CLI, Infrastructure層が明確に分離されている
- [ ] **Dependency Rule**: 依存関係が内側に向かっている（Domain ← Application ← CLI/Infrastructure）
- [ ] **No Circular Dependencies**: 循環依存が存在しない
- [ ] **Module Organization**: src/domain/, src/application/, src/cli/, src/infrastructure/のディレクトリ構造

## 📦 Domain Layer
- [ ] **Pure Domain Entities**: Memory, Feature, ProjectConfigがビジネスロジックのみを含む
- [ ] **Value Objects**: Confidence value objectが0.0-1.0の検証を実装
- [ ] **No External Dependencies**: Domain層が外部ライブラリに依存していない
- [ ] **Domain Errors**: DomainError型が定義され、ドメイン固有のエラーを表現

## 🔧 Application Layer
- [ ] **Use Case Functions**: 各ユースケースが純粋関数として実装されている
- [ ] **Repository Traits**: MemoryRepository, ProjectRepositoryトレイトが定義されている
- [ ] **Business Logic**: ビジネスロジックがuse cases内に集約されている
- [ ] **Application Errors**: ApplicationError型が適切に定義されている

## 💾 Infrastructure Layer
- [ ] **Repository Implementations**: SqliteMemoryRepository, FilesystemProjectRepositoryが実装されている
- [ ] **Database Migrations**: Refineryによる埋め込みマイグレーションが動作する
- [ ] **FTS5 Search**: 日本語トークナイゼーションを含むFTS5検索が機能する
- [ ] **PathManager**: 集中化されたパス管理が実装されている
- [ ] **Transaction Support**: バッチ操作でトランザクションが使用されている

## 🖥️ CLI Layer
- [ ] **Command Structure**: Commands enumでCLIコマンドが定義されている
- [ ] **Output Formatter**: OutputFormatterで出力形式（Text/Json/Markdown）が統一されている
- [ ] **Error Handling**: anyhow::Resultでエラーが適切に伝播される
- [ ] **Argument Parsing**: clapでコマンドライン引数が解析される

## 🔌 Dependency Injection
- [ ] **Smart Arc Usage**: Arc<Mutex<>>はMCPサーバーのみで使用される
- [ ] **Synchronous by Default**: 非同期は必要な箇所のみ（MCPサーバー）
- [ ] **Manual DI in main.rs**: main.rsで依存関係が明示的に構築される
- [ ] **No DI Container**: DIコンテナを使用せず、シンプルな手動注入

## 🧪 Testing
- [ ] **Unit Tests**: Domain層とApplication層が独立してテスト可能
- [ ] **Repository Mocks**: MockRepositoryでuse casesがテスト可能
- [ ] **Integration Tests**: SQLite統合テストが一時DBで実行される
- [ ] **E2E Tests**: CLIコマンドのエンドツーエンドテスト

## 📊 Performance & Quality
- [ ] **Minimal Overhead**: 不要なArc wrappingがない
- [ ] **Direct Calls**: 非同期が不要な箇所は同期呼び出し
- [ ] **Resource Efficiency**: リポジトリがオンデマンドで作成される
- [ ] **Code Coverage**: 80%以上のテストカバレッジ

## 🔄 Migration Safety
- [ ] **Backward Compatibility**: 既存のデータベースと互換性がある
- [ ] **Data Migration**: 既存のメモリデータが保持される
- [ ] **Config Migration**: .kiro/config.tomlが引き続き機能する
- [ ] **Feature Parity**: 現在の全機能が新アーキテクチャで動作する

## 📚 Documentation
- [ ] **Architecture Documentation**: 各層の責務が文書化されている
- [ ] **API Documentation**: 公開インターフェースにrustdocコメント
- [ ] **Migration Guide**: 移行手順が文書化されている
- [ ] **Example Usage**: 各コマンドの使用例が提供されている

## 🚀 MCP Server
- [ ] **Partial Async**: MCPサーバーのみ非同期で実装
- [ ] **rmcp Integration**: rmcp 0.5.0でMCPプロトコルが実装される
- [ ] **Tool Registration**: remember/recallツールが登録される
- [ ] **JSON Schema**: schemasでAPIスキーマが生成される

## Validation Process

### Phase 1: Structure Validation
1. Verify directory structure matches design
2. Check dependency directions with `cargo tree`
3. Ensure no circular dependencies exist

### Phase 2: Implementation Validation
1. Test each use case function independently
2. Verify repository trait implementations
3. Confirm database migrations work correctly

### Phase 3: Integration Validation
1. Test CLI commands end-to-end
2. Verify MCP server functionality
3. Check backward compatibility with existing data

### Phase 4: Quality Validation
1. Run test coverage analysis
2. Perform performance benchmarks
3. Review code against Rust best practices
4. Update CLAUDE.md and README.md with new architecture details

## Success Metrics
- All checkboxes above are checked ✓
- Zero compilation warnings
- Test coverage > 80%
- All existing features continue to work
- Performance benchmarks show no regression

## Notes
- This document should be updated as implementation progresses
- Each checkbox represents a deliverable that can be independently verified
- Priority should be given to maintaining backward compatibility