# System Prompt Templates

## 概要

このディレクトリには、Hail-Maryが使用するClaude Codeのシステムプロンプトテンプレートが含まれています。これらのテンプレートは、Kiro仕様駆動開発のオーケストレーション機能を実現するための中核コンポーネントです。

## ファイル構成

### specification_driven_template.md
**役割**: Tasks.mdベースのオーケストレーションシステム

Kiro仕様駆動開発の中核となるテンプレートで、以下の機能を提供します：

#### 1. リアクティブ・オーケストレーション
- スラッシュコマンド（`/spec:requirements`, `/spec:investigate`, `/spec:design`）をトリガーとして起動
- Tasks.mdを**Single Source of Truth**として、すべての状態管理と意思決定を実行
- **線形ワークフローの強制ではなく**、パターン認識による動的な反応システム
- ユーザーの自由を尊重しながら、より良い方向へ**賢くナッジング**

#### 2. XMLタグによる構造化
Lost in the Middle問題を解決するため、フラットなXMLタグ構造を採用：

```xml
<kiro-spec-driven>
  <!-- 共通オーケストレーション要素 -->
  <kiro-philosophy>基本思想とリアクティブパターン</kiro-philosophy>
  <kiro-tasks-hub>Tasks.md中央制御（CRITICAL）</kiro-tasks-hub>
  <kiro-orchestration>操作シーケンスとフロー</kiro-orchestration>
  <kiro-nudging>ナッジング動作（80%賢い提案/20%最小限の強制）</kiro-nudging>

  <!-- アクション固有要素 -->
  <kiro-requirements>要件管理とテンプレート</kiro-requirements>
  <kiro-investigation>調査管理とドメイン別スタイル</kiro-investigation>
  <kiro-design>設計管理とエビデンスチェーン</kiro-design>

  <kiro-spec-files>仕様ファイルパス</kiro-spec-files>
</kiro-spec-driven>
```

スラッシュコマンドは必要なタグのみをピンポイント参照することで、効率的な動作を実現します。

#### 3. Tasks.md中央ハブによる状態管理

```markdown
## Required Investigations
### Authentication
- [x] jwt-implementation → investigation.md#jwt-implementation
- [x] database-schema → investigation.md#database-schema
- [ ] session-management
- [ ] security-best-practices

## State Tracking
| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | complete | - | - |
| investigation.md | in-progress | 2/4 (50%) | Complete remaining topics |
| design.md | pending | - | Awaiting 100% coverage |

## Timeline
- [x] Requirements defined → requirements.md
- [x] JWT implementation investigated → investigation.md#jwt-implementation
- [x] Database schema investigated → investigation.md#database-schema
- [ ] Design authentication flow (blocked: 2/4 investigations incomplete)
```

**重要な変更点**：
- **Confidence廃止**: 曖昧な「信頼度」から明確な「チェックリスト」へ
- **Coverage導入**: 完了した調査項目数/全調査項目数で進捗を表示
- **Topic統一**: tasks.mdのトピック名とinvestigation.mdのセクション名を完全一致

すべてのオーケストレーション判断は、このTasks.mdのチェックリストとカバレッジに基づいて行われます。

#### 4. インタラクティブな確認フロー

各ドキュメント作成後、ユーザーとの対話的な確認を実施：
- Requirements: "Save to requirements.md? (Y to save / or provide feedback for improvements)"
- Investigation: "Append to investigation.md? (Y to save / or ask questions to clarify)"
- Design: "Save to design.md? (Y to save / or suggest improvements)"

#### 5. エビデンスチェーンの強制

すべての設計決定に対して、以下の要素を含むことを強制：
```markdown
### [file-path]
[Purpose (requirements.md#section), current state,
 investigation findings (investigation.md#section1, investigation.md#section2, ...),
 key evidence, and solution approach with technical details]
```

これにより、requirements → investigation → design のトレーサビリティが保証されます。

**調査管理の改善**：
- requirementsで必要な調査を定義 → tasks.mdにチェックリスト作成
- 各調査完了時にチェック → investigation.mdに詳細記録
- 全チェック完了(100%)でdesign開始可能

#### 6. ナッジングと最小限のブロッキング

**80% インテリジェントなナッジング**：
- Design要求でRequirements未定義 → "requirementsから始めませんか？design作成がスムーズになります"
- Requirements更新後 → "既存のdesign.mdの更新も必要かもしれません。確認しますか？"
- Investigation未完了 → "これらのトピック [list] を調査すると品質が向上します。今調査しますか？"

**20% 最小限のハードブロック**（本当に危険な場合のみ）：
- 矛盾する変更で既存システムを壊す可能性
- セキュリティリスクやデータ損失の危険性
- 明らかに不完全な状態での本番デプロイ

### base_template.md
**役割**: 基本的なKiro紹介テンプレート

Kiroシステムの基本的な説明を提供します（仕様選択なしの場合に使用）。

### steering_template.md
**役割**: ステアリング情報テンプレート

プロジェクト固有のステアリング情報（product, tech, structure等）を埋め込むためのテンプレートです。

## 統合フロー

1. **ユーザーがスラッシュコマンドを実行**
   ```
   /spec:investigate "API構造を調査"
   ```

2. **スラッシュコマンドがシステムプロンプトのXMLタグを参照**
   ```markdown
   Refer to system prompt sections:
   - <kiro-tasks-hub> for critical update timing and checklist management
   - <kiro-orchestration> for operation sequence
   - <kiro-investigation> for investigation behaviors
   - <kiro-nudging> for post-investigation prompts
   ```

3. **Tasks.mdの状態を確認**
   - Requirements complete? → Yes/No
   - Investigation coverage? → X/Y topics complete

4. **アクション実行とTasks.md更新**
   - BEFORE: Status = pending → in-progress
   - DURING: 調査実行、investigation.mdに追記
   - AFTER: Topic checked ✓, coverage = 3/5 (60%)

5. **ナッジング**
   ```
   Append to investigation.md? (Y to save / or ask questions to clarify)
   ```

## 設計原則

### 1. Tasks.md as Single Source of Truth
すべての状態管理と意思決定はTasks.mdを経由します。ファイルの存在チェックではなく、Tasks.mdのステータスで判断します。

### 2. リアクティブパターンベース哲学
- **強制ではなく提案**: ユーザーの意図を理解し、最適なパスを提案
- **文脈認識**: 現在の状態から最適な次のステップを動的に判断
- **自由と品質の両立**: ユーザーの選択を尊重しつつ、品質向上へ導く

### 3. フラットなXMLタグ構造
ネストを避け、各タグを独立して参照可能にすることで、Lost in the Middle問題を回避します。

### 4. チェックリストベースの調査管理
- 曖昧な「Confidence %」から明確な「チェックリスト」へ移行
- トピック名の統一によりtasks.md ↔ investigation.mdの関係を自明に
- GitHubのMarkdownでチェックボックスが直接操作可能

### 5. インタラクティブな確認
保存前に必ずユーザー確認を行い、フィードバックループを提供します。

### 6. エビデンスベースの設計
すべての設計決定に対して、requirements と investigation への明示的なリンクを要求します。

## 開発時の注意点

- テンプレートは実行時にRustコードでプレースホルダー（`{spec_name}`, `{spec_path}`）が置換されます
- コンパイル時に`include_str!`でバイナリに埋め込まれます
- XMLタグの追加・変更時は、対応するスラッシュコマンド（`.claude/commands/hm/*.md`）も更新が必要です