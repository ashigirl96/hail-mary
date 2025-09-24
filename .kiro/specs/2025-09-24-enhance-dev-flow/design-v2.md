# Design V2 - Reactive Kiro System Prompt

## Core Philosophy

Kiroシステムプロンプトを**リアクティブで文脈認識可能な**システムへと進化させる。slash commandの明示的な実行から、自然な会話フローでの暗黙的な理解と提案へ。

## Design Principles

### 1. Flat-First Architecture
- **主要な参照点はフラット**: 素早いアクセスと明確な参照
- **ネストは最小限**: before/after など真に階層が必要な場合のみ
- **命名規則でグループ化**: `kiro-pattern-*`, `kiro-state-*` など接頭辞で論理構造を表現

### 2. Recognition → Pattern → Flow
```
ユーザー入力
    ↓
Recognition（話題の認識）
    ↓
Pattern（文脈に応じた判断）
    ↓
Flow（処理の実行）
    ↓
Suggestion（次の提案）
```

### 3. Context-Aware Intelligence
- **状態を常に追跡**: 各ドキュメントの完成度、鮮度、依存関係
- **影響を自動検出**: 変更が他のドキュメントに与える影響を認識
- **プロアクティブな提案**: 次の最適なアクションを自然に提案

## XML Tag Structure

### Recognition Layer - 話題の認識
```xml
<kiro-recognition>
- requirements keywords: 要件, 仕様, PRD, requirement, バグレポート
- investigation keywords: 調査, 解析, 検証, investigate, リサーチ
- design keywords: 設計, アーキテクチャ, design, 実装方針
- implementation keywords: 実装, コード, 開発, build
</kiro-recognition>
```

### Pattern Layer - 文脈判断と行動決定
```xml
<!-- Requirements Patterns -->
<kiro-pattern-requirements-new>
Recognition: requirements + Context: no existing file
→ Action: Create new with template → <kiro-requirements-flow>
</kiro-pattern-requirements-new>

<kiro-pattern-requirements-update>
Recognition: requirements + Context: existing file
→ Action: Show diff and update → <kiro-requirements-flow>
</kiro-pattern-requirements-update>

<kiro-pattern-requirements-incomplete>
Recognition: requirements + Context: completeness < 50%
→ Action: "詳細を追加しますか？調査が必要な項目があります"
</kiro-pattern-requirements-incomplete>

<kiro-pattern-requirements-github>
Recognition: requirements + Context: contains github issue URL
→ Action: Auto-fetch → Convert to requirements → <kiro-requirements-flow>
</kiro-pattern-requirements-github>

<!-- Investigation Patterns -->
<kiro-pattern-investigation-prerequisite>
Recognition: design + Context: no investigation exists
→ Action: "設計の前に調査が必要です" → <kiro-investigation-flow>
</kiro-pattern-investigation-prerequisite>

<kiro-pattern-investigation-tbd>
Recognition: investigation + Context: requirements has [TBD]
→ Action: "TBDセクションを調査で解決しましょう"
</kiro-pattern-investigation-tbd>

<kiro-pattern-investigation-impact>
Recognition: investigation + Finding: "API変更|DB構造|アーキテクチャ"
→ Action: Analyze impact → "designを更新しますか？"
</kiro-pattern-investigation-impact>

<!-- Cross-Document Patterns -->
<kiro-pattern-cascading-update>
Recognition: any change + Impact detected
→ Action: List affected documents → "関連ドキュメントも更新しますか？"
</kiro-pattern-cascading-update>

<kiro-pattern-consistency-check>
Recognition: any update + Conflict detected
→ Action: "矛盾を検出: {document1} vs {document2}"
</kiro-pattern-consistency-check>
```

### Flow Layer - 処理フロー
```xml
<kiro-requirements-flow>
1. Check <kiro-state-requirements>
2. Apply <kiro-requirements-template> if new
3. Show preview with completeness score
4. Execute <kiro-suggestions-requirements> → before-update
5. Update <kiro_requirements>
6. Execute <kiro-suggestions-requirements> → after-update
7. Update <kiro-state-requirements>
8. Check <kiro-impact-detection>
</kiro-requirements-flow>

<kiro-investigation-flow>
1. Check <kiro-state-investigation>
2. Verify <kiro-prerequisites-investigation>
3. Perform investigation based on scope
4. Score confidence level
5. Execute <kiro-suggestions-investigation> → before-update
6. Update <kiro_investigation>
7. Execute <kiro-impact-detection>
8. Execute <kiro-suggestions-investigation> → after-update
</kiro-investigation-flow>

<kiro-design-flow>
1. Verify <kiro-prerequisites-design>
2. Check <kiro-state-design>
3. Load <kiro_investigation> for context
4. Generate with <kiro-design-template>
5. List affected files
6. Execute <kiro-suggestions-design> → before-update
7. Update <kiro_design>
8. Execute <kiro-suggestions-design> → after-update
</kiro-design-flow>
```

### State Layer - 状態追跡
```xml
<kiro-state-requirements>
status: none|incomplete|complete|outdated
completeness: 0-100
last-modified: ISO8601
blocks: [list of blocked items]
enables: [investigation, design]
</kiro-state-requirements>

<kiro-state-investigation>
status: none|stale|fresh|in-progress
confidence: 0-100
last-modified: ISO8601
findings: [key discoveries]
impacts: [affected documents]
</kiro-state-investigation>

<kiro-state-design>
status: none|draft|ready|implemented
completeness: 0-100
based-on-investigation: ISO8601
conflicts: [list of conflicts]
target-files: [count]
</kiro-state-design>

<kiro-state-next-action>
primary: [most logical next step based on current state]
alternatives: [other valid options]
blocked-by: [prerequisites not met]
</kiro-state-next-action>
```

### Suggestion Layer - 対話的提案
```xml
<kiro-suggestions-requirements>
  <before-update>
    - "この内容で要件を更新しますか？ [Y/n]"
    - Show: "完成度: {completeness}%"
    - If < 70%: "調査が必要: {missing sections}"
  </before-update>

  <after-update>
    - If completeness < 70%: "要件を追加しますか？"
    - If has [TBD]: "調査を開始しますか？" → <kiro-investigation-flow>
    - If completeness >= 70%: "設計を作成できます" → <kiro-design-flow>
  </after-update>
</kiro-suggestions-requirements>

<kiro-suggestions-investigation>
  <before-update>
    - "調査結果を保存しますか？ [Y/n]"
    - Show: "信頼度: {confidence}%"
    - Show: "影響範囲: {impacted documents}"
  </before-update>

  <after-update>
    - If impacts design: "設計の更新が必要です" → <kiro-design-flow>
    - If impacts requirements: "要件に反映しますか？" → <kiro-requirements-flow>
    - "追加調査が必要ですか？"
  </after-update>
</kiro-suggestions-investigation>

<kiro-suggestions-design>
  <before-update>
    - "この設計で進めますか？ [Y/n]"
    - Show: "影響ファイル: {count}個"
    - Show: "完成度: {completeness}%"
  </before-update>

  <after-update>
    - "実装を開始しますか？"
    - If requirements changed: "要件の変更を反映しますか？"
    - "設計を調整しますか？"
  </after-update>
</kiro-suggestions-design>
```

### Impact Detection Layer - 影響分析
```xml
<kiro-impact-detection>
Check after any document update:
1. Scan for keywords indicating structural changes
2. Calculate impact score (0-100)
3. List affected documents with specific sections
4. If score > 80: Auto-suggest updates
5. If score > 50: Mention potential impacts
6. Always: Update state tracking
</kiro-impact-detection>

<kiro-impact-keywords>
requirements: "スコープ変更", "新機能", "要件追加"
investigation: "API変更", "DB構造", "既存実装", "制約発見"
design: "アーキテクチャ変更", "ファイル構造", "依存関係"
</kiro-impact-keywords>
```

### Example Layer - 具体的な会話例
```xml
<kiro-example-fresh-start>
User: ユーザー認証を作りたい
Claude: [Recognition: implementation desire + no requirements]
        認証機能の要件から始めましょう。
        - 認証方式は？（パスワード、OAuth、など）
        - 対象ユーザーは？
        要件ドキュメントを作成しますか？ [Y/n]
</kiro-example-fresh-start>

<kiro-example-investigation-discovery>
User: 調査したらRedisが既に使われていた
Claude: [Pattern: investigation-impact detected]
        Redis実装の発見を記録しました（信頼度: 90%）

        この発見は設計に影響します：
        - セッション管理にRedis活用可能
        - キャッシュ戦略の見直し

        設計を更新しますか？ [Y/n]
</kiro-example-investigation-discovery>

<kiro-example-design-conflict>
User: 設計を変更したけど要件と合わない
Claude: [Pattern: consistency-check triggered]
        要件と設計の不整合を検出しました。

        選択してください：
        [1] 要件を設計に合わせる
        [2] 設計を要件に戻す
        [3] 両方を調整する
</kiro-example-design-conflict>
```

### Template Layer - ドキュメントテンプレート
```xml
<kiro-requirements-template>
# Requirements - [Feature Name]

## Metadata
- Completeness: [0-100%]
- Last Updated: [ISO8601]
- Status: [draft|review|approved]

## Overview
[Problem statement and solution approach]

## User Stories
- As a [role], I want [feature] so that [benefit]

## Acceptance Criteria
- Given [context], When [action], Then [outcome]

## Technical Requirements
[TBD - will be populated by investigation]

## Dependencies
- Investigation: [required|optional]
- External: [list of external dependencies]
</kiro-requirements-template>

<kiro-investigation-template>
# Investigation - [Topic]

## Metadata
- Confidence: [0-100%]
- Sources: [codebase|documentation|web]
- Last Updated: [ISO8601]

## Findings
### Key Discoveries
[What was found]

### Technical Constraints
[Limitations identified]

### Opportunities
[Possible improvements or optimizations]

## Impact Analysis
- Requirements: [sections affected]
- Design: [components affected]
- Implementation: [estimated effort]

## Recommendations
[Actionable next steps based on findings]
</kiro-investigation-template>

<kiro-design-template>
# Design - [Feature Name]

## Metadata
- Completeness: [0-100%]
- Based on Investigation: [timestamp]
- Requirements Version: [timestamp]

## Architecture Overview
[High-level approach]

## Component Design
### [Component Name]
- Purpose: [why this component]
- Interface: [how it connects]
- Implementation: [key details]

## File Changes
### Modified Files
- `path/to/file`: [what changes]

### New Files
- `path/to/new`: [purpose]

## Migration Strategy
[If applicable, how to migrate existing code]

## Risk Assessment
- Technical: [risks and mitigations]
- Timeline: [schedule risks]
</kiro-design-template>
```

### Rules Layer - 制約とガイドライン
```xml
<kiro-rules>
  <requirements>
    <will>
      - Always show completeness score
      - Max 70% without investigation
      - Interactive refinement until satisfaction
      - Track dependencies explicitly
    </will>
    <will-not>
      - Generate technical details without evidence
      - Skip user confirmation
      - Allow completion beyond 70% without investigation
    </will-not>
  </requirements>

  <investigation>
    <will>
      - Score confidence for all findings
      - Track sources explicitly
      - Analyze impact automatically
      - Suggest follow-up investigations
    </will>
    <will-not>
      - Make assumptions without evidence
      - Ignore cross-document impacts
      - Overwrite without preserving history
    </will-not>
  </investigation>

  <design>
    <will>
      - Require fresh investigation (< 3 days)
      - List all affected files
      - Link to investigation evidence
      - Provide implementation guidance
    </will>
    <will-not>
      - Design without investigation
      - Ignore requirement constraints
      - Skip risk assessment
    </will-not>
  </design>
</kiro-rules>
```

### Prerequisites Layer - 前提条件
```xml
<kiro-prerequisites-investigation>
- Can start fresh anytime
- If updating: check for outdated sections
- If requirements exist: align with requirements scope
</kiro-prerequisites-investigation>

<kiro-prerequisites-design>
- MUST have investigation (status: fresh)
- Investigation age < 3 days
- Requirements completeness > 50%
- If missing: "先に調査が必要です。実行しますか？"
</kiro-prerequisites-design>

<kiro-prerequisites-implementation>
- MUST have design (status: ready|implemented)
- Design completeness > 70%
- No unresolved conflicts
- If missing: "設計を先に完成させてください"
</kiro-prerequisites-implementation>
```

### Dependencies Layer - 相互依存関係
```xml
<kiro-dependencies>
# Enablement Chain
- requirements → enables → investigation, design
- investigation → enables → design, implementation
- design → enables → implementation

# Update Chain
- investigation → updates → requirements[technical], design[all]
- design → updates → requirements[feasibility], tasks
- requirements → updates → investigation[scope], design[constraints]

# Validation Chain
- investigation → validates → requirements[feasibility]
- design → validates → requirements[implementability]
- implementation → validates → design[correctness]

# Cascade Rules
- Major change in requirements → invalidate investigation, design
- Significant investigation finding → suggest design update
- Design conflict with requirements → force reconciliation
</kiro-dependencies>
```

## Reactive Behavior Patterns

### Pattern 1: Natural Flow
```
User: "認証機能を作りたい"
  ↓
Recognition: Implementation desire
  ↓
Pattern: No requirements exists
  ↓
Action: Suggest requirements first
  ↓
Flow: Create requirements → Investigation → Design → Implementation
```

### Pattern 2: Impact Cascade
```
User: "調査で新しいAPIを発見した"
  ↓
Recognition: Investigation finding
  ↓
Pattern: API change detected
  ↓
Impact: Design affected
  ↓
Suggestion: Update design to use new API
```

### Pattern 3: Conflict Resolution
```
User: "設計を更新した"
  ↓
State Check: Design conflicts with requirements
  ↓
Pattern: Consistency violation
  ↓
Action: Present reconciliation options
  ↓
Resolution: User chooses approach
```

## Key Innovations

### 1. State-Driven Intelligence
システムは常に全ドキュメントの状態を把握し、最適な次のアクションを提案

### 2. Pattern-Based Recognition
単純なキーワードマッチングではなく、文脈を理解した判断

### 3. Impact Propagation
変更の影響を自動的に分析し、関連ドキュメントの更新を提案

### 4. Natural Conversation Flow
明示的なコマンドなしに、自然な会話から意図を理解

### 5. Completeness Tracking
各ドキュメントの完成度を追跡し、次に必要なアクションを明確化