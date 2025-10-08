# Kiro Orchestration 再構築案 #2

## 背景と目的

現在の`specification_driven_template.md`の問題を解決し、Lost in the Middle問題を回避するため、XMLタグ構造を完全に再設計する。

## 現在の問題点（specification_driven_template.md）

### 1. 構造的問題
- **タグの肥大化**: 各XMLタグが100行以上になり、Lost in the Middleが発生
- **責任の混在**: 1つのタグに複数の責任が混在（例：`<kiro-tasks-hub>`に管理とオーケストレーションが混在）
- **重複した説明**: Tasks.mdの重要性が5箇所以上で繰り返される

### 2. 参照性の問題
- **深いネスト**: サブセクションが多すぎて、特定の情報を探しにくい
- **スラッシュコマンドからの参照困難**: どのタグを参照すべきか不明確

## 提案する新構造

### 完全なXMLタグ構造
```xml
<kiro-spec-driven>
  <!-- Core Philosophy -->
  <kiro-philosophy>                    <!-- 19行: 哲学と基本原則 -->

  <!-- Orchestration Mechanisms -->
  <kiro-orchestration-patterns>        <!-- Pattern → Action Mapping -->
  <kiro-orchestration-validation>      <!-- Blocking conditions & rules -->
  <kiro-orchestration-recommendation>  <!-- Next action suggestions -->

  <!-- Document Management -->
  <kiro-orchestration-tasks>          <!-- tasks.md management -->
  <kiro-requirements>                 <!-- requirements.md management -->
  <kiro-investigation>                <!-- investigation.md management -->
  <kiro-design>                       <!-- design.md management -->

  <!-- Optional/Future -->
  <kiro-examples>                     <!-- Conversation examples -->
</kiro-spec-driven>
```

## 各タグの詳細内容

### `<kiro-philosophy>` ✅ 実装済み
**現在の内容**（orchestration.md）:
- Core Philosophy（3原則）
- Reactive Rationale
- Orchestration Pattern（9ステップフロー）
- Two-Phase Response

### `<kiro-orchestration-patterns>` 🆕 新規
**移動元**: `<kiro-orchestration>` の Pattern-Action Mapping
**内容**:
```markdown
## Orchestration Patterns

### Pattern → Action Mapping
| User Pattern | Action | Post-Action |
|-------------|--------|-------------|
| "requirements", "要件" | Create/Update requirements | Suggest investigation topics |
| "investigate", "調査" | Append to investigation | Show coverage (X/Y) |
| "design", "設計" | Check 100% → Create design | Extract tasks |

### Recognition Rules
- Keywords in any language trigger same pattern
- Context determines specific action
- State influences response
```

### `<kiro-orchestration-validation>` 🆕 新規
**移動元**: 散在していたValidation Rules
**内容**:
```markdown
## Validation Rules

### Blocking Conditions
- **Design without requirements**: ❌ "Requirements must be complete first"
- **Design without 100% investigation**: ❌ "All investigations must be complete"
- **Investigation without topics**: ⚠️ "Define investigation topics in requirements first"

### State Verification
- Check tasks.md state before any operation
- Verify preconditions are met
- Ensure consistency across documents
```

### `<kiro-orchestration-recommendation>` 🆕 新規
**移動元**: 散在していたRecommendation Patterns
**内容**:
```markdown
## Recommendation Patterns

### After Requirements
- "Technical investigation needed. Start with [first-topic]?"
- "Investigation topics defined: [list]. Begin investigation?"

### During Investigation
- "Topic complete. Coverage: X/Y. Continue with [next-topic]?"
- "All investigations complete. Ready for design?"

### After Design
- "Design complete. Extract implementation tasks?"
```

### `<kiro-orchestration-tasks>` 🆕 新規
**移動元**: `<kiro-tasks-hub>` 全体
**内容**:
```markdown
## Tasks.md Management

### Boundaries
**Will:**
- Track state with `pending | in-progress | complete` ONLY
- Maintain investigation checklist and coverage
- Keep Timeline entries to ONE LINE

**Will Not:**
- Include detailed findings (→ other documents)
- Use custom status values
- Write multi-line explanations

### Key Behaviors
- **Update Protocol**: BEFORE (pending→in-progress) and AFTER (→complete)
- **One Line Rule**: Each timeline entry = single line with arrow
- **Link Everything**: Always include document#section references
- **Claude-Exclusive**: Users NEVER edit directly

### Template
```markdown
## Required Investigations
- [x] topic-1 → investigation.md#topic-1
- [ ] topic-2

## State Tracking
| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | complete | - | - |
| investigation.md | in-progress | 1/2 (50%) | Continue |

## Timeline
- [x] Requirements defined → requirements.md
- [x] Topic-1 investigated → investigation.md#topic-1
```
```

### `<kiro-requirements>` 📦 既存を整理
**現在の内容を保持**:
- Boundaries (Will/Will Not)
- Key Behaviors
- Templates (PRD/Bug Report)

### `<kiro-investigation>` 📦 既存を整理
**現在の内容を保持**:
- Boundaries (Will/Will Not)
- Key Behaviors (Append-only, Domain-specific)
- Investigation Template

### `<kiro-design>` 📦 既存を整理
**現在の内容を保持**:
- Boundaries (Will/Will Not)
- Key Behaviors (Evidence-based, Complete)
- Design Template

### `<kiro-examples>` 🤔 検討中
**移動元**: `<kiro-orchestration>` の Conversation Examples
**懸念**:
- 必要性が不明確
- 各タグ内に分散した方が良いかも
- 一旦保留

## メリット

1. **Lost in the Middle 完全回避**
   - 各タグ50-100行以内
   - フラットな構造で独立参照可能

2. **責任の明確な分離**
   - 1タグ = 1責任
   - 混在や重複を排除

3. **スラッシュコマンドとの相性**
   - 必要なタグのみピンポイント参照
   - 例：`/spec:investigate` → `<kiro-investigation>` と `<kiro-orchestration-patterns>` のみ参照

4. **拡張性**
   - 新機能追加時は新タグを追加
   - 既存タグへの影響最小限

## 実装ステップ

1. **Phase 1**: `orchestration.md` に基本構造を作成 ✅ 完了
2. **Phase 2**: Orchestration系タグを追加（patterns, validation, recommendation, tasks）
3. **Phase 3**: 既存のDocument系タグを整理・移行
4. **Phase 4**: 全体テストと調整

## 未解決の課題

1. **CRITICAL強調事項の扱い**
   - 「Claude-managed ONLY」などの重要ルール
   - 各タグの冒頭に配置？
   - または`<kiro-principles>`として独立？

2. **Conversation Examplesの位置**
   - 独立タグ vs 各タグ内に分散
   - 実例の必要性自体を再検討

3. **タグ名の長さ**
   - `kiro-orchestration-recommendation` は長い
   - でも明確性を優先すべき

---

## 責任分離の明確化（追加議論）

### 疑問: ドキュメント間連携の責任所在

#### Q1: requirements.md更新 → tasks.md更新は誰が管理？
- requirements.mdのBoundariesに書く？
- orchestrator側が知るべき？

#### Q2: "Link Everything" ルールはどこに？
- designでも使うが、designのBoundariesに書く？
- orchestrator側の共通ルール？

### 解決: 3層の責任モデル

┌──────────────────────────────────────┐
│ Universal Principles (全体に適用)    │
│ - Link Everything                    │
│ - Evidence Chain                     │
│ - Claude-Exclusive                   │
└──────────────────────────────────────┘
           ↓ applies to
┌──────────────────────────────────────┐
│ Cross-Document Coordination          │
│ - requirements → tasks.md update     │
│ - investigation → coverage update    │
│ - design → task extraction           │
└──────────────────────────────────────┘
           ↓ applies to
┌──────────────────────────────────────┐
│ Document-Specific Rules              │
│ - Boundaries (what this doc does)    │
│ - Key Behaviors (how to write it)    │
└──────────────────────────────────────┘

### 原則

1. **Universal原則**: すべてに適用される基本ルール → `<kiro-orchestration-principles>`
2. **Coordination**: ドキュメント間連携ロジック → `<kiro-orchestration-patterns>`
3. **Document-specific**: 各ドキュメント固有 → `<kiro-requirements>` など

### 改訂版XMLタグ構造 v2

```xml
<kiro-spec-driven>
  <!-- Core -->
  <kiro-philosophy>                    <!-- 哲学と基本原則 -->

  <!-- Universal Rules -->
  <kiro-orchestration-principles>      <!-- 🆕 全体に適用されるルール -->
    <!-- Link Everything, Evidence Chain, Claude-Exclusive, One Line Rule -->

  <!-- Cross-Document Coordination -->
  <kiro-orchestration-patterns>        <!-- Pattern → Action → Post-Action Updates -->
  <kiro-orchestration-validation>      <!-- Blocking conditions -->
  <kiro-orchestration-recommendation>  <!-- Next action suggestions -->

  <!-- Document Management -->
  <kiro-orchestration-tasks>          <!-- tasks.md specifics -->
  <kiro-requirements>                 <!-- requirements.md specifics -->
  <kiro-investigation>                <!-- investigation.md specifics -->
  <kiro-design>                       <!-- design.md specifics -->
</kiro-spec-driven>
```

### 具体例: `<kiro-orchestration-principles>`

```markdown
<kiro-orchestration-principles>
## Universal Orchestration Principles

### Link Everything
All documents MUST include explicit references to related documents:
- Design → investigation.md#section, requirements.md#requirement
- Investigation → requirements.md#topic, source#location
- Tasks.md → document#section for all links

### Evidence Chain
Every decision must trace back to its source:
- Requirements → User stories
- Investigation → Evidence sources
- Design → Investigation findings

### Claude-Exclusive Management
Users NEVER directly edit Kiro documents:
- All updates via Claude orchestration
- Ensures consistency and integrity
- State changes tracked in tasks.md

### One Line Rule
Timeline and summary entries must be single line:
- Use arrow notation (→) for relationships
- Keep detailed content in respective documents
- Enables quick scanning of progress
</kiro-orchestration-principles>
```

### 具体例: `<kiro-orchestration-patterns>` のPost-Action

```markdown
### Post-Action Updates

#### After Requirements Complete
1. Extract investigation topics
2. Update tasks.md Required Investigations checklist
3. Update State Tracking table
4. Recommend: "Investigation topics defined. Start with [first-topic]?"

#### After Investigation Complete
1. Mark topic complete in tasks.md checklist
2. Update coverage calculation (X/Y)
3. If 100%: Trigger design readiness check
4. Recommend next action based on coverage

#### After Design Complete
1. Extract implementation tasks to tasks.md
2. Update Timeline with design completion
3. Recommend: "Design complete. Extract implementation tasks?"
```

## メリット（改訂版）

1. **責任分離の明確化**
   - Universal: 全体ルール
   - Coordination: ドキュメント間連携
   - Document-specific: 個別の振る舞い

2. **重複の正当化**
   - "Link Everything"が複数箇所に出ても問題なし
   - Universal原則として一度定義
   - 各ドキュメントで具体的な適用方法を記述

3. **拡張性の向上**
   - 新しいUniversal原則を追加しやすい
   - ドキュメント間連携を明示的に管理
   - 個別ドキュメントの変更が独立