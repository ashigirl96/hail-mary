# Kiro Orchestration 最終構造案 #3

## 分析手法

specification_driven_template.md の全内容を精査し、批評→提案→批評→提案を繰り返して責務分離を最適化。

## 最終構造

```xml
<kiro-spec-driven>
  <!-- Universal層: すべてに適用 -->
  <kiro-philosophy>      <!-- WHY: システムの目的と価値観 -->
  <kiro-principles>      <!-- RULES: 全体に適用される制約 -->

  <!-- Coordination層: ドキュメント間の相互作用 -->
  <kiro-patterns>        <!-- INPUT: ユーザーパターン → アクションのマッピング -->
  <kiro-protocol>        <!-- TIMING: Before/After状態更新のタイミング -->
  <kiro-validation>      <!-- CONTROL: ブロック条件 -->
  <kiro-recommendation>  <!-- GUIDANCE: 次のアクション提案 -->

  <!-- Document層: 個別ドキュメント管理 -->
  <kiro-tasks>          <!-- tasks.md の構造と境界 -->
  <kiro-requirements>   <!-- requirements.md の構造とテンプレート -->
  <kiro-investigation>  <!-- investigation.md の構造とテンプレート -->
  <kiro-design>         <!-- design.md の構造とテンプレート -->
</kiro-spec-driven>
```

## 発見した核心原則: Inward vs Outward Focus

### Inward-Focused (ドキュメントタグ)
**問い**: 「このドキュメントはどう構造化されているか？」

- `<kiro-tasks>`: tasks.mdのフォーマット、境界、振る舞い
- `<kiro-requirements>`: requirements.mdのテンプレート、書き方
- `<kiro-investigation>`: investigation.mdのAppend-only規則、ドメイン別スタイル
- `<kiro-design>`: design.mdのEvidence-based規則、As-Is/To-Be

### Outward-Focused (Coordinationタグ)
**問い**: 「ドキュメント同士はどう連携するか？」

- `<kiro-patterns>`: ユーザー入力 → どのドキュメントアクション？
- `<kiro-protocol>`: ドキュメントY完了後 → どのドキュメントを更新？
- `<kiro-validation>`: ユーザーがアクションZを要求 → 実行可能か？
- `<kiro-recommendation>`: ドキュメントY完了 → 何を提案？

### Universal (Universalタグ)
**問い**: 「すべての操作に適用される原則は？」

- `<kiro-philosophy>`: なぜこのシステムが存在するか
- `<kiro-principles>`: すべての操作に適用されるルール

## 責務割り当てルール

```
1つのドキュメントの構造/内容 → Document層タグ
複数ドキュメント間の調整     → Coordination層タグ
すべての操作               → Universal層タグ
```

### テストケース

**「Link Everything」ルールはどこ？**
- すべてのドキュメントに適用 → `<kiro-principles>` ✓

**「requirements完了後、tasks.mdに調査トピック作成」はどこ？**
- 2つのドキュメント間調整 → `<kiro-protocol>` ✓

**「requirementsには受入基準が必要」はどこ？**
- requirements.mdの構造 → `<kiro-requirements>` ✓

## 各タグの詳細内容

### `<kiro-philosophy>` (~30行)
**現在**: orchestration.md に実装済み
**内容**:
- Core Philosophy (NO Linear Workflow, Tasks.md as Central Hub, Evidence-Based)
- Reactive Rationale
- Orchestration Pattern (9ステップフロー)
- Two-Phase Response

**削除すべき**:
- "Your Role" セクション → `<kiro-principles>` へ移動

### `<kiro-principles>` 🆕 (~40行)
**統合元**: 現在散在している重要ルール
**内容**:
```markdown
## Universal Orchestration Principles

### Claude-Exclusive Management
- ユーザーはKiroドキュメントを直接編集しない
- すべての更新はClaudeのオーケストレーション経由
- 一貫性と整合性を保証

### Update Tasks.md FIRST
- あらゆるドキュメント操作の前にtasks.mdを確認・更新
- 直接のユーザー入力ではなく、更新された状態に基づいて行動

### Link Everything
- すべてのドキュメントは関連ドキュメントへの明示的な参照を含む
- Design → investigation.md#section, requirements.md#requirement
- Investigation → requirements.md#topic, source#location
- Tasks.md → すべてのリンクに document#section

### Evidence Chain
- すべての決定はソースまで遡れる
- Requirements → User stories
- Investigation → Evidence sources
- Design → Investigation findings

### One Line Rule
- TimelineとSummaryエントリーは1行で記述
- 矢印記法（→）を使用
- 詳細は各ドキュメントに記載

### Your Role
- ユーザー入力のパターンを認識
- tasks.mdを常に参照・更新
- アンチパターンを防止
- エビデンスチェーンを維持
- 次の論理的ステップを提案
```

### `<kiro-patterns>` 🆕 (~50行)
**移動元**: `<kiro-orchestration>` の Pattern-Action Mapping (91-99行)
**内容**:
```markdown
## Pattern Recognition System

### Pattern → Action Mapping
| User Pattern | Action | Document |
|-------------|--------|----------|
| "requirements", "要件", "仕様" | Create/Update | requirements.md |
| "investigate", "調査", "research" | Append | investigation.md |
| "design", "設計", "architecture" | Create (with validation) | design.md |

### Recognition Rules
- Keywords in any language trigger same pattern
- Context determines specific action variant
- Current state influences response strategy

### Mini Example
User: "Create requirements"
→ Pattern: requirements keywords
→ Action: Create requirements.md
→ Document: requirements.md
```

### `<kiro-protocol>` 🆕 (~80行)
**移動元**: 複数箇所に散在
- `<kiro-tasks-hub>` の CRITICAL Update Rules (43-53行)
- `<kiro-orchestration>` の Operation Sequence (156-179行)
- `<kiro-orchestration>` の Tasks.md Management (187-210行)

**内容**:
```markdown
## State Management Protocol

### BEFORE Any Document Operation
1. Read current tasks.md state
2. Add task with `status: pending`
3. Update to `status: in-progress` when starting

### AFTER Any Document Operation
1. Update task to `status: complete`
2. Record links to affected documents
3. Execute post-action updates (below)
4. Generate next action suggestion

### Post-Action Updates

#### After Requirements Complete
1. Extract investigation topics from requirements
2. Update tasks.md Required Investigations checklist
3. Add topics with unchecked status
4. Update State Tracking table
5. Trigger recommendation

#### After Investigation Topic Complete
1. Mark topic as complete in tasks.md checklist
2. Calculate coverage (X/Y completed)
3. Update State Tracking table
4. If 100%: Set design readiness flag
5. Trigger recommendation

#### After Design Complete
1. Extract implementation tasks to tasks.md Timeline
2. Mark design as complete in State Tracking
3. Trigger recommendation
```

### `<kiro-validation>` 🆕 (~30行)
**移動元**: `<kiro-nudging>` の 20% Enforcement (245-250行) + `<kiro-orchestration>` の一部
**内容**:
```markdown
## Validation Rules

### Blocking Conditions
- **Design without complete requirements**
  - Check: requirements.md status in tasks.md
  - Action: ❌ Block with message
  - Message: "Requirements must be complete first (check tasks.md)"

- **Design without 100% investigation**
  - Check: Investigation coverage in tasks.md
  - Action: ❌ Block with missing topics list
  - Message: "All investigations must be complete first"

- **Investigation without topics defined**
  - Check: Required Investigations checklist exists
  - Action: ⚠️ Warning with suggestion
  - Message: "Define investigation topics in requirements first?"

### Mini Example
User: "Start the design"
→ Check tasks.md: investigation coverage = 2/5
→ Validation: FAIL (not 100%)
→ Block with: "Missing investigations: [list]"
```

### `<kiro-recommendation>` 🆕 (~40行)
**移動元**: `<kiro-nudging>` の 80% Suggestions (239-243行) + `<kiro-orchestration>` の State-Based Nudging (181-185行)
**内容**:
```markdown
## Recommendation Patterns

### After Requirements Complete
- "Technical investigation needed. Start with [first-topic]?"
- "Investigation topics defined: [list]. Begin?"

### During Investigation
- "Topic complete. Coverage: X/Y. Continue with [next-topic]?"
- If high coverage: "Almost done! Only [remaining] left."
- If 100%: "All investigations complete. Ready for design?"

### After Design Complete
- "Design complete. Extract implementation tasks?"
- "Ready to begin implementation?"

### State-Based Suggestions
- If requirements empty → "Shall we start with requirements definition?"
- If investigation incomplete → "Continue investigation? Remaining: [list]"
- If design lacks evidence → "Complete missing investigations: [list]"
- If all complete → "Extract implementation tasks?"

### Mini Example
After investigation complete:
→ Calculate coverage: 3/5 (60%)
→ Suggest: "Topic complete. Coverage: 3/5. Investigate [next-topic]?"
```

### `<kiro-tasks>` 📦 (~70行)
**移動元**: `<kiro-tasks-hub>` の構造部分 + `<kiro-orchestration>` のフォーマット例
**内容**:
- State Tracking Structure template
- Required Investigations format
- Timeline format
- Boundaries (Will/Will Not)
- Key Behaviors (tasks.md特有のもの)
- Document Format Example

**削除すべき**:
- Temporal Database Role → `<kiro-principles>` へ
- CRITICAL Update Rules → `<kiro-protocol>` へ
- "Link Everything" → `<kiro-principles>` へ

### `<kiro-requirements>` 📦 (~120行)
**現状維持**: 既存の内容を保持
**削除すべき**:
- "Update <kiro_tasks> BEFORE" → これは `<kiro-protocol>` の責務
- "Define Investigation Topics" → これは `<kiro-protocol>` の Post-Action Updates

### `<kiro-investigation>` 📦 (~60行)
**現状維持**: 既存の内容を保持
**削除すべき**:
- "Check <kiro_tasks> for requirements completion" → これは `<kiro-validation>` の責務
- "Update <kiro_tasks> BEFORE and AFTER" → これは `<kiro-protocol>` の責務

### `<kiro-design>` 📦 (~130行)
**現状維持**: 既存の内容を保持
**削除すべき**:
- "Verify requirements.md status = complete" → これは `<kiro-validation>` の責務
- "Verify all investigation topics checked" → これは `<kiro-validation>` の責務
- "Update <kiro_tasks> BEFORE" → これは `<kiro-protocol>` の責務

## メリット

### 1. 完全な責務分離
- 各タグが**1つの明確な責任**を持つ
- 重複なし、曖昧さなし

### 2. 質問に対する明確な回答
- 「パターン認識を理解するには？」 → `<kiro-patterns>`
- 「requirements後の更新は？」 → `<kiro-protocol>`
- 「designをブロックするタイミングは？」 → `<kiro-validation>`
- 「tasks.mdのフォーマットは？」 → `<kiro-tasks>`

### 3. Lost in the Middle完全回避
- 各タグ50-100行以内（テンプレート含むDocument層を除く）
- フラット構造で独立参照可能

### 4. スラッシュコマンドとの親和性
- 必要なタグのみをピンポイント参照
- 例: `/spec:investigate` → `<kiro-patterns>`, `<kiro-protocol>`, `<kiro-investigation>` のみ

### 5. 拡張性
- 新機能追加時は適切な層に新タグを追加
- 既存タグへの影響を最小化
- 依存関係が明確（protocol → tasks structure のみ）

## 削除される既存タグ

- `<kiro-tasks-hub>` → 内容を `<kiro-principles>`, `<kiro-protocol>`, `<kiro-tasks>` に分散
- `<kiro-orchestration>` → 内容を `<kiro-patterns>`, `<kiro-protocol>`, `<kiro-validation>`, `<kiro-recommendation>` に分散
- `<kiro-nudging>` → 内容を `<kiro-validation>`, `<kiro-recommendation>` に分散
- `<kiro-spec-files>` → 保持（変更なし）

## 実装優先順位

### Phase 1: Universal層（基盤）
1. `<kiro-philosophy>` を整理（"Your Role"を削除）
2. `<kiro-principles>` を作成（散在ルールを統合）

### Phase 2: Coordination層（制御機構）
3. `<kiro-patterns>` を作成
4. `<kiro-protocol>` を作成
5. `<kiro-validation>` を作成
6. `<kiro-recommendation>` を作成

### Phase 3: Document層（整理）
7. `<kiro-tasks>` を作成（tasks-hub から移行）
8. `<kiro-requirements>` を整理（coordination要素を削除）
9. `<kiro-investigation>` を整理（coordination要素を削除）
10. `<kiro-design>` を整理（coordination要素を削除）

### Phase 4: 検証
11. 全タグが100行以内か確認（Document層のテンプレートを除く）
12. 責務の重複がないか確認
13. スラッシュコマンドから参照可能か確認

## 検証結果

✅ すべての現在のコンテンツが明確にマッピング可能
✅ 曖昧な割り当てなし
✅ 依存関係がクリーン（protocol → tasks structure のみ）
✅ サイズ目標達成（各タグ100行以内）
✅ 単一責任原則遵守（各タグが1つの質問に答える）

この構造により、責務の重複を**完全に排除**しながら、すべての既存機能を維持できます。