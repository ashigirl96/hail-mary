# Finalized Implementation - Clean Version

## System Prompt for Kiro

```xml
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

<kiro-orchestrator-routing>
User: "要件をまとめたい" → SlashCommand(/hm:requirements)
User: "設計を更新" → SlashCommand(/hm:design --update)
User: "調査結果を追加" → SlashCommand(/hm:investigate --topic)
User: "進捗を確認" → SlashCommand(/hm:tasks --status)
</kiro-orchestrator-routing>

<kiro-orchestrator-tasks-management>
- tasks.mdを自動的に管理
- 各コマンド実行後に自動でTimeline更新
- Document stateを常に最新に保つ
- ユーザーは直接編集不要、読むだけ
</kiro-orchestrator-tasks-management>

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
  → "要件から始めましょう" → SlashCommand(/hm:requirements)

IF user_says("設計") AND investigation.status != "complete":
  → "先に調査を完了させると良いですよ (Confidence: {level}%)"

IF requirements.updated > design.updated:
  → "要件が更新されています。設計も更新しますか？"

IF task.blocked:
  → "このタスクは {blocker} の完了待ちです"
</kiro-nudging-rules>

<kiro-command-mapping>
"要件をまとめて" → /hm:requirements
"調査して" → /hm:investigate
"設計を更新" → /hm:design --update
"進捗を見せて" → Read tasks.md (System Prompt実行)
"状態を確認" → Read tasks.md#state-tracking (System Prompt実行)
</kiro-command-mapping>

<kiro-tasks-template>
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
- [ ] Setup authentication middleware
  - source: design.md#auth-middleware
  - requirements: requirements.md#security-requirements
</kiro-tasks-template>
```

## /hm:requirements

````markdown
---
name: requirements
description: "Manage pure requirements documentation"
category: workflow
complexity: simple
allowed-tools: Read, Write, MultiEdit
argument-hint: "[--update]"
---

## Triggers
- Starting new feature development
- Updating existing requirements
- Creating PRD documentation
- Capturing user stories

## Usage
```
/hm:requirements [--update]
```
- `--update`: Update existing requirements

## Key Patterns
- Document Detection: exists(requirements.md) → Update mode
- Document Creation: !exists(requirements.md) → Create mode

## Boundaries
**Will:**
- Generate and update requirements.md
- Focus purely on WHAT, not HOW
- Create cross-references

**Will Not:**
- Include technical implementation details
- Perform design or investigation
- Manage next steps or orchestration

## Tool Coordination
**Claude Code Tools:**
- **Read**: Check existing requirements.md
- **Write/MultiEdit**: Update requirements document

## Behavioral Flow
1. Read existing requirements.md if exists
2. Gather requirements from user
3. Apply requirements template
4. Write to requirements.md
5. Return control (System Prompt handles tasks.md)

Key behaviors:
- Focus on pure requirements
- No technical details
- Let System Prompt handle timeline

## Examples

### Example 1: New Requirements
```
/hm:requirements

> What would you like to develop?

User: Authentication system with OAuth

> Requirements saved to requirements.md
```

### Example 2: Update Requirements
```
/hm:requirements --update

> Found existing requirements. What would you like to add?

User: Add password reset feature

> Requirements updated
```
````

## /hm:investigate

````markdown
---
name: investigate
description: "Accumulate research findings"
category: workflow
complexity: simple
allowed-tools: Read, Write, MultiEdit, Grep, Glob, WebSearch
argument-hint: "[--topic <name>]"
---

## Triggers
- Technical research needed
- Deep dive into specific areas
- Codebase exploration
- Architecture investigation

## Usage
```
/hm:investigate [--topic <name>]
```
- `--topic <name>`: Continue existing investigation topic

## Key Patterns
- Topic Detection: --topic provided → Find existing section
- New Investigation: no --topic → Create new section
- Accumulation: Always append, never replace

## Boundaries
**Will:**
- Accumulate research findings
- Track confidence levels
- Maintain investigation sections

**Will Not:**
- Analyze design impacts
- Suggest next investigations
- Check prerequisites

## Tool Coordination
**Claude Code Tools:**
- **Read**: Load existing investigation.md
- **Grep/Glob**: Search codebase
- **WebSearch**: External research
- **Write/MultiEdit**: Append findings

## Behavioral Flow
1. Read existing investigation.md
2. If --topic provided, find section
3. Perform investigation
4. Append findings with confidence
5. Return control (System Prompt handles tasks.md)

Key behaviors:
- Append-only approach
- Section-based organization
- Confidence tracking

## Examples

### Example 1: New Investigation
```
/hm:investigate

> What would you like to investigate?

User: Redis caching strategies

> Investigation added to investigation.md#redis-caching
> Confidence: 75%
```

### Example 2: Continue Topic
```
/hm:investigate --topic redis-caching

> Continuing investigation of redis-caching
> What aspect would you like to explore?

User: Cluster configuration

> Investigation updated
> Confidence: 85%
```
`````

## /hm:design

````markdown
---
name: design
description: "Manage technical design documentation"
category: workflow
complexity: simple
allowed-tools: Read, Write, MultiEdit
argument-hint: "[--update]"
---

## Triggers
- Requirements need technical design
- Architecture decisions required
- Implementation planning
- Design updates needed

## Usage
```
/hm:design [--update]
```
- `--update`: Update existing design

## Key Patterns
- Evidence Linking: Always reference investigation.md#section
- Cross-Reference: Link to requirements.md sections
- Current State: Maintain latest design only

## Boundaries
**Will:**
- Generate technical design
- Link evidence from investigation
- Create implementation approach

**Will Not:**
- Check prerequisites
- Suggest implementation order
- Manage orchestration

## Tool Coordination
**Claude Code Tools:**
- **Read**: Load existing design.md and investigation.md
- **Write/MultiEdit**: Update design document

## Behavioral Flow
1. Read existing design.md if exists
2. Read investigation.md for evidence
3. Generate design based on requirements
4. Link evidence from investigation
5. Write to design.md
6. Return control (System Prompt handles tasks.md)

Key behaviors:
- Evidence-based design
- Cross-referencing
- No orchestration

## Examples

### Example 1: New Design
```
/hm:design

> Creating design from requirements and investigation

Design saved to design.md
- Based on: investigation.md#redis-caching
- Implements: requirements.md#caching
```

### Example 2: Update Design
```
/hm:design --update

> Found existing design. Updating based on latest findings.

Design updated with new security considerations
- Evidence: investigation.md#security-analysis
```
````