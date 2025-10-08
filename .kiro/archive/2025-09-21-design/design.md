# Design Document - `/spec:design` Slash Command

## 概要

`/spec:design` コマンドは、要件定義書(<kiro_requirements>)から構造化された技術設計書を生成するslash commandです。専門的なアーキテクトエージェントを活用し、バックエンド、フロントエンド、システムレベルの包括的な設計を行います。

## Slash Command 仕様書 (design.md)

````markdown
---
name: design
description: "Generate technical design documents from requirements with specialized architect agents"
category: workflow
complexity: advanced
mcp-servers: []
personas: [architect, analyzer, backend-architect, frontend-architect, system-architect]
allowed-tools: Read, Write, MultiEdit, Task
argument-hint: "[--simple] [--update]"
---

# /spec:design - Technical Design Generator

## Triggers
- Requirements document needs technical design specification
- Architecture decisions require documentation
- Implementation planning from business requirements
- Design review or update of existing specifications

## Usage
```
/spec:design [--simple] [--update]
```
- `--simple`: Use simplified design template for small changes
- `--update`: Update existing design incrementally

## Key Patterns
- **Document Selection**: <kiro_requirements> exists → Detailed Template
- **Document Selection**: No requirements + complex needs → Detailed Template
- **Document Selection**: No requirements + simple needs → Simple Template
- **Complexity Detection**: Complex requirements → Activate all architect agents
- **Domain Detection**: Backend-heavy → backend-architect, UI-heavy → frontend-architect
- **Scale Detection**: System-wide → system-architect activation
- **Investigation Constraint**: Design phase → No file editing by agents
- **Update Mode**: --update flag → Incremental design refinement

## Boundaries
**Will:**
- Generate technical design documents from requirements
- Leverage specialized architect agents for comprehensive analysis
- Read existing code to understand patterns and conventions
- Support iterative design refinement
- Document all files and modifications for review
- Think in English, document in Japanese

**Will Not:**
- Estimate implementation time or effort
- Include release procedures or deployment steps
- Replace requirements gathering or investigation phases
- Make business or product decisions

## Tool Coordination
**Claude Code Tools:**
- **Read**: Load <kiro_design>, <kiro_requirements>, <kiro_investigation>
- **Write/MultiEdit**: Save design document progressively (main command only)
- **Task**: Launch specialized architect agents when complexity requires

**Agent Integration:**
- **backend-architect**: API design, database architecture, security patterns
- **frontend-architect**: UI components, accessibility, performance optimization
- **system-architect**: System boundaries, scalability, technology selection

## Document Templates

### Detailed Design Document Template
````markdown
## Meta
- **Completeness**: [0-100%]
- **Requirements**: [Brief requirements summary]
- **Architecture Scope**: [Backend/Frontend/Full-stack]

## Overview
[As-Is/To-Be overview]

## Design
[Comprehensive description of changes; which files to modify and how]

### [Target File 1: path/to/file1.ts]
[Current issues or gaps to be addressed]
[Post-modification state and additions]

```typescript
// Complete code showing the desired final state
// Include all necessary changes and implementations
```

### [Target File 2: path/to/file2.py]
[Current issues or gaps to be addressed]
[Post-modification state and additions]

```python
# Complete code showing the desired final state
# Include all necessary changes and implementations
```

### [Target File 3: path/to/new-file.js] (New File)
[Purpose and rationale for this new file]

```javascript
// Complete implementation of the new file
```

---

## Completeness Scoring Rule
- 0-30%: Overview and file identification
- 30-60%: Detailed modifications per file
- 60-80%: Complete code examples
- 80-100%: Ready for implementation
````

### Simple Design Document Template
````markdown
## Meta
- **Completeness**: [0-100%]
- **Requirements**: [One-line requirements summary]

## Overview
[Change overview]

## Design
[Key design decisions and implementation approach]

### [file1.ts]
[What needs to be changed and why]

```typescript
// Key code changes
```

### [file2.py]
[What needs to be changed and why]

```python
# Key code changes
```

## Next Steps
- [ ] Start implementation
- [ ] Create tests
````

## Behavioral Flow

1. **Initialize & Assessment**: Load existing documents and analyze current state
   - Read <kiro_design>, <kiro_requirements>, <kiro_investigation>

   **State Assessment and Next Actions:**
   - No requirements, no design → Ask: "Please provide requirements for design:"
   - Requirements exist, no design → Ask: "Create design document from requirements? [Y/n]:"
   - No requirements, design exists → Ask: "What's next? (update/investigate/questions/implement):"
   - Both exist → Ask: "What's next? (update/investigate/questions/implement/done):"

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

2. **Design Strategy Planning**: Determine architecture approach based on complexity

   **Complexity Assessment:**
   - Simple feature (single layer) → Direct design generation
   - Medium complexity (2 layers) → Activate relevant architects
   - High complexity (full-stack) → **All three architects in parallel**

   **If architect agents needed:**
   - Display: "🏗️ Launching architecture analysis with specialized agents..."
   - Execute parallel Task agents as described in Key behaviors

3. **Document Generation**: Create/update design document
   - Select template based on complexity (Detailed vs Simple)
   - If agents were used: Aggregate and synthesize recommendations
   - Resolve conflicts between domain recommendations
   - Ensure consistency across all architectural layers

4. **Review Loop**: Present design and iterate based on feedback
   - Display generated design document with completeness score
   - Ask: "Is this design acceptable? [Y/n] or provide modifications:"

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

   - If "Y" or Enter → Save to <kiro_design> and proceed
   - If "n" or modifications requested:
     * Analyze if feedback requires additional investigation
     * If investigation needed → Use Grep/Glob/Read to gather specific information
     * If simple clarification → Update with existing knowledge
     * Regenerate design incorporating new findings
     * Return to step 4 for re-review

5. **Finalization**: Save and suggest next steps
   - Write final design to <kiro_design>
   - Display: "✅ Design document saved (Completeness: XX%)"
   - Suggest next actions:
     - If needs investigation: "Run `/spec:investigate --for design` for technical details"
     - If ready: "Proceed with implementation"

Key behaviors:
- **Architect Specialization**: Each agent focuses on their domain expertise
- **Parallel Execution**: When needed, launch multiple subagents in parallel for comprehensive analysis
  - **Execute Parallel Investigation**: Launch approved Task agents
  - **[The implementation will send multiple Task tool calls in one response]**
  - Each agent operates **independently** with its own investigation context
  - Agents process **concurrently** without dependencies
- **Progressive Enhancement**: Design evolves through iterations
- **Completeness Tracking**: Display design maturity percentage

## Examples

### Example 1: Full-Stack Design with Architects
```
/spec:design

> 📋 Found requirements.md (Completeness: 70%)
> Create design document from requirements? [Y/n]:

User: Y

> 🏗️ Launching architecture analysis with specialized agents...
> • backend-architect: Analyzing API and data layer
> • frontend-architect: Analyzing UI and accessibility
> • system-architect: Analyzing scalability and integration
> [Parallel Task agents processing independently...]

> 📝 Generated design document (Completeness: 75%):
> [Design content with aggregated recommendations...]
> Is this design acceptable? [Y/n]:

User: Y

> ✅ Design document saved (Completeness: 75%)
> Next: Proceed with implementation
```

### Example 2: Simple Design without Agents
```
/spec:design --simple

> 📋 Using simple template for lightweight design...
> Please provide requirements for design:

User: "Add user profile edit functionality"

> 📝 Generated simple design (Completeness: 60%):
> [Simplified design document...]
> Is this design acceptable? [Y/n]:

User: Y

> ✅ Design saved to <kiro_design>
```

### Example 3: Design Update
```
/spec:design --update

> 📋 Found existing design.md (Completeness: 70%)
> Please provide design modifications:

User: "Add caching layer to API endpoints"

> 🏗️ Launching backend-architect for cache design...
> [Task agent analyzing caching strategies...]

> 📝 Updated design (Completeness: 85%):
> [Design with new caching section...]
> Is this design acceptable? [Y/n]:

User: Y

> ✅ Design updated successfully
```

````

## 設計の解説

### 1. **セクション構造の設計思想**
`/spec:requirements`と同様の実行フロー志向の構造：
- **Triggers → Usage**: コマンドの起動条件と使い方
- **Key Patterns**: 入力を処理戦略に変換するルール
- **Boundaries**: 実行可能な操作の明確な境界
- **Tool Coordination**: アーキテクトエージェントの詳細な連携
- **Behavioral Flow**: 段階的な実行手順
- **Examples**: 具体的な使用例

### 2. **YAMLフロントマター設計**
- `category: workflow`: Kiroワークフローの一部
- `complexity: advanced`: 複雑な並列エージェント処理
- `mcp-servers: []`: 外部MCPサーバー不要
- `personas`: 5つの専門エージェント（3つのアーキテクト含む）

### 3. **アーキテクトエージェントの活用**
3つの専門アーキテクトを並列実行：
- **backend-architect**: API、データベース、セキュリティ
- **frontend-architect**: UI、アクセシビリティ、パフォーマンス
- **system-architect**: スケーラビリティ、技術選定、長期戦略

### 4. **エージェント制約の明確化**
アーキテクトエージェントには`tools: Read, Grep, Glob, Bash`のみが許可され、Write/MultiEdit/Editは使用不可

### 5. **並列実行の強調**
- `**[The implementation will send multiple Task tool calls in one response]**`
- 各エージェントは`independently`に動作
- `concurrently`による効率的な分析

### 6. **テンプレートの使い分け**
- **Detailed Template**: フルスタック設計向け
- **Simple Template**: 小規模変更向け
- Completeness Scoreによる成熟度の可視化

### 7. **コマンド間の連携**
```
requirements → investigate → design → 実装
                    ↑          ↓
                    ←─────────←
```

### 8. **日本語ドキュメント生成**
- 思考は英語、ドキュメントは日本語
- ユーザーとのやり取りも日本語

この設計により、要件から包括的な技術設計書を生成する高度なワークフローを実現します。