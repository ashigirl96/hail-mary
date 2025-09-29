### 1. 構造的な課題

#### 1.1 XMLタグの肥大化
- 各XMLタグ（`<kiro-requirements>`, `<kiro-investigation>`, `<kiro-design>`）が200行以上の長大な内容を含む
- 1つのファイルに全情報を詰め込むことで、Lost in the Middle問題の解決を図ったが、結果として新たな複雑性を生んでいる
- タグ内のサブセクションが深くネストされ、参照性が低下

#### 1.2 情報の重複と散在
- **Tasks.md の重要性**が少なくとも5箇所で繰り返し説明されている
  - `<kiro-philosophy>` での概要
  - `<kiro-tasks-hub>` での詳細
  - `<kiro-orchestration>` での操作シーケンス
  - 各document管理セクションでの参照
- **CRITICAL/BEFORE/AFTER** の更新ルールが複数箇所に分散
- 同じ概念が異なる文脈で微妙に異なる説明をされている

#### 1.3 読解フローの不明瞭さ
- どのセクションから読み始めるべきか不明
- philosophy → orchestration → 各document の論理的な流れが見えにくい
- 初見のユーザーには全体像が把握しづらい

### 2. 内容的な課題

#### 2.1 抽象度の不統一
- 哲学的な説明（リアクティブパターン）と具体的な手順（テンプレート）が混在
- 概念説明と実装詳細が同じレベルで記述されている

#### 2.2 実例の不足
- Conversation Examplesはあるが、実際のドキュメント変更の流れが見えない
- Tasks.mdの状態遷移の具体例が限定的

#### 2.3 スラッシュコマンドとの分離
- システムプロンプトとスラッシュコマンドの役割分担が不明確
- どの情報をどちらに置くべきかの基準がない

## 新しい kiro.md への段階的移行提案

### フェーズ1: コア哲学の結晶化（最小限の原則）

```markdown
# Kiro: Reactive Specification Orchestration

## Core Philosophy (100行以内)
- Kiroの本質的価値提案
- リアクティブ vs 線形ワークフロー
- Tasks.md as Single Source of Truth
- パターン認識による動的反応

## Three Pillars
1. **State Management**: Tasks.md による一元管理
2. **Pattern Recognition**: ユーザー意図の理解と適応
3. **Intelligent Nudging**: 80/20ルール（提案/強制）
```

### フェーズ2: オーケストレーション・メカニズム（シンプルな実行モデル）

```markdown
## Orchestration Engine

### Pattern → Action Mapping
- 簡潔なマッピングテーブル
- 状態遷移の明確な定義
- エラーハンドリングの基本原則

### Tasks.md Protocol
- 最小限の更新ルール（BEFORE/AFTER）
- チェックリスト管理
- カバレッジ計算
```

### フェーズ3: ドキュメント・ライフサイクル（役割と関係性）

```markdown
## Document Lifecycle

### Document Roles
- requirements.md: What（要件定義）
- investigation.md: Research（調査と証拠）
- design.md: How（実装設計）
- tasks.md: Orchestrator（状態管理）

### Flow Patterns
- 典型的な進行パターン
- ブロッキング条件
- 分岐と例外処理
```

### フェーズ4: テンプレートの外部化

```markdown
## Templates (参照のみ)

実際のテンプレートは別ファイル:
- templates/requirements-prd.md
- templates/requirements-bug.md
- templates/investigation.md
- templates/design.md
```

### フェーズ5: 実例による学習

```markdown
## Complete Examples

### Example 1: Simple Feature
- 完全なフロー（requirements → investigation → design）
- Tasks.md の状態変化を含む

### Example 2: Bug Fix
- バグレポートからの流れ
- 調査と修正の実例
```
