# Pattern Router Framework
# パターンルーターフレームワーク

## 核心哲学

このPattern Router Frameworkは**真のReactive Pattern-Based Routing**を実装しています。パターン分類がルーティング戦略全体を決定し、**デフォルトフローは存在しません** - あらゆる入力は分類され、その分類がどのパイプラインをどのコンポーネントで実行するかを選択します。

### 核心原則：パターンこそが戦略

```
従来のアプローチ:
Input → [分岐を含む単一フロー]

このフレームワーク:
Input → Pattern Classification → Strategy Selection → Pipeline Execution
```

パターン認識は単にアクションをトリガーするだけでなく、異なるインタラクションタイプに最適化された**ルーティング戦略全体を選択**します。

### NO Linear Workflow - 会話的インタラクション

このフレームワークは**機械的な選択を排除**します。`[Y/n]`や`[1/2/3]`のような番号選択は、まさに我々が否定する「Linear Workflow」の現れです。

**排除するパターン**:
- `[Y/n]` - 二者択一の強制
- `[1] Option A [2] Option B` - 番号による選択
- `Select: a/b/c` - 制限された選択肢

**推奨するパターン**:
- オープンエンドな質問: "どのように進めましょうか？"
- 文脈的な提案: "〇〇も可能ですが、どう思いますか？"
- 自然な会話: ユーザーの自由な応答からパターン認識で意図を理解

これにより、開発は**会話の流れの中で自然に方向が決まる**、真にReactiveなインタラクションとなります。

## コアアーキテクチャ

### 4つの独立したパイプライン

フレームワークはパターン分類に基づいて、入力を4つの専用パイプラインのいずれかにルーティングします：

```
┌─────────────────────────────────────────────────────────────┐
│                    Pattern Recognition                       │
│              (03_patterns.md - Router/Classifier)           │
└────────────┬────────────┬────────────┬─────────────────────┘
             │            │            │            │
        EXPLICIT      IMPLICIT      QUERY      EMERGENCY
             │            │            │            │
             ▼            ▼            ▼            ▼
      ┌───────────┐ ┌───────────┐ ┌──────────┐ ┌──────────┐
      │ Command   │ │Suggestion │ │Diagnostic│ │ Recovery │
      │ Pipeline  │ │ Pipeline  │ │ Pipeline │ │ Pipeline │
      └───────────┘ └───────────┘ └──────────┘ └──────────┘
       重量級操作     軽量級操作    読取専用    緊急対応
       完全I/O       I/Oなし      書込なし    ゲートバイパス
```

### パイプライン特性

| パイプライン | 重量 | Hub アクセス | ゲート | 永続化 | ユースケース |
|----------|--------|------------|-------|-------------|----------|
| Command | 重量級 | 完全R/W | 全検証 | tasks.md更新 | 明示的コマンド |
| Suggestion | 軽量級 | なし | 信頼度のみ | 一時的状態 | 会話的ヒント |
| Diagnostic | 中量級 | 読取専用 | なし | 変更なし | 状態クエリ |
| Recovery | 可変 | 最小限 | 緊急オーバーライド | 最小限 | エラー処理 |

## ファイル構造と責務

### 概要

```
pattern_router/
├── index.md              # 変数プレースホルダーを含むテンプレート
├── 00_philosophy.md      # 基盤層: システム存在理由
├── 01_principles.md      # 基盤層: 普遍的運用ルール
├── 02_hub.md            # 条件付きコンポーネント: tasks.md状態管理
├── 03_patterns.md       # ルーター: パターン分類と戦略選択
├── 04_workflows.md      # パイプラインコンテナ: 複数のルーティング戦略
├── 05_gates.md          # 戦略固有: パイプライン別検証ルール
├── 06_nudges.md         # 戦略固有: パイプライン別提案テンプレート
├── 07_requirements.md   # ドキュメント構造: 要件テンプレート
├── 08_investigation.md  # ドキュメント構造: 調査テンプレート
├── 09_design.md         # ドキュメント構造: 設計テンプレート
├── 10_spec_files.md     # 動的パス: 現在の仕様ファイル参照
└── README.md            # このファイル
```

### コンポーネント分類

**基盤層** (常にロード):
- `00_philosophy.md` - システムの「なぜ」を定義
- `01_principles.md` - 全パイプライン共通の「どのように」を定義

**ルーティング＆戦略層**:
- `03_patterns.md` - **ルーター**: 入力を分類しルーティング戦略を出力
- `04_workflows.md` - **パイプラインコンテナ**: 4つの異なる実行戦略を定義

**条件付きコンポーネント** (パイプラインに基づいて起動):
- `02_hub.md` - 状態管理 (Command: R/W, Diagnostic: 読取, Suggestion: なし)
- `05_gates.md` - 検証 (パイプライン固有のルールセット)
- `06_nudges.md` - 提案生成 (パイプライン固有のテンプレート)

**構造定義** (必要に応じて参照):
- `07-09_*.md` - ドキュメントテンプレート
- `10_spec_files.md` - 動的ファイルパス提供

## 詳細ファイル説明

### 00_philosophy.md - システム哲学

**目的**: すべての設計判断を駆動する根本的信念を確立。

**核心概念**:
- **NO Linear Workflow**: 開発は非線形、どこからでも開始可能
- **Pattern Recognition over Process**: プロセスよりパターン認識を重視
- **Routing without Control**: 制約せずにルーティング
- **Single Source of Truth**: 単一の権威ある状態、複数のビュー
- **Evidence-Based Progress**: すべての決定が根拠に遡れる
- **Autonomy with Safety**: 一貫性のための検証、ガイダンスのための推奨

**参照元**: 全パイプラインが基礎的コンテキストとして参照

### 01_principles.md - 普遍的原則

**目的**: 全パイプラインに適用される運用ルールを定義。

**主要原則**:
- **Claude-Exclusive Management**: ユーザーはKiroドキュメントを直接編集しない
- **Conditional Hub Access**: パイプライン固有のhubインタラクションルール
- **Link Everything**: すべての参照に`document#section`形式を使用
- **Evidence Chain**: Requirements → Investigation → Design の追跡可能性
- **Status Discipline**: `pending | in-progress | complete`のみ使用
- **Pattern-Based Routing**: 分類が戦略を決定
- **Efficiency Through Strategy Selection**: 適切なタスクに適切なパイプライン

**参照元**: 全コンポーネントが運用ガイドラインとして参照

### 03_patterns.md - パターン認識とルーティング

**目的**: 入力を分類し、完全なルーティング戦略を出力。

**核心責務**: パターン分類 → 戦略選択 → コンポーネントリスト出力

**パターンクラス**:

| クラス | 特性 | 戦略 | コンポーネント |
|-------|-----------------|----------|------------|
| EXPLICIT | コマンド、キーワード | Command Pipeline | `[hub, gates, workflows, document, nudges]` |
| IMPLICIT | 文脈的、会話的 | Suggestion Pipeline | `[accumulate, nudges]` |
| QUERY | 状態チェック、質問 | Diagnostic Pipeline | `[hub(read), nudges]` |
| EMERGENCY | エラー、ブロッカー | Recovery Pipeline | `[nudges, recovery]` |

**分類例**:
```
Input: "/hm:requirements"
Output: {
  class: "EXPLICIT",
  confidence: 1.0,
  strategy: "command",
  components: ["hub", "gates", "workflows", "document", "nudges"]
}
→ Command Pipelineにルーティング

Input: "Users need login functionality"
Output: {
  class: "IMPLICIT",
  confidence: 0.7,
  strategy: "suggestion",
  components: ["accumulate", "nudges"]
}
→ Suggestion Pipelineにルーティング (hubアクセスなし)
```

**主要機能**: IMPLICITパターンの信頼度累積（メモリ内、閾値到達まで永続化なし）。

### 04_workflows.md - マルチ戦略ルーティングコンテナ

**目的**: 単一フローではなく、4つの異なるパイプライン実行戦略を定義。

**パイプライン定義**:

**1. Command Pipeline** (EXPLICIT):
```
Input → patterns → hub → gates → workflows(BEFORE) → document → workflows(AFTER) → nudges
```
- 完全な検証と永続化
- tasks.md更新
- 完全なBEFORE/AFTERプロトコル
- 監査証跡付き重量級操作

**2. Suggestion Pipeline** (IMPLICIT):
```
Input → patterns → [accumulate] → nudges
```
- hubインタラクションなし (tasks.md更新なし!)
- 検証ゲートなし
- 一時的な会話状態
- 直接提案生成
- **最も効率的なパス**

**3. Diagnostic Pipeline** (QUERY):
```
Input → patterns → hub(read-only) → nudges(report)
```
- 読取専用hubアクセス
- 状態変更なし
- 情報取得に特化

**4. Recovery Pipeline** (EMERGENCY):
```
Input → patterns → nudges(alert) → [recovery action]
```
- 通常検証をバイパス
- 即座の応答
- 最小限の状態チェック

**ドキュメント固有の後処理** (Command Pipelineのみ):
- Requirements完了後: 調査トピックをtasks.mdチェックリストに抽出
- Investigation完了後: カバレッジ計算、トピックチェック
- Design完了後: 実装タスクをタイムラインに抽出

### 02_hub.md - 条件付き状態管理

**目的**: tasks.md状態管理 - パイプラインが必要とする時のみアクセス。

**パイプライン別アクセスパターン**:

| パイプライン | アクセス | 操作 | 例 |
|----------|--------|------------|---------|
| Command | 完全R/W | 全CRUD操作 | 状態読取、タスク追加、ステータス更新 |
| Suggestion | **なし** | hubインタラクションなし | 一時的会話状態のみ |
| Diagnostic | 読取専用 | 状態クエリ | 読取とレポート、変更なし |
| Recovery | 最小限 | 緊急コンテキスト | オプショナルなコンテキスト読取 |

**状態追跡構造**:
- **State Tracking Table**: ドキュメント状態、カバレッジ、次アクション
- **Required Investigations Checklist**: トピック完了追跡
- **Timeline**: ドキュメントリンク付きアクション履歴

**重要な洞察**: Hubは**必須ステップではない** - 選択されたパイプラインが永続化を必要とする時のみ起動される条件付きコンポーネント。

### tasks.md更新クイックリファレンス

**更新タイミング** (`04_workflows.md`): BEFORE Protocol (pending→in-progress)、AFTER Protocol (in-progress→complete)、Document-Specific Post-Actions
**更新内容** (`02_hub.md`): State Tracking Table、Required Investigations Checklist、Timeline with links
**アクセス権限** (`01_principles.md`): Command Pipeline (完全R/W)、Suggestion Pipeline (アクセスなし)、Diagnostic Pipeline (読取専用)、Recovery Pipeline (最小限)

### 05_gates.md - 戦略固有の検証

**目的**: パイプライン戦略によって変化する検証ルール。

**パイプライン別ゲート適用**:

| パイプライン | 必須ゲート | オプショナルゲート |
|----------|---------------|----------------|
| Command | 全ドキュメント検証ゲート | - |
| Suggestion | 信頼度ゲートのみ | クールダウンゲート |
| Diagnostic | なし | - |
| Recovery | なし (緊急オーバーライド) | - |

**ドキュメント検証ゲート** (Command Pipelineのみ):
- Requirements なしの Design → ❌ BLOCK
- 100%未満のInvestigationでの Design → ❌ BLOCK
- Topicsなしの Investigation → ⚠️ WARNING

**提案ゲート** (Suggestion Pipelineのみ):
- 信頼度閾値 (0.7) → ✅ ALLOW または監視継続
- クールダウンゲート → 繰り返し提案を防止

**重要な洞察**: 軽量級パイプラインは重量級検証をスキップし、効率性を実現。

### 06_nudges.md - 戦略固有の提案

**目的**: 各パイプラインのコンテキストに合わせた提案テンプレート。

**パイプライン別テンプレートカテゴリ**:

| パイプライン | テンプレートタイプ | 例 |
|----------|---------------|----------|
| Command | 状態ベース進捗 | "Investigation 3/5 complete. Continue?" |
| Suggestion | 会話ベース | "Would you like to add this to requirements.md?" |
| Diagnostic | 状態レポート | "Current progress: Requirements ✓, Investigation 60%" |
| Recovery | 問題解決 | "⚠️ Issue detected. Immediate action: [step]" |

**信頼度ベースの表現** (Suggestion Pipeline):
- 低 (0.5-0.7): "This might be worth documenting..."
- 中 (0.7-0.85): "I recommend adding this to requirements.md"
- 高 (0.85+): "Let's add this to requirements.md! [Y/n]:"

### 07-09_*.md - ドキュメント構造定義

**目的**: 純粋なテンプレート定義、ルーティングロジックなし。

- `07_requirements.md`: PRDとBug Reportテンプレート
- `08_investigation.md`: Append-Onlyプロトコル、トピック構造、エビデンス形式
- `09_design.md`: As-Is/To-Be形式、ファイル別設計セクション

**主要原則**: これらはドキュメントが「何」であるかを定義し、「どのように」「いつ」作成するかは定義しない。

### 10_spec_files.md - 動的パス提供

**目的**: 現在の仕様ファイルパスをXMLタグ経由で提供。

**出力**:
```xml
<requirements-file>/path/to/requirements.md</requirements-file>
<design-file>/path/to/design.md</design-file>
<tasks-file>/path/to/tasks.md</tasks-file>
<investigation-file>/path/to/investigation.md</investigation-file>
<memo-file>/path/to/memo.md</memo-file>
```

## パイプライン実行例

### 例1: 明示的コマンド

```
ユーザー入力: "/hm:requirements"

Pattern Recognition (03):
→ Class: EXPLICIT
→ Confidence: 1.0
→ Strategy: command
→ Components: [hub, gates, workflows, document, nudges]

選択されたパイプライン: Command Pipeline

実行フロー:
1. Hub: tasks.md読取、pendingタスク追加
2. Gates: ブロッカーなしを検証
3. Workflows(BEFORE): タスクをin-progressに更新
4. Document: テンプレート(07)を使用してrequirements.md作成
5. Workflows(AFTER): 調査トピック抽出、tasks.md更新
6. Nudges: "Requirements complete. Start investigating [first-topic]?"

結果: tasks.md更新、requirements.md作成、状態永続化
```

### 例2: 暗黙的会話

```
ユーザー入力: "Users need to log in with email and password"

Pattern Recognition (03):
→ Class: IMPLICIT
→ Confidence: 0.7
→ Strategy: suggestion
→ Components: [accumulate, nudges]

選択されたパイプライン: Suggestion Pipeline

実行フロー:
1. Patterns: メモリ内で信頼度を累積 (hubアクセスなし)
2. Nudges: "Would you like to add this feature to requirements.md? 📝"

結果: tasks.md更新なし、一時的な提案のみ
```

### 例3: 状態クエリ

```
ユーザー入力: "What's the current progress?"

Pattern Recognition (03):
→ Class: QUERY
→ Confidence: 1.0
→ Strategy: diagnostic
→ Components: [hub(read), nudges]

選択されたパイプライン: Diagnostic Pipeline

実行フロー:
1. Hub: tasks.md読取 (読取専用)
2. Nudges: 状態レポート整形
   "Current progress: Requirements ✓, Investigation 60%, Design pending"

結果: 読取専用アクセス、状態変更なし
```

### 例4: 緊急事態

```
ユーザー入力: "Error: design validation is broken"

Pattern Recognition (03):
→ Class: EMERGENCY
→ Confidence: 1.0
→ Strategy: recovery
→ Components: [nudges, recovery]

選択されたパイプライン: Recovery Pipeline

実行フロー:
1. Nudges: "⚠️ Issue detected: Design validation failure"
2. Recovery: 通常ゲートをバイパス、即座の支援提供

結果: 緊急モード、検証バイパス
```

## 戦略選択による効率性

### なぜ複数パイプラインが重要か

**単一フローの問題**: 軽量級操作が重量級検証と永続化を強制される。

**解決策**: 軽量級操作を軽量級パイプラインにルーティング。

| 操作タイプ | 旧アプローチ | 新アプローチ | 効率化 |
|----------------|--------------|--------------|-----------------|
| 会話的提案 | 完全フロー + tasks.md更新 | nudgesに直接 | ~80%軽量化 |
| 状態クエリ | 検証 + R/W | 読取専用 | ~60%軽量化 |
| 緊急 | 検証待ち | ゲートバイパス | 即座 |
| 明示的コマンド | 完全検証 | 完全検証 | 変更なし (適切) |

### 主要効率性機能

1. **Suggestion Pipelineはファイルシステムに触れない**: 会話状態は一時的 (メモリ内のみ)
2. **Diagnostic Pipelineは読取専用**: クエリに対する状態変更なし
3. **Recovery Pipelineはゲートをバイパス**: 緊急時は即座の応答
4. **Command Pipelineは完全検証**: 重量級操作は適切な検証を受ける

## 拡張ガイド

### 新しいパターンクラスの追加

1. **パターンクラスを定義** `03_patterns.md`内:
```markdown
**NEW_CLASS Patterns**:
| User Pattern | Strategy Output |
|-------------|-----------------|
| "new-pattern" | `{class: "NEW_CLASS", strategy: "new-pipeline", components: [list]}` |
```

2. **パイプラインを作成** `04_workflows.md`内:
````markdown
### New Pipeline (NEW_CLASS class)
```
Input → patterns → [components...] → nudges
```
**Characteristics**: [定義]
````

3. **ゲートを定義** `05_gates.md`内 (必要な場合):
```markdown
## New Pipeline Gates
**Gate Name**:
- Check: [条件]
- Action: [応答]
```

4. **テンプレートを追加** `06_nudges.md`内:
```markdown
## New Pipeline Templates
**Context**: [使用タイミング]
- "Suggestion template 1"
- "Suggestion template 2"
```

### 新しいドキュメントタイプの追加

1. 構造定義で`11_newdoctype.md`を作成
2. `03_patterns.md`にパターンマッピングを追加
3. `04_workflows.md`にワークフロープロトコルを追加
4. `05_gates.md`に検証ゲートを追加 (必要な場合)
5. `06_nudges.md`に提案テンプレートを追加
6. `index.md`に新しい変数プレースホルダーを追加
7. `mod.rs`を更新して新ファイルをインクルード

## 設計原則

### 1. パターンが戦略を決定

すべての入力は分類される必要があります。分類出力は、どのコンポーネントを起動するかを含む完全なルーティング戦略を指定します。

### 2. デフォルトフローなし

単一の「メイン」フローは存在しません。各パイプラインは等しく有効で、パターンクラスに基づいて選択されます。

### 3. コンポーネント分離

コンポーネント (hub, gates, nudges) は、選択されたパイプライン戦略で指定された時のみ起動されます。

### 4. 効率性優先

軽量級操作は軽量級パイプラインを使用。重量級操作は完全検証を受けます。

### 5. 明確な境界

各パイプラインは明確な特性、コンポーネントアクセス、ユースケースを持ちます。

## スラッシュコマンド統合

スラッシュコマンドは必要に応じて特定のタグを参照します：

### `/hm:requirements` (明示的コマンド)
```yaml
参照:
- kiro-philosophy    # システム原則
- kiro-principles    # 運用ルール
- kiro-patterns      # Command Pipelineにルーティング
- kiro-hub          # 永続化のためアクセス
- kiro-workflows    # BEFORE/AFTERプロトコル実行
- kiro-gates        # 前提条件検証
- kiro-nudges       # 次アクション提案
- kiro-requirements # テンプレート使用
```

### 会話的インタラクション (暗黙的)
```yaml
参照:
- kiro-philosophy    # システム原則
- kiro-principles    # 運用ルール (注: Conditional Hub Access)
- kiro-patterns      # Suggestion Pipelineにルーティング
- kiro-nudges       # 提案生成
# 注: Hub, gates, workflowsはアクセスされない
```

## 実装メカニズム

全ファイルは`include_str!`マクロでコンパイル時に埋め込まれます：

```rust
const PATTERN_ROUTER_PATTERNS: &str = include_str!("pattern_router/03_patterns.md");
// ... 全ファイルインクルード

// 実行時:
let content = PATTERN_ROUTER_INDEX
    .replace("{patterns}", PATTERN_ROUTER_PATTERNS)
    // ... 全変数を置換
```

**メリット**:
- 実行時ファイルI/O不要
- コンパイル時の型安全性
- モジュラー編集
- 単一バイナリデプロイ

## 検証チェックリスト

- ✅ パターン認識が全入力を分類
- ✅ 明確な特性を持つ4つの独立したパイプライン
- ✅ Hubアクセスは条件付き、必須ではない
- ✅ ゲートはパイプライン固有
- ✅ Nudgesは戦略に整合
- ✅ Suggestion Pipelineはtasks.mdに触れない
- ✅ 適切なパイプライン選択による効率性
- ✅ デフォルトフローなし - すべてがパターン駆動
- ✅ コンポーネント分離の維持
- ✅ 真のreactive pattern-based routingを達成