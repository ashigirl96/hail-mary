# Hail-Mary v3 アーキテクチャ実装タスクリスト

## 📋 実装管理情報

- **開始日**: 2025-08-17
- **目標完了日**: 2025-10-12 (8週間)
- **現在のステータス**: 🔵 Planning
- **最終更新**: 2025-08-17

## 🎯 Phase 1: FTS5ロジック保護 (Week 1) ⭐最優先

### 準備作業
- [ ] 現在の検索パフォーマンスをベンチマーク測定
- [ ] 日本語テキストのテストコーパス作成（最低100ケース）
- [ ] 既存の検索クエリパターンを文書化

### FtsQueryBuilder実装
- [ ] `src/database/query_builders/` ディレクトリ作成
- [ ] `fts_query_builder.rs` ファイル作成
- [ ] `normalize_content_for_fts()` 関数を移植（lines 45-73）
- [ ] `enhance_query_for_partial_match()` 関数を移植（lines 75-166）
- [ ] QueryConstraint enum 実装
- [ ] QueryOptions struct 実装
- [ ] OptimizedQuery struct 実装
- [ ] QueryComplexity enum 実装

### テスト実装
- [ ] 日本語・英語境界テスト
- [ ] ハイフン処理テスト
- [ ] 名前空間演算子(::)テスト
- [ ] 特殊文字エスケープテスト
- [ ] ワイルドカード処理テスト
- [ ] ブーリアン演算子テスト
- [ ] パフォーマンステスト（ベンチマーク）

### 検証
- [ ] 既存の検索クエリとの互換性確認（100%一致）
- [ ] パフォーマンス劣化なし（±5%以内）
- [ ] メモリ使用量の計測

## 📝 Phase 2: エラー階層構築 (Week 2)

### エラー型定義
- [ ] `src/business/domain/errors.rs` ファイル作成
- [ ] DomainError enum 実装
  - [ ] InvalidMemoryId
  - [ ] EmptyTitle/EmptyContent
  - [ ] TitleTooLong
  - [ ] TooManyTags
  - [ ] InvalidTag
  - [ ] UnsupportedMemoryType
- [ ] OperationError enum 実装
  - [ ] MemoryNotFound
  - [ ] DuplicateMemory
  - [ ] MergeConflict
  - [ ] BulkOperationPartialFailure
  - [ ] OperationTimeout
- [ ] SearchError enum 実装
  - [ ] InvalidQuery
  - [ ] QueryTooComplex
  - [ ] NoResults
  - [ ] SearchTimeout
- [ ] AnalyticsError enum 実装
  - [ ] InsufficientData
  - [ ] ClusteringFailed
  - [ ] ArchiveFailed
- [ ] InfrastructureError enum 実装
  - [ ] DatabaseConnection
  - [ ] EmbeddingServiceUnavailable
  - [ ] McpServerError

### エラーユーティリティ
- [ ] HailMaryError 統合型実装
- [ ] `is_recoverable()` メソッド実装
- [ ] `retry_strategy()` メソッド実装
- [ ] RetryStrategy enum 実装
- [ ] エラーコンテキスト保持の実装

### 移行作業
- [ ] 既存エラーからのマッピング表作成
- [ ] 移行スクリプト作成
- [ ] エラーハンドリングのベストプラクティス文書化

## 🔧 Phase 3: Operations層分割 (Week 3-4)

### Week 3: Core & Search Operations

#### CoreMemoryOperations
- [ ] `src/business/operations/core/` ディレクトリ作成
- [ ] `memory_ops.rs` ファイル作成（<150行）
- [ ] `upsert_memory()` メソッド実装
- [ ] `get_memory()` メソッド実装
- [ ] `delete_memory()` メソッド実装
- [ ] `list_memories()` メソッド実装
- [ ] ユニットテスト作成

#### TextSearchOperations
- [ ] `src/business/operations/search/` ディレクトリ作成
- [ ] `text_search.rs` ファイル作成（<180行）
- [ ] `search_with_scoring()` メソッド実装
- [ ] `search()` メソッド実装
- [ ] `search_by_tags()` メソッド実装
- [ ] キャッシュ統合
- [ ] ユニットテスト作成

### Week 4: Analytics Operations & Integration

#### ClusteringOperations
- [ ] `src/business/operations/analytics/` ディレクトリ作成
- [ ] `clustering.rs` ファイル作成（<200行）
- [ ] `merge_duplicates()` メソッド実装
- [ ] `archive_old_memories()` メソッド実装
- [ ] `cluster_memories()` メソッド実装
- [ ] MergeReport struct 実装
- [ ] ArchiveReport struct 実装
- [ ] MemoryCluster struct 実装
- [ ] ユニットテスト作成

#### 統合作業
- [ ] 既存サービスからの移行計画作成
- [ ] 並行実行期間の設定
- [ ] フィーチャーフラグ実装
- [ ] 段階的切り替えスクリプト作成

## ⚡ Phase 4: パフォーマンス最適化DI (Week 5)

### 依存性注入実装
- [ ] OptimizedOperations struct 実装（具象型使用）
- [ ] FlexibleOperations struct 実装（trait objects）
- [ ] OperationsMode enum 実装
  - [ ] Production variant
  - [ ] Testing variant
  - [ ] Development variant
- [ ] `for_environment()` メソッド実装

### Repository実装の最適化
- [ ] SqliteMemoryRepository の具象型版作成
- [ ] MockMemoryRepository 実装
- [ ] Repository trait の最適化

### パフォーマンス測定
- [ ] Arc<dyn> vs 具象型のベンチマーク
- [ ] メモリアロケーション分析
- [ ] CPU使用率プロファイリング
- [ ] 最適化前後の比較レポート作成

## 📊 Phase 5: 監視とキャッシュ (Week 6)

### PerformanceMonitor実装
- [ ] `src/infrastructure/monitoring/` ディレクトリ作成
- [ ] `metrics.rs` ファイル作成
- [ ] `record_operation()` メソッド実装
- [ ] `record_cache_hit/miss()` メソッド実装
- [ ] `generate_report()` メソッド実装
- [ ] PerformanceReport struct 実装
- [ ] OperationStats struct 実装

### QueryCache実装
- [ ] `src/database/cache/` ディレクトリ作成
- [ ] `query_cache.rs` ファイル作成
- [ ] LRUキャッシュアルゴリズム実装
- [ ] キャッシュ無効化戦略実装
- [ ] TTL（Time To Live）実装
- [ ] キャッシュメトリクス収集

### 監視ダッシュボード
- [ ] メトリクス収集パイプライン構築
- [ ] Grafanaダッシュボード設定（オプション）
- [ ] アラート設定
- [ ] パフォーマンスレポート自動生成

## 🧪 Phase 6: 統合テストと最適化 (Week 7-8)

### Week 7: テスト実装

#### ユニットテスト
- [ ] Domain層テスト完成（カバレッジ95%以上）
- [ ] Operations層テスト完成（カバレッジ90%以上）
- [ ] Database層テスト完成（カバレッジ85%以上）
- [ ] Infrastructure層テスト完成（カバレッジ80%以上）

#### 統合テスト
- [ ] E2Eテストシナリオ作成
- [ ] 検索フロー統合テスト
- [ ] CRUD操作統合テスト
- [ ] エラーハンドリング統合テスト
- [ ] パフォーマンステスト自動化

### Week 8: 最適化と仕上げ

#### パフォーマンス最適化
- [ ] ボトルネック分析
- [ ] クエリ最適化
- [ ] インデックス最適化
- [ ] キャッシュチューニング
- [ ] メモリ使用量最適化

#### ドキュメント作成
- [ ] アーキテクチャドキュメント更新
- [ ] API仕様書作成
- [ ] 移行ガイド作成
- [ ] トラブルシューティングガイド作成
- [ ] パフォーマンスチューニングガイド作成

#### リリース準備
- [ ] ロールバック計画の最終確認
- [ ] デプロイメントスクリプト作成
- [ ] モニタリングアラート設定
- [ ] チームトレーニング資料作成
- [ ] ステークホルダー向けプレゼン資料作成

## 🎯 成功基準チェックリスト

### パフォーマンス指標
- [ ] 検索レスポンス P95 < 200ms
- [ ] キャッシュヒット率 > 70%
- [ ] メモリ使用量 -20%達成
- [ ] ビルド時間 < 45秒

### 品質指標
- [ ] テストカバレッジ 90%以上
- [ ] 各モジュール200行以下
- [ ] エラー回復率 80%以上
- [ ] ドキュメント完成度 100%

### 運用指標
- [ ] ロールバック時間 < 5分
- [ ] アラート設定完了
- [ ] 監視ダッシュボード稼働
- [ ] チームトレーニング完了

## 📅 マイルストーン

| マイルストーン | 期限 | ステータス | 承認者 |
|--------------|------|------------|--------|
| FTS5ロジック保護完了 | Week 1 | ⏳ 進行中 | Tech Lead |
| エラー階層実装完了 | Week 2 | 🔵 未着手 | Tech Lead |
| Operations層分割完了 | Week 4 | 🔵 未着手 | Tech Lead |
| パフォーマンスDI完了 | Week 5 | 🔵 未着手 | Tech Lead |
| 監視システム稼働 | Week 6 | 🔵 未着手 | DevOps Lead |
| 統合テスト完了 | Week 7 | 🔵 未着手 | QA Lead |
| v3リリース準備完了 | Week 8 | 🔵 未着手 | PM |

## 🚨 リスク管理タスク

### 継続的モニタリング
- [ ] 週次進捗レビュー会議設定
- [ ] リスクレジスター更新（週次）
- [ ] ステークホルダー報告（隔週）
- [ ] 技術的負債の追跡

### コンティンジェンシープラン
- [ ] ロールバック手順書作成
- [ ] データバックアップ自動化
- [ ] フィーチャーフラグ設定
- [ ] 並行稼働環境準備

## 📝 Notes

### 優先度の考え方
1. **P0（最優先）**: FTS5ロジック保護 - 検索機能の根幹
2. **P1（高）**: エラー階層、Operations層分割 - アーキテクチャの基盤
3. **P2（中）**: パフォーマンスDI、監視 - 品質向上
4. **P3（低）**: ドキュメント、追加最適化 - 長期保守性

### 依存関係
- Operations層はエラー階層に依存
- 監視システムはOperations層に依存
- 統合テストはすべてのPhaseに依存

### 並行作業可能項目
- エラー階層とFTS5ロジックは並行開発可能
- ドキュメント作成は各Phase完了後即座に開始可能
- テスト作成は実装と並行して進行可能

---

**最終更新者**: Architecture Team  
**次回レビュー**: 2025-08-24