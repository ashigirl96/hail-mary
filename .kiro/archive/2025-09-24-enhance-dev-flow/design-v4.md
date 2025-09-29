# Design v4: Hybrid Slash Command Coordination

## System Prompt Addition

```markdown
# Kiro Orchestration

## Recognition Patterns
When user mentions:
- "要件", "requirements", "PRD", "仕様" → SlashCommand(/hm:requirements)
- "調査", "investigate", "research", "検証" → SlashCommand(/hm:investigate)
- "設計", "design", "architecture", "実装" → SlashCommand(/hm:design)

## Automatic Task Management
After any /hm:* command execution:
1. Read current .kiro/specs/[active-spec]/tasks.md state
2. Append completion to Timeline with timestamp
3. Update State Tracking table with new status
4. Identify impacted documents based on changes

## Nudging Rules (80% suggestion, 20% enforcement)
- If requirements.updated > design.updated:
  → "要件が更新されました。設計も更新しますか？ [Y/n]"
- If investigation.confidence < 70% AND user requests design:
  → "調査の確信度が低いです(現在: X%)。先に調査を完了させることをお勧めします。"
- If task.blocked_by exists:
  → "このタスクは {blocked_by} の完了待ちです。"
- If all documents complete:
  → "すべてのドキュメントが完成しました。実装を開始しますか？"

## Tasks Template
When initializing new spec, create tasks.md:

# Tasks - [Spec Name]

## State Tracking
| Document | Status | Confidence | Last Updated |
|----------|--------|------------|--------------|
| requirements.md | pending | - | - |
| investigation.md | pending | - | - |
| design.md | pending | - | - |

## Timeline
- [ ] Define requirements
- [ ] Conduct investigation
- [ ] Create technical design
- [ ] Begin implementation
```

## /hm:requirements

````markdown
---
name: requirements
description: "Generate or update pure requirements documentation"
category: workflow
complexity: basic
mcp-servers: [github]
personas: []
allowed-tools: Read, Write, MultiEdit, mcp__github__get_issue
argument-hint: "[--type prd|bug] [--issue <github-url>]"
---

## Triggers
- Starting new feature development
- Updating existing requirements
- Converting GitHub issues to requirements
- Capturing user stories and acceptance criteria

## Usage
```
/hm:requirements [--type prd|bug] [--issue <github-url>]
```
- `--type`: Document type (prd or bug report, defaults to prd)
- `--issue`: GitHub issue URL to import requirements from

## Key Patterns
- **Type Detection**: --type prd → Product requirements template
- **Type Detection**: --type bug → Bug report template
- **Source Import**: --issue present → Fetch from GitHub
- **Update Mode**: Existing requirements.md → Merge new requirements

## Boundaries
**Will:**
- Generate and maintain requirements.md only
- Focus purely on WHAT, not HOW
- Import from GitHub issues when specified
- Maintain requirement traceability

**Will Not:**
- Include technical implementation details
- Suggest design approaches
- Update tasks.md (system prompt handles this)
- Recommend next steps

## Tool Coordination
**Claude Code Tools:**
- **Read**: Check for existing requirements.md
- **Write/MultiEdit**: Create or update requirements document

**MCP Integration:**
- **GitHub**: Fetch issue content when --issue provided

## Behavioral Flow
1. **Initialize**: Check for existing requirements.md
   - If exists: "現在の要件があります。更新しますか？ [Y/n]:"
   - If new: "どのような機能を開発しますか？"

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

2. **Gather Requirements**: Collect user input or import from GitHub
   - If --issue: Fetch and parse GitHub issue
   - Otherwise: Use user's description

3. **Apply Template**: Generate structured requirements
   ```markdown
   # Requirements

   ## Overview
   [Brief description]

   ## User Stories
   - As a [user type], I want [goal] so that [benefit]

   ## Acceptance Criteria
   - [ ] Criterion 1
   - [ ] Criterion 2

   ## Non-functional Requirements
   - Performance: [metrics]
   - Security: [requirements]
   ```

4. **Review**: Display generated requirements
   "要件を確認してください。問題なければ保存します。 [Y/n]:"

   **[STOP HERE AND WAIT FOR USER CONFIRMATION - DO NOT PROCEED]**

5. **Save**: Write to requirements.md and return control

Key behaviors:
- Interactive requirement gathering
- Template-based structure
- Pure requirements focus
- No orchestration logic

## Examples
```
/hm:requirements --type prd

> どのような機能を開発しますか？
[WAIT FOR USER INPUT]

User: ユーザー認証システム

> Generated requirements:
[Document content...]
> 要件を確認してください。問題なければ保存します。 [Y/n]:
[WAIT FOR USER CONFIRMATION]

User: Y

> ✅ requirements.md saved
```
````

## /hm:investigate

````markdown
---
name: investigate
description: "Accumulate technical research findings"
category: workflow
complexity: standard
mcp-servers: [sequential-thinking, context7]
personas: []
allowed-tools: Read, Write, MultiEdit, Grep, Glob, WebSearch, Task
argument-hint: "[--topic <name>] [--parallel]"
---

## Triggers
- Technical research requirements
- Architecture exploration needs
- Library evaluation requests
- Performance investigation tasks

## Usage
```
/hm:investigate [--topic <name>] [--parallel]
```
- `--topic <name>`: Continue research on existing topic
- `--parallel`: Launch parallel investigation agents

## Key Patterns
- **Topic Resume**: --topic matches section → Append to existing
- **New Topic**: No --topic → Create new investigation section
- **Parallel Mode**: --parallel → Spawn multiple Task agents
- **Confidence Tracking**: Each finding includes confidence percentage

## Boundaries
**Will:**
- Accumulate findings in investigation.md
- Track confidence levels per topic
- Preserve all previous findings (append-only)
- Use MCP servers for deep analysis

**Will Not:**
- Delete or overwrite existing research
- Make design decisions
- Update tasks.md (system prompt handles this)
- Suggest implementation approaches

## Tool Coordination
**Claude Code Tools:**
- **Read**: Load existing investigation.md
- **Grep/Glob**: Search codebase for patterns
- **WebSearch**: Research external resources
- **Task**: Spawn investigation agents when --parallel

**MCP Integration:**
- **Sequential-thinking**: Deep technical analysis
- **Context7**: Library documentation lookup

## Behavioral Flow
1. **Initialize**: Read existing investigation.md
   - If --topic provided: Find matching section
   - Otherwise: "何を調査しますか？"

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

2. **Conduct Investigation**: Research using appropriate tools
   - If --parallel: Launch multiple Task agents
   - Otherwise: Sequential investigation

3. **Document Findings**: Append to investigation.md
   ```markdown
   ## [Topic Name]
   **Date**: YYYY-MM-DD
   **Confidence**: 85%

   ### Findings
   [Research results]

   ### Evidence
   - Source: [file:line or URL]
   - Data: [metrics or benchmarks]

   ### Recommendations
   [Technical suggestions]
   ```

4. **Review**: Display findings with confidence
   "調査結果 (確信度: X%):"
   [Show findings]
   "この調査結果で問題ないですか？ [Y/n]:"

   **[STOP HERE AND WAIT FOR USER CONFIRMATION - DO NOT PROCEED]**

5. **Save**: Append to investigation.md and return control

Key behaviors:
- Append-only documentation
- Confidence scoring
- Evidence-based findings
- Multi-source investigation

## Examples
```
/hm:investigate --topic authentication

> 既存の 'authentication' 調査を継続します。
> 何を調査しますか？
[WAIT FOR USER INPUT]

User: JWT vs Session比較

> Investigating JWT vs Session authentication...
> 調査結果 (確信度: 90%):
[Findings...]
> この調査結果で問題ないですか？ [Y/n]:
[WAIT FOR USER CONFIRMATION]

User: Y

> ✅ investigation.md updated
```
````

## /hm:design

````markdown
---
name: design
description: "Generate technical design from requirements and investigation"
category: workflow
complexity: standard
mcp-servers: []
personas: [backend-architect, frontend-architect, system-architect]
allowed-tools: Read, Write, MultiEdit, Task
argument-hint: "[--simple]"
---

## Triggers
- Requirements completed and need technical design
- Architecture decisions required
- Implementation planning phase
- Design updates after investigation

## Usage
```
/hm:design [--simple]
```
- `--simple`: Use simplified design template without architect agents

## Key Patterns
- **Evidence Linking**: All decisions reference investigation.md sections
- **Requirement Tracing**: Each component maps to requirements.md
- **Architect Activation**: Complex designs trigger specialist agents
- **Decision Documentation**: Trade-offs explicitly documented

## Boundaries
**Will:**
- Generate design.md from requirements + investigation
- Link all decisions to evidence
- Document technical trade-offs
- Identify implementation tasks

**Will Not:**
- Implement code
- Estimate timelines
- Update tasks.md (system prompt handles this)
- Override requirements

## Tool Coordination
**Claude Code Tools:**
- **Read**: Load requirements.md, investigation.md, existing design.md
- **Write/MultiEdit**: Save design document
- **Task**: Launch architect agents for complex designs

**Agent Integration:**
- **backend-architect**: API design, data models
- **frontend-architect**: UI components, state management
- **system-architect**: Infrastructure, scalability

## Behavioral Flow
1. **Prerequisites Check**: Verify requirements and investigation exist
   - If missing requirements: "要件が見つかりません。先に要件を定義してください。"
   - If investigation < 70%: "調査の確信度が低いです。続けますか？ [y/N]:"

   **[STOP HERE IF PREREQUISITES MISSING]**

2. **Design Generation**: Create design based on evidence
   - If not --simple: Launch architect agents for analysis
   - Apply design template with evidence links

3. **Structure Design**: Generate comprehensive design
   ```markdown
   # Technical Design

   ## Architecture Overview
   [High-level architecture]

   ## Design Decisions
   ### Decision: [Name]
   - **Choice**: [Selected approach]
   - **Evidence**: investigation.md#[section]
   - **Requirement**: requirements.md#[section]
   - **Trade-offs**:
     - Pros: [benefits]
     - Cons: [drawbacks]

   ## Components
   ### [Component Name]
   - **Purpose**: [description]
   - **Implementation**: [approach]
   - **Dependencies**: [list]

   ## Implementation Tasks
   - [ ] Task 1 (from design.md#section)
   - [ ] Task 2 (from design.md#section)
   ```

4. **Review**: Display design with completeness metric
   "技術設計 (完成度: X%):"
   [Show design]
   "この設計で実装を開始できますか？ [Y/n]:"

   **[STOP HERE AND WAIT FOR USER CONFIRMATION - DO NOT PROCEED]**

5. **Save**: Write to design.md and return control

Key behaviors:
- Evidence-based design
- Architect agent coordination
- Requirement traceability
- Task extraction

## Examples
```
/hm:design

> 要件と調査結果から設計を生成します。
> 🏗️ アーキテクト分析を開始...
> [Architect agents working...]
>
> 技術設計 (完成度: 85%):
[Design content with evidence links...]
> この設計で実装を開始できますか？ [Y/n]:
[WAIT FOR USER CONFIRMATION]

User: Y

> ✅ design.md saved
> 実装タスクが抽出されました
```
````