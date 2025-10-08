# Design Document - `/spec:requirements` Slash Command

## 概要

`/spec:requirements` コマンドは、ユーザーの要望やGitHub issueから構造化された要件ドキュメントを生成するslash commandです。このドキュメントは後続の調査、設計、タスク分解フェーズの基盤となります。

## Slash Command 仕様書 (requirements.md)

````markdown
---
name: requirements
description: "Generate structured requirement documents from user needs or GitHub issues"
category: workflow
complexity: standard
mcp-servers: [github]
personas: [analyst, architect]
allowed-tools: Read, Write, MultiEdit, mcp__github__get_issue
argument-hint: "[--type prd|bug] [--issue <github-url>]"
---

# /spec:requirements - Requirements Document Generator

## Triggers
- Starting new feature development requiring structured documentation
- Bug reporting that needs formal issue documentation
- GitHub issue needs to be converted to actionable requirements
- Project planning phase initiation

## Usage
```
/spec:requirements [--type prd|bug] [--issue <github-url>]
```
- `--type`: Document type (prd for new features, bug for issue tracking)
- `--issue`: Optional GitHub issue URL for auto-population

## Key Patterns
- **Type Detection**: --type prd → PRD template activation
- **Type Detection**: --type bug → Bug template activation
- **Source Detection**: --issue present → GitHub MCP activation
- **Source Detection**: no --issue → Interactive mode activation
- **Complexity Assessment**: PRD → high complexity → multiple iterations
- **Persona Activation**: Requirements analysis → analyst + architect

## Boundaries
**Will:**
- Generate and update <kiro_requirements_path> only
- Fetch and parse GitHub issues when provided
- Calculate and display completeness scores
- Achieve maximum 70% completeness through interactive refinement with user
- Iterate based on user feedback until satisfaction or reaching 70% completion
- Structure content differently for PRD vs Bug types
- Include references to source documents and materials used

**Will Not:**
- Exceed 70% completeness (requires `/spec:investigate` for technical discovery)
- Attempt to fill technical sections without codebase investigation
- Perform investigation, design, or task decomposition
- Modify files other than <kiro_requirements_path>
- Auto-generate content without user input
- Proceed without explicit user confirmation at [STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED] points
- Make assumptions about technical implementation details
- End iterative refinement without explicit user approval

## Tool Coordination
**Claude Code Tools:**
- **Read**: Read <kiro_requirements_path> to understand existing content and context
- **Write/MultiEdit**: Create or update <kiro_requirements_path>

**MCP Integration:**
- **GitHub Server**: Use `mcp__github__get_issue` to fetch issue details
  - Extract title, description, labels, comments
  - Parse into appropriate requirement sections
  - Maintain issue link for traceability

## Document Templates

### PRD Template
```markdown
# Requirements - [Feature Name]

## Metadata
- **Completeness**: [0-100%]
- **Source**: [user-input|github-issue: URL]
- **References**:
  - [List of consulted documents]
  - [Will be populated by /spec:investigate]

## 1. Overview
- Problem statement
- Proposed solution

## 2. Purpose
- [Why this feature is needed]

## 3. User Stories
- As a [user], I want [feature] so that [benefit]
- Priority: [P0/P1/P2]

## 4. Acceptance Criteria
- Given [context], When [action], Then [outcome]
- Edge cases and error conditions

## 5. Technical Requirements
[TBD - populated by /spec:investigate]
- Architecture decisions
- Dependencies
- Integration points
- Impact analysis

---

## Completeness Scoring Rule
- **0-70%**: User requirements and business context
  - Problem definition, user stories, acceptance criteria
  - Maximum achievable through user interaction alone
- **70-100%**: Technical completion
  - Dependencies, constraints, impact analysis
  - Requires codebase investigation
```

### Bug Template
```markdown
# Bug Report - [Title]

## Metadata
- **Severity**: [Critical/High/Medium/Low]
- **Completeness**: [0-100%]
- **Source**: [user-input|github-issue: URL]
- **References**:
  - [Error logs/screenshots]
  - [Will be populated by /spec:investigate]

## 1. Problem
- **What's broken**: [user description]
- **How to reproduce**:
  1. [Step by step]
- **Error/Logs**: [if any]

## 2. Expected
- **Should do**: [expected behavior]
- **Success criteria**: [how to verify fix]

## 3. Technical Details
[TBD - populated by /spec:investigate]
- Root cause
- Affected files
- Fix approach
- Impact analysis

---

## Completeness Scoring Rule
- **0-70%**: Problem documentation
  - Symptoms, reproduction steps, expected behavior
  - Maximum achievable through user reporting
- **70-100%**: Root cause identification
  - Root cause, affected components, technical context
  - Requires codebase investigation
```

## Behavioral Flow

1. **Initialize**: Parse command arguments and determine document type
   - Validate type parameter (prd or bug)
   - If --issue provided, fetch GitHub issue content using MCP
   - Read <kiro_requirements_path> to understand existing content and context

2. **Interactive Requirements Gathering**: Present type-specific questions
   - **For PRD**: "What new feature or capability would you like to develop? Please describe the problem you're solving, target users, and desired outcomes."
   - **For Bug**: "Please describe the current problematic behavior and what the expected behavior should be. Include steps to reproduce if possible."

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

3. **Document Generation**: Create initial <kiro_requirements_path> draft
   - Parse user input and structure into appropriate sections
   - Calculate completeness score (weighted by section importance)
   - Display generated document with completeness percentage
   - Ask: "Here's the initial requirements document (Completeness: XX%). Is this accurate? [Y/n] or provide clarification:"

   **[STOP HERE AND WAIT FOR USER CONFIRMATION - DO NOT PROCEED]**

4. **Iterative Refinement**: Process user feedback and save
   - If "Y" or Enter → Write to <kiro_requirements_path> and proceed to step 5
   - If "n" or clarification provided → Update document based on feedback
   - Recalculate completeness score
   - Return to step 3 for re-confirmation

5. **Summary**: Display final results
   - Show final completeness score
   - Display document summary
   - Confirm successful save with path

Key behaviors:
- **Completeness Tracking**: Display document completeness percentage after each generation
- **GitHub Integration**: Auto-extract and structure issue content when URL provided
- **Interactive Refinement**: Multiple feedback rounds until user satisfaction
- **Type-Specific Templates**: Different structures for PRD vs Bug documentation
- **Progressive Enhancement**: Document grows through conversation, not all at once

## Examples

### Example 1: PRD Creation
```
/spec:requirements --type prd

> 📋 Starting PRD creation...
> What new feature or capability would you like to develop?
> Please describe:
> - The problem you're solving
> - Target users
> - Desired outcomes
>
> [STOP AND WAIT FOR USER INPUT]

User: "We need a dashboard for monitoring system health..."

> 📝 Generated requirements document (Completeness: 75%):
> [Document content...]
> Is this accurate? [Y/n] or provide clarification:

User: Y

> ✅ Requirements saved to <kiro_requirements_path>
```

### Example 2: Bug Documentation from GitHub Issue
```
/spec:requirements --type bug --issue https://github.com/org/repo/issues/123

> 🔍 Fetching GitHub issue #123...
> 📋 Analyzing issue content...
>
> Based on the issue, here's the bug documentation (Completeness: 85%):
> [Generated bug report...]
>
> Would you like to add any additional context? [Y/n]:

User: n

> ✅ Bug requirements saved to <kiro_requirements_path>
```

### Example 3: Iterative Refinement
```
/spec:requirements --type prd

[Initial gathering...]

> 📝 Generated requirements (Completeness: 60%):
> [Document content...]
> Is this accurate? [Y/n] or provide clarification:

User: n, we also need to consider mobile users and API integration

> 📝 Updated requirements (Completeness: 80%):
> [Updated document with mobile and API sections...]
> Is this accurate? [Y/n]:

User: Y

> ✅ Requirements finalized (Completeness: 80%)
```

````

## 設計の解説

### 1. **セクション構造の設計思想**
実際のプログラム実行フローに沿った論理的な構造を採用：
- **Triggers → Usage**: コマンドの起動条件と使い方
- **Key Patterns**: 入力を処理方法に変換する戦略決定
- **Boundaries**: 実行可能な操作の境界を定義
- **Tool Coordination**: 必要なツールとMCPサービスの準備
- **Behavioral Flow**: すべての情報が揃った上での実行手順
- **Examples → Boundaries**: 具体例と制約の明示

### 2. **YAMLフロントマター設計**
- `category: workflow`: Kiroワークフローの一部として位置づけ
- `complexity: standard`: 中程度の複雑さ（対話型だが単純な処理）
- `mcp-servers: [github]`: GitHub issue統合のためGitHub MCPを使用
- `personas: [analyst, architect]`: 要件分析と構造化に適したペルソナ

### 3. **Key Patternsの役割**
変換ルール（IF-THEN）として機能：
- Type Detection: コマンド引数からテンプレート選択
- Source Detection: データソースに応じた処理モード決定
- Complexity Assessment: 処理の複雑度判定
- Persona Activation: 適切なペルソナの活性化

### 4. **インタラクティブフロー**
- 明示的な`[STOP]`マーカーで対話ポイントを制御
- ユーザー入力を待ってから処理を継続
- 反復的な改善サイクルをサポート

### 5. **完成度スコアリング**
PRDとBugで異なる重み付け：
- PRD: 各セクションの存在と内容の充実度を評価
- Bug: 再現手順と期待動作の明確さを重視

### 6. **GitHub統合**
- `--issue`フラグでGitHub issueを自動取得
- MCP serverを使用してissue内容を構造化
- issueリンクを保持してトレーサビリティ確保

### 7. **境界の明確化**
- <kiro_requirements_path>のみを更新（単一責任）
- 調査や設計には踏み込まない
- ユーザー確認なしに進まない

## 追加設計: Investigation統合

### Investigation Command (`/spec:investigate`)

**Purpose**: Deep technical analysis of codebase to populate TBD sections

**Process**:
1. **Marker Detection**: Find all [TBD] markers in requirements
2. **Code Analysis**: Scan relevant files and dependencies
3. **Impact Assessment**: Identify affected components
4. **Technical Discovery**: Find implementation constraints
5. **Auto-Population**: Fill technical sections with discoveries
6. **Completeness Update**: Adjust metadata scores

**Integration Flow**:
```yaml
/spec:requirements:
  creates: user_requirements
  marks: [TBD] sections
  achievable_completeness: 0-70%

/spec:investigate:
  finds: [TBD] markers
  populates: technical_details
  updates: completeness_score
  final_completeness: 70-100%
```

### Two-Stage Workflow
```bash
# Stage 1: User-driven requirements
/spec:requirements --type prd
# → Interactive refinement with user
# → Captures business context and acceptance criteria
# → Achieves completeness: 0-70%

# Stage 2: Technical discovery
/spec:investigate --enhance requirements.md
# → Analyzes codebase for technical context
# → Populates [TBD] sections automatically
# → Completes document: 70-100%
```

### Adaptive Template Selection
```yaml
template_selection:
  simple_feature: minimal_prd.md      # 5 sections
  complex_feature: full_prd.md        # 10+ sections
  critical_bug: detailed_bug.md       # Full analysis
  minor_bug: quick_bug.md             # 3 sections only

context_rules:
  if (greenfield_project):
    skip: ["Dependencies", "Migration"]

  if (bug_in_production):
    require: ["Severity", "Workaround"]

  if (refactoring):
    require: ["Current State", "Target State"]
```

この設計により、人間が書ける部分と機械が発見する部分を明確に分離し、段階的に文書を充実させる実用的なワークフローを実現します。