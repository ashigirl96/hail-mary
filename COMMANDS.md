# hail-mary コマンドリファレンス

このドキュメントでは、hail-maryの全コマンドとそのオプション、使用例について説明します。

## メインコマンド

### 1. `new` - 新しい機能を作成
**目的**: 新しいプロジェクト機能を作成し、必要なファイル構造を生成

**引数**:
- `<FEATURE_NAME>` - 機能名（kebab-case形式で指定）

**オプション**:
- なし

**使用例**:
```bash
hail-mary new my-awesome-feature
```

**作成されるファイル**:
- `requirements.md` - 要件定義
- `design.md` - 設計ドキュメント  
- `task.md` - タスクリスト
- `spec.json` - 仕様書

## メモリコマンド

### 2. `memory serve` - MCPサーバーを開始
**目的**: メモリ管理用のMCP（Model Context Protocol）サーバーを開始

**オプション**:
- `--db-path <PATH>` - データベースファイルパス [デフォルト: .kiro/memory/memory.db]
- `--daemon` - デーモンモード（バックグラウンド実行）※未実装
- `-v, --verbose` - 詳細ログを有効にする

**使用例**:
```bash
hail-mary memory serve --verbose
hail-mary memory serve --db-path /path/to/custom.db
```

### 3. `memory document` - ドキュメントからメモリを追加
**目的**: ファイルまたはコンテンツからメモリを作成

**オプション**:
- `<PATH>` - ファイルパスまたはコンテンツ
- `--db-path <PATH>` - データベースファイルパス
- `--type <TYPE>` - メモリタイプ [デフォルト: tech]
- `--tags <TAGS>` - カンマ区切りのタグ
- `--confidence <SCORE>` - 信頼度スコア (0.0-1.0) [デフォルト: 0.8]
- `--source <SOURCE>` - ソース情報
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory document README.md --type domain --tags "ドキュメント,readme"
```

### 4. `memory reindex` - メモリを再インデックス
**目的**: 検索インデックスを再構築し、データベースを最適化

**オプション**:
- `--db-path <PATH>` - データベースファイルパス
- `--similarity-threshold <SCORE>` - 重複の類似度閾値 [デフォルト: 0.85]
- `--no-backup` - バックアップ作成をスキップ
- `--backup-dir <PATH>` - バックアップディレクトリ
- `--dry-run` - 変更せずに何が行われるかを表示
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory reindex --similarity-threshold 0.9 --verbose
```

### 5. `memory search` - メモリを検索
**目的**: 様々な条件でメモリを検索

**オプション**:
- `<QUERY>` - 検索クエリ
- `--db-path <PATH>` - データベースファイルパス
- `--type <TYPE>` - メモリタイプでフィルタ
- `--tags <TAGS>` - タグでフィルタ（カンマ区切り）
- `--limit <N>` - 最大結果数 [デフォルト: 10]
- `--min-confidence <SCORE>` - 最小信頼度スコア
- `--output <FORMAT>` - 出力形式 (table, json, compact) [デフォルト: table]
- `--sort <FIELD>` - ソートフィールド (relevance, created, confidence, references)
- `--no-highlight` - 結果のハイライトを無効にする
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory search "非同期プログラミング" --type tech --limit 5 --sort confidence
```

### 6. `memory list` - メモリをリスト表示
**目的**: フィルタとページネーションでメモリをリスト表示

**オプション**:
- `--db-path <PATH>` - データベースファイルパス
- `--type <TYPE>` - メモリタイプでフィルタ
- `--tags <TAGS>` - タグでフィルタ
- `--limit <N>` - 最大結果数 [デフォルト: 50]
- `--offset <N>` - N件スキップ [デフォルト: 0]
- `--sort <FIELD>` - ソートフィールド (created, confidence, references, topic)
- `--order <ORDER>` - ソート順 (asc, desc) [デフォルト: desc]
- `--output <FORMAT>` - 出力形式 (table, json, compact)
- `--show-content` - 完全なコンテンツを表示
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory list --type tech --sort confidence --limit 20 --show-content
```

### 7. `memory delete` - メモリを削除
**目的**: IDでメモリを削除

**オプション**:
- `<ID>` - 削除するメモリID
- `--db-path <PATH>` - データベースファイルパス
- `--force` - 確認プロンプトをスキップ
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory delete abc123def --force
```


### 8. `memory related` - 関連メモリを検索
**目的**: 指定されたメモリまたはクエリに関連するメモリを検索

**オプション**:
- `<QUERY_OR_ID>` - メモリIDまたは検索クエリ
- `--db-path <PATH>` - データベースファイルパス
- `--limit <N>` - 最大結果数 [デフォルト: 5]
- `--min-similarity <SCORE>` - 最小類似度スコア [デフォルト: 0.7]
- `--output <FORMAT>` - 出力形式 (table, json, compact)
- `--include-self` - 結果にソースメモリを含める
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory related "非同期プログラミング" --limit 10 --min-similarity 0.8
```

### 9. `memory dedup` - 重複を削除
**目的**: 重複メモリを検索・削除

**オプション**:
- `--db-path <PATH>` - データベースファイルパス
- `--similarity-threshold <SCORE>` - 類似度閾値 [デフォルト: 0.95]
- `--strategy <STRATEGY>` - マージ戦略 (keep_newest, keep_highest_confidence, manual) [デフォルト: keep_highest_confidence]
- `--dry-run` - 削除せずに重複を表示
- `--auto-approve` - すべてのマージを自動承認
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory dedup --similarity-threshold 0.9 --strategy keep_newest --dry-run
```

### 10. `memory cluster` - メモリをクラスタリング
**目的**: 意味的類似性でメモリをグループ化

**オプション**:
- `--db-path <PATH>` - データベースファイルパス
- `--type <TYPE>` - クラスタリングするメモリタイプ
- `--algorithm <ALGORITHM>` - クラスタリングアルゴリズム (kmeans, hierarchical, dbscan, topic) [デフォルト: kmeans]
- `--num-clusters <N>` - k-meansのクラスタ数 [デフォルト: 5]
- `--min-similarity <SCORE>` - 階層クラスタリングの最小類似度 [デフォルト: 0.7]
- `--limit <N>` - クラスタリングする最大メモリ数 [デフォルト: 1000]
- `--format <FORMAT>` - 出力形式 (text, json, summary) [デフォルト: text]
- `--export <PATH>` - クラスタをJSONファイルにエクスポート
- `--stats` - クラスタ統計を表示
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory cluster --algorithm hierarchical --min-similarity 0.8 --stats --verbose
```

### 11. `memory embed-analytics` - 埋め込み分析
**目的**: 埋め込みとベクトル空間の特性を分析

**オプション**:
- `--db-path <PATH>` - データベースファイルパス
- `--type <TYPE>` - 分析するメモリタイプ
- `--analysis <TYPE>` - 分析タイプ (overview, similarity, density, outliers, vocabulary, temporal) [デフォルト: overview]
- `--limit <N>` - 分析するメモリ数 [デフォルト: 1000]
- `--export <PATH>` - 分析をJSONにエクスポート
- `--histogram` - 分布ヒストグラムを表示
- `--bins <N>` - ヒストグラムのビン数 [デフォルト: 10]
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory embed-analytics --analysis outliers --histogram --bins 15 --export analysis.json
```

### 12. `memory export` - メモリをエクスポート
**目的**: メモリをJSONまたはCSV形式でエクスポート

**オプション**:
- `--format <FORMAT>` - 出力形式 (json, csv) [デフォルト: json]
- `-o, --output <PATH>` - 出力ファイルパス（デフォルトは標準出力）
- `--db-path <PATH>` - データベースファイルパス
- `--type <TYPE>` - メモリタイプでフィルタ
- `--tags <TAGS>` - タグでフィルタ（カンマ区切り）
- `--min-confidence <SCORE>` - 最小信頼度スコア
- `--max-age-days <DAYS>` - 最大経過日数
- `--include-deleted` - 削除済みメモリを含める
- `--pretty` - JSONを整形して出力
- `--include-metadata` - メタデータ（タイムスタンプ、参照数）を含める
- `--fields <FIELDS>` - 含めるフィールド（カンマ区切り）
- `--csv-delimiter <CHAR>` - CSV区切り文字 [デフォルト: ,]
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory export --format csv --output memories.csv --include-metadata --type tech
```

### 13. `memory import` - メモリをインポート
**目的**: JSONまたはCSVファイルからメモリをインポート

**オプション**:
- `<PATH>` - 入力ファイルパス
- `--format <FORMAT>` - 入力形式 (json, csv)（拡張子から自動検出）
- `--db-path <PATH>` - データベースファイルパス
- `--dry-run` - 変更せずにインポート内容を表示
- `--update-existing` - IDが一致する既存メモリを更新
- `--skip-duplicates` - 重複コンテンツのメモリをスキップ
- `--csv-delimiter <CHAR>` - CSV区切り文字 [デフォルト: ,]
- `--default-confidence <SCORE>` - 信頼度なしメモリのデフォルト信頼度 [デフォルト: 0.8]
- `--default-type <TYPE>` - デフォルトメモリタイプ [デフォルト: tech]
- `--force` - 検証警告があってもインポートを強制
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory import data.csv --format csv --update-existing --verbose
```

### 14. `memory index` - インデックス管理
**目的**: パフォーマンス最適化のための埋め込みインデックス管理

**オプション**:
- `--db-path <PATH>` - データベースファイルパス
- `--operation <OP>` - インデックス操作 (build, update, stats, clear-cache, optimize, verify) [デフォルト: build]
- `--type <TYPE>` - インデックスするメモリタイプ
- `--batch-size <N>` - インデックス構築のバッチサイズ [デフォルト: 100]
- `--quantize` - メモリ使用量削減のため量子化を有効にする
- `--clear-cache-days <DAYS>` - N日より古いキャッシュエントリを削除
- `--force` - インデックスが存在しても強制再構築
- `-v, --verbose` - 詳細出力を有効にする

**使用例**:
```bash
hail-mary memory index --operation build --quantize --batch-size 50 --verbose
```

## 分析サブコマンド

### 15. `memory analytics` - メモリ分析
詳細な分析のためのサブコマンドを含む:

- `content` - コンテンツ分析と統計
- `health` - データベースヘルスチェック
- `memory` - メモリ使用量分析
- `performance` - パフォーマンス指標
- `summary` - 要約統計
- `trends` - トレンド分析
- `usage` - 使用パターン

**使用例**:
```bash
hail-mary memory analytics trends --period monthly --output chart.png
```

## 一括操作

### 16. `memory bulk` - 一括操作
一括操作のためのサブコマンドを含む:

- `delete` - フィルタリングによる一括削除
- `update` - メモリプロパティの一括更新
- `tag` - 一括タグ操作

**使用例**:
```bash
hail-mary memory bulk delete --type tech --confidence-below 0.5 --dry-run
```

## 共通パターン

**データベースパス**: すべてのコマンドは`--db-path`オプションをサポートし、デフォルトは`.kiro/memory/memory.db`

**詳細モード**: ほとんどのコマンドは詳細出力のため`-v, --verbose`をサポート

**ドライラン**: データを変更するコマンドは通常`--dry-run`で変更のプレビューをサポート

**出力形式**: 多くのコマンドは複数の出力形式（table, json, csv, compact）をサポート

**フィルタリング**: 検索およびリストコマンドは、タイプ、タグ、信頼度、その他の条件でのフィルタリングをサポート