# Design - Slash Command Coordination with tasks.md

## Meta
- **Completeness**: 90%
- **Architecture Scope**: Kiro System Architecture (System Prompt + Slash Commands + New tasks.md)
- **Key Decision**: tasks.mdを中心とした4文書体制による責務分離

## Overview

現在のKiroシステムの課題である「slash commandにorchestrator責務が混在している問題」を、新たな`tasks.md`を導入することで解決する。これにより、各ドキュメントは単一責務となり、システム全体がシンプルかつ追跡可能になる。

**As-Is**:
- Slash commandsが実行とorchestrationを兼務
- 各ドキュメントにversion管理が必要
- 依存関係が不明確

**To-Be**:
- tasks.mdが中央タイムライン・状態管理
- 各ドキュメントは常に「現在の完全な状態」
- 依存関係が明示的で追跡可能

## Design

### Architecture: 4-Document System with Centralized State

```
┌─────────────────────────────────────────────────┐
│            System Prompt (Orchestrator)          │
│  - Natural recognition                           │
│  - Nudging based on tasks.md state              │
│  - Implicit command routing via SlashCommand     │
└────────────────┬────────────────────────────────┘
                 │
     ┌───────────┴───────────────────────┐
     │                                   │
     ▼                                   ▼
┌──────────────┐                ┌──────────────────┐
│ SlashCommand │                │  Slash Commands  │
│    Tool      │◄───────────────│  (Pure Executors)│
└──────────────┘                └────────┬─────────┘
                                         │
                ┌────────────────────────┼────────────────────────┐
                │                        │                        │
                ▼                        ▼                        ▼
        ┌──────────────┐        ┌──────────────┐        ┌──────────────┐
        │requirements.md│        │investigation.md│       │  design.md   │
        │    (WHAT)    │        │  (RESEARCH)   │       │    (HOW)     │
        └──────────────┘        └──────────────┘        └──────────────┘
                │                        │                        │
                └────────────────────────┼────────────────────────┘
                                         │
                                         ▼
                                ┌──────────────────┐
                                │    tasks.md      │
                                │ (TIMELINE+STATE) │
                                └──────────────────┘
```

### Component Responsibilities

#### 1. System Prompt (Lightweight Orchestrator)

```xml
<!-- Flat structure for better Claude recognition -->

<kiro-orchestrator-recognition>
- 自然言語から意図を理解
- requirements/investigation/design/tasksの文脈識別
- キーワード: 要件, 調査, 設計, タスク, PRD, investigate, design
</kiro-orchestrator-recognition>

<kiro-orchestrator-nudging>
- tasks.mdから現在状態を読み取り
- 依存関係に基づいて次のアクションを提案
- 95% suggestion, 5% enforcement
- "先に調査しておくと良いですよ"
- "設計への影響がありそうです"
</kiro-orchestrator-nudging>

<kiro-orchestrator-tasks-management>
- tasks.mdを自動的に管理
- 各コマンド実行後に自動でTimeline更新
- Document stateを常に最新に保つ
- ユーザーは直接編集不要、読むだけ
</kiro-orchestrator-tasks-management>

<kiro-orchestrator-routing>
User: "要件をまとめたい" → SlashCommand(/spec:requirements)
User: "設計を更新" → SlashCommand(/spec:design --update)
User: "調査結果を追加" → SlashCommand(/spec:investigate --topic)
User: "進捗を確認" → SlashCommand(/hm:tasks --status)
</kiro-orchestrator-routing>
```

#### 2. Slash Commands (Simplified Pure Executors)

##### /spec:requirements - Requirements Document Manager

````yaml
---
name: requirements
description: "Manage pure requirements documentation"
category: workflow
complexity: simple
allowed-tools: Read, Write, MultiEdit
argument-hint: "[--update]"
---

責務:
  - requirements.md の作成/更新
  - Cross-reference の作成
  - (tasks.md への記録はSystem Promptが自動実行)

削除:
  - "What's next?" 判断
  - 次ステップの提案
  - Impact analysis
  - Orchestration logic

Behavioral Flow:
  1. Read existing requirements.md if exists
  2. Gather requirements from user
  3. Apply requirements template
  4. Write to requirements.md
  5. Update tasks.md with timeline entry
  6. Return control (no suggestions)

Template:
```markdown
# Requirements - [Feature Name]

## Overview
[Pure requirements, no technical details]

## User Stories
- As a [role], I want [feature] so that [benefit]

## Cross-References
- Investigation: investigation.md#[section]
- Design: design.md#[section]
```
````

##### /spec:investigate - Investigation Accumulator

````yaml
---
name: investigate
description: "Accumulate research findings"
category: workflow
complexity: simple
allowed-tools: Read, Write, MultiEdit, Grep, Glob
argument-hint: "[--topic <name>]"
---

責務:
  - investigation.md への調査結果追記
  - Confidence level の記録
  - セクション単位での追記管理
  - (tasks.md への記録はSystem Promptが自動実行)

削除:
  - Design への影響分析
  - 次の調査提案
  - Prerequisites checking

Behavioral Flow:
  1. Read existing investigation.md
  2. If --topic provided, find existing section
  3. Perform investigation using tools
  4. Append findings to investigation.md
  5. Update tasks.md with investigation entry
  6. Return control (no impact analysis)

Template:
```markdown
## [Topic Name] [YYYY-MM-DD]
**Confidence**: [0-100]%
**Status**: [complete|in-progress]

### Findings
[Investigation results]

### Evidence
- Source: [location]
- Method: [approach]
```
````

##### /spec:design - Design Document Manager

````yaml
---
name: design
description: "Manage technical design documentation"
category: workflow
complexity: simple
allowed-tools: Read, Write, MultiEdit
argument-hint: "[--update]"
---

責務:
  - design.md の作成/更新
  - Evidence linking (investigation.md#section)
  - Cross-reference管理
  - (tasks.md への記録はSystem Promptが自動実行)

削除:
  - Prerequisites チェック
  - Implementation 提案
  - "What's next?" logic

Behavioral Flow:
  1. Read existing design.md if exists
  2. Read investigation.md for evidence
  3. Generate design based on requirements
  4. Link evidence from investigation
  5. Write to design.md
  6. Update tasks.md with design changes
  7. Return control (no next steps)

Template:
```markdown
# Design - [Feature Name]

## Architecture Decisions

### Decision: [Name]
**Evidence**:
- Source: investigation.md#[section]
- Finding: [key discovery]

## Implementation Approach
[Technical details with evidence links]
```
````

#### 3. tasks.md Management (System Prompt Automatic)

**重要**: tasks.mdは独立したslash commandではなく、System Promptによって自動管理される

```yaml
# Not a slash command - Automatically managed by System Prompt

自動管理の仕組み:
  - 各コマンド実行の副作用として更新
  - System Promptが状態を追跡
  - ユーザーの直接操作不要

更新タイミング:
  - /spec:requirements実行 → Timeline自動追加
  - /spec:investigate実行 → Timeline自動追加 + State更新
  - /spec:design実行 → Timeline自動追加 + State更新

閲覧方法:
  - "進捗を見せて" → System PromptがRead tasks.md実行
  - "状態を確認" → System PromptがRead tasks.md#state-tracking実行
```

### Document Templates

#### requirements.md - Pure Requirements
```markdown
# Requirements - [Feature Name]

## Overview
[現在の完全な要件説明]

## User Stories
- As a [role], I want [feature] so that [benefit]
- Priority: [P0/P1/P2]

## Acceptance Criteria
- [ ] Given [context], When [action], Then [outcome]
- [ ] Edge cases and error conditions

## Non-Functional Requirements
- Performance: [specific metrics]
- Security: [requirements]
- Accessibility: [standards]

## Cross-References
- Investigation basis: investigation.md#[section-name]
- Design implementation: design.md#[section-name]
```

#### investigation.md - Research Accumulation
```markdown
# Investigation

## [Topic Name] [YYYY-MM-DD]
**Confidence**: [0-100]%
**Status**: [complete|in-progress]

### Research Questions
- [What we're trying to answer]

### Findings
[調査結果をここに記載]

### Evidence
- Source: [code/documentation/external]
- Method: [how we investigated]

### Recommendations
[Based on findings]

---

## [Next Topic] [YYYY-MM-DD]
[次の調査を追記形式で追加]
```

#### design.md - Current Design State
````markdown
# Design - [Feature Name]

## Overview
[現在の完全な設計概要]

## Architecture Decisions

### Decision: [Name]
**Evidence**:
- Source: investigation.md#[section-name]
- Finding: [key discovery]
- Confidence: [percentage]

**Rationale**: [Why this approach]
**Trade-offs**: [Pros and cons]

## Implementation Approach

### [Component/Module Name]
**Based on**: investigation.md#[section]
**Constrained by**: requirements.md#[section]

#### Changes Required
```[language]
// Specific implementation details
```

### File Changes
- `path/to/file1.rs`: [what changes]
- `path/to/file2.ts`: [what changes]
````

#### tasks.md - Central Timeline & State Tracker
````markdown
# Tasks - Project Timeline & State

## State Tracking
| Document | Status | Notes |
|----------|--------|-------|
| requirements.md | complete | auth features added |
| investigation.md | in-progress | 85% confidence |
| design.md | outdated | needs sync |
| tasks.md | active | - |

## Timeline

- [x] Initial requirements defined → requirements.md created
- [x] Technology stack researched → investigation.md#tech-stack
- [x] Basic architecture designed → design.md created
- [x] Authentication requirements added → requirements.md#security
- [x] JWT vs Session investigation → investigation.md#auth-research
- [ ] Complete performance benchmarks
- [ ] Update design with auth decisions
  - source: investigation.md#auth-research
- [ ] Setup authentication middleware
  - source: design.md#auth-middleware
  - requirements: requirements.md#security-requirements
- [ ] Implement user registration
  - source: design.md#user-registration
  - requirements: requirements.md#user-stories
- [ ] Add caching layer
  - source: design.md#cache-strategy
  - investigation: investigation.md#redis-evaluation
- [ ] Review design decisions against investigation findings
````

### System Prompt Implementation

```xml
<!-- Flat XML structure for optimal Claude recognition -->

<kiro-triggers>
requirements: 要件, PRD, requirement, spec, 仕様
investigation: 調査, research, analyze, 検証, investigate
design: 設計, architecture, implementation, design
tasks: タスク, 進捗, status, timeline, priority
</kiro-triggers>

<kiro-state-awareness>
On any command:
1. Read tasks.md#state-tracking
2. Check timeline for pending tasks
3. Identify blocked items or outdated documents
</kiro-state-awareness>

<kiro-nudging-rules>
IF user_says("要件") AND !exists(requirements.md):
  → "要件から始めましょう" → SlashCommand(/spec:requirements)

IF user_says("設計") AND investigation.status != "complete":
  → "先に調査を完了させると良いですよ (Confidence: {level}%)"

IF requirements.updated > design.updated:
  → "要件が更新されています。設計も更新しますか？"

IF task.blocked:
  → "このタスクは {blocker} の完了待ちです"
</kiro-nudging-rules>

<kiro-command-mapping>
"要件をまとめて" → /spec:requirements
"調査して" → /spec:investigate
"設計を更新" → /spec:design --update
"進捗を見せて" → Read tasks.md (System Prompt実行)
"状態を確認" → Read tasks.md#state-tracking (System Prompt実行)
</kiro-command-mapping>
```

### Implementation Workflow Example

```markdown
User: "セキュリティ要件を追加したい"

System: [Recognition: requirements + security]
        [Check: tasks.md → requirements.md is complete]
        [Nudge]: "要件を更新しますね"
        [Execute]: SlashCommand(/spec:requirements)

/spec:requirements:
  1. Update requirements.md with security section
  2. Add to tasks.md:
     - Timeline: "14:00: Security requirements added"
     - Impact: "design.md needs update"
  3. Return control

System: [Check: tasks.md → new impact detected]
        [Nudge]: "セキュリティ要件が追加されました。
                 設計への影響があるので、先に調査しましょうか？"

User: "はい、調査して"

System: [Execute]: SlashCommand(/spec:investigate --topic security)
```

## Key Benefits

### 1. Separation of Concerns
- **System Prompt**: Orchestration only
- **Slash Commands**: Execution only
- **tasks.md**: State management and timeline only
- **Documents**: Content only

### 2. No Version Complexity
- 各ドキュメントは常に「現在の状態」
- 変更履歴はtasks.mdで一元管理
- Cross-referenceで関連明示

### 3. Clear Timeline
- 過去・現在・未来が一つの流れ
- チェックボックスで完了状態明確
- 必要な参照情報のみ記載

### 4. User Control
- 95% nudge, 5% enforcement
- 自動syncなし、常にユーザー選択
- シンプルな状態表示

## Implementation Files

### New Files to Create
- `.kiro/specs/*/tasks.md` - Timeline tracking document in each spec (auto-managed)

### Files to Modify
- `.claude/commands/hm/requirements.md` - Remove orchestration logic
- `.claude/commands/hm/investigate.md` - Remove impact analysis
- `.claude/commands/hm/design.md` - Remove prerequisites and next steps
- `.kiro/system-prompt.md` - Add lightweight orchestrator logic (if exists)

### Template Files
- `.kiro/templates/tasks.md` - Master template for tasks document
- `.kiro/templates/requirements.md` - Updated template without versioning
- `.kiro/templates/investigation.md` - Template for accumulative research
- `.kiro/templates/design.md` - Template with evidence linking

### System Prompt Configuration
- Add `<kiro-orchestrator-tasks-management>` for automatic management
- Add `<kiro-tasks-template>` for template reference
- Update command mapping to use Read instead of /hm:tasks

## Success Metrics
- 50% reduction in command complexity
- 100% traceability of changes
- Zero version conflicts
- Clear dependency visibility