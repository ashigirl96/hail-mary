# Prompt Engineering Guidelines

## Command Priority Override with Quality Control
**When**: Complex conversations prevent slash commands from executing their behavioral flow
- Use `<command_execution priority="immediate">` wrapper at command start
- Include explicit **OVERRIDE** directive to supersede all active contexts
- Specify **PROTOCOL** enforcement for exact behavioral flow execution
- Add **CONTEXT** instruction to preserve conversation history while following workflow
- Add **QUALITY** directive to maintain specification compliance despite priority override

```markdown
# ✅ Good - Priority override with 4 directives
<command_execution priority="immediate">
**OVERRIDE**: This command supersedes all active tasks and contexts.
**PROTOCOL**: Execute behavioral flow exactly as specified below.
**CONTEXT**: Use conversation history for learning extraction while following this workflow.
**QUALITY**: Maintain full specification compliance despite priority execution.
</command_execution>

# ❌ Bad - Standard slash command header in complex contexts
## /my-command - Standard Implementation
[Behavioral Flow starts here without priority override]
```

## File References in Slash Commands
**When**: Referencing files in Claude Code commands
- Use `@` prefix to auto-load file contents into command context
- Avoid redundant Read tool calls when `@` is used
- `@file.txt` automatically provides file content to Claude

```markdown
# ✅ Good - @ symbol auto-loads content
Load Types from Config: Analyze @.kiro/config.toml for type definitions

# ❌ Bad - Double reading with @ and Read tool
Load Types from Config: Read @.kiro/config.toml using Read tool
```

## Interactive User Input Patterns
**When**: Requiring confirmed user input before proceeding
- Use explicit stop markers: `[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]`
- Define clear response handling after stop point
- Specify actions for each valid response (Y/n, 1-4, etc.)
- Include invalid input handling instructions

```markdown
# ✅ Good - Explicit stop with clear handling
> Append to file? [Y/n]: 

**[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

After user responds:
- Response = "Y" or "y" or Enter → Proceed with action
- Response = "n" or "N" → Skip and continue
- Any other response → Ask for clarification

# ❌ Bad - Ambiguous waiting instruction
> Append to file? [Y/n]: 
→ Wait for user confirmation → Proceed
```

## Bracket Notation in Prompts
**When**: Using brackets in prompt engineering
- `[XXX]` for meta-instructions and command markers
- `<XXX>` for data boundaries and structured I/O
- `{XXX}` for variable expansion and templates
- Square brackets signal "system-level" instructions to AI

```markdown
# ✅ Good - Clear meta-instruction
[STOP HERE AND WAIT FOR USER INPUT]
[INST] Your instruction here [/INST]
Select [1-4]:  # Placeholder for selection

# ✅ Good - Data boundaries
<email>content here</email>
<user_input>question</user_input>

# ❌ Bad - Weak instruction
Wait for user confirmation
Please stop here
```

## Bash Command Execution in Slash Commands
**When**: Ensuring bash commands execute in custom slash commands
- Must include `allowed-tools: Bash(command:*)` in frontmatter
- Use `!` prefix with backticks: `!`command``
- Command output automatically included in context

```markdown
# ✅ Good - Proper bash execution setup
---
allowed-tools: Bash(git status:*), Bash(git diff:*)
---
- Current status: !`git status`
- Changes: !`git diff HEAD`

# ❌ Bad - Missing required elements
---
description: Git status check
---
- Status: `git status`  # Won't execute without ! and allowed-tools
```

## Positional Arguments Usage
**When**: Designing slash command arguments
- Use `$1`, `$2` for required, ordered arguments with clear roles
- Use `[hint]` or flags for optional, flexible arguments
- Positional args suit fixed sequences (e.g., PR number, priority, assignee)
- Avoid positional args when all arguments are optional

```bash
# ✅ Good - Clear required sequence
/review-pr $1 $2 $3  # PR#, priority, assignee
/review-pr 456 high alice

# ❌ Bad - Optional/flexible arguments
/steering-remember $1 $2 $3  # hint?, format?, type?
/steering-remember "" "" security  # Awkward empty args
```

## Preventing False Reporting
**When**: Designing AI commands that modify files
- Add explicit "Will Not" boundaries to prevent success claims without actual tool usage
- Use strong prohibitive language: "Report success without actually using Edit/Write tools"
- Include verification requirements in command specifications
- Test commands to ensure they actually perform file modifications

```markdown
# ✅ Good - Strong boundaries
**Will Not:**
- Report success without actually using Edit/Write tools to modify files
- Claim completion without verifying file changes

# ❌ Bad - Weak or missing boundaries
**Will Not:**
- Skip file operations (too vague)
```

## Slash Command Structure
**When**: Creating slash commands with logical execution flow
- YAML frontmatter for static configuration
- 7 mandatory sections following execution flow
- Tool Coordination includes both Claude Code Tools and MCP Integration
- Key Patterns positioned before Behavioral Flow for strategy determination

### Required Frontmatter Structure
```yaml
---
name: command-name                    # Command identifier
description: "Brief command purpose"  # Quoted description
category: utility|workflow|special|session
complexity: basic|standard|advanced|high
mcp-servers: [list]                  # Empty array if none
personas: [list]                     # Empty array if none
allowed-tools: Tool1, Tool2, mcp__server__tool
argument-hint: "[--type prd|bug] [--issue <url>]"  # Quote if contains pipes
---
```

### Section Order (Execution Flow)
1. **## Triggers** - When/why command is used, specific scenarios
2. **## Usage** - Command syntax with options inline (no separate Options section)
3. **## Key Patterns** - Input transformation rules (IF-THEN patterns)
4. **## Boundaries** - What the command will and will not do
5. **## Tool Coordination** - Claude Code Tools + MCP Integration subsections
6. **## Behavioral Flow** - Execution steps with Key behaviors embedded
7. **## Examples** - 3-4 concrete usage examples with code blocks

### Section Responsibilities
- **Triggers**: 4 bullet points covering main activation scenarios
- **Usage**: Command syntax with inline option explanations
- **Key Patterns**: Transform input → strategy (type detection, source routing, complexity assessment)
- **Boundaries**: Will/Will Not statements establishing execution context
- **Tool Coordination**:
  - Claude Code Tools: Read, Write, Edit, Bash, etc.
  - MCP Integration: External server connections and usage
- **Behavioral Flow**: 5-step execution with Key behaviors paragraph
- **Examples**: Complete scenarios showing full command execution

### Design Philosophy
```
Input → Interpretation → Validation → Preparation → Execution
```
This structure follows actual program execution flow, enabling:
- Early pattern matching and strategy decisions
- Boundary validation before execution
- Clear dependency chain through sections
- Logical flow from trigger to completion

````markdown
# ✅ Good - Logical execution flow structure
---
name: requirements
description: "Generate structured requirement documents"
category: workflow
complexity: standard
mcp-servers: [github]
---

## Triggers
- Starting new feature development
- Bug reporting needs documentation
- GitHub issue conversion needed

## Usage
```
/spec:requirements [--type prd|bug] [--issue <github-url>]
```
- `--type`: Document type (prd or bug)
- `--issue`: Optional GitHub issue URL

## Key Patterns
- **Type Detection**: --type prd → PRD template activation
- **Source Detection**: --issue present → GitHub MCP activation
- **Complexity Assessment**: PRD → multiple iterations

## Boundaries
**Will:**
- Generate requirements.md only
- Iterate based on feedback

**Will Not:**
- Perform design or investigation
- Auto-generate without confirmation

## Tool Coordination
**Claude Code Tools:**
- **Read**: Check existing requirements
- **Write/Edit**: Update document

**MCP Integration:**
- **GitHub**: Fetch issue content

## Behavioral Flow
1. Initialize and apply patterns
2. Gather requirements
3. Generate document
4. Refinement loop
5. Finalize

Key behaviors:
- Interactive refinement
- Completeness tracking

## Examples
[Usage scenarios...]

# ❌ Bad - Old structure mixing concerns
## Tool Usage
- Use Read tool to inspect
## Options
- `--focus`: Analysis focus
## MCP Integration
- Separate from tools
````

## Key Behaviors vs Key Patterns
**When**: Writing SuperClaude Framework slash commands
- **Key Behaviors**: コマンドの動作特性（どう動作するか）
- **Key Patterns**: 処理フローパターン（どう処理するか）
- Key Behaviors is embedded within Behavioral Flow section
- Key Patterns is independent section with arrow notation (→)

### Conceptual Difference
**Key Behaviors**: Behavioral Flowの中で記述される「動作の特性」
- 実行時の振る舞いを説明（インタラクティブ、自動検出、並列実行など）
- Flowの各ステップがどのように動作するかを補足

**Key Patterns**: 独立セクションで記述される「変換ルール」
- 入力を出力に変換する「交差点での判断基準」
- 同じFlowでも異なる結果を生み出すための処理パターン

### Relationship Example
```yaml
Behavioral Flow (固定の道順):
  1. Analyze: 入力を分析する
     Key Behavior: "Auto-detect format based on content"
     Key Pattern適用: "Context Detection: API keyword → Backend persona"

  2. Execute: 処理を実行する
     Key Behavior: "Parallel execution for efficiency"
     Key Pattern適用: "Multi-Persona: Backend + Security → Secure API code"
```

### Format Differences
```markdown
# ✅ Good - Key Behaviors (within Behavioral Flow)
Key behaviors:
- Interactive type selection with intelligent suggestions
- Auto-detect format based on content patterns
- Maximum brevity with concrete examples
- Parallel execution for multi-file operations

# ✅ Good - Key Patterns (independent section)
## Key Patterns
- **Learning Extraction**: Conversation analysis → actionable knowledge → title generation
- **Type Matching**: Content analysis → criteria comparison → confidence scoring
- **Context Detection**: Framework/library → appropriate MCP server activation
- **Severity Assessment**: Issue classification → prioritized recommendations

# ❌ Bad - Mixing concepts
## Key Behaviors
- **Learning Flow**: Analysis → extraction → formatting (should be Key Pattern)
- Interactive selection (correct for behaviors)
```

### Detailed Pattern Examples

#### Example 1: `/sc:improve` with Different Inputs
```yaml
Input A: "/sc:improve --type performance"
Behavioral Flow Step 1 - Analyze:
  Key Behavior: "Examines codebase systematically"
  Key Pattern: "Performance Optimization: profiling → bottleneck identification"
  Result: Performance personaを活性化、プロファイリングツール選択

Input B: "/sc:improve --type security"
Behavioral Flow Step 1 - Analyze:
  Key Behavior: "Examines codebase systematically" (同じBehavior)
  Key Pattern: "Security Hardening: vulnerability scan → threat modeling"
  Result: Security personaを活性化、OWASP検査ツール選択
```

#### Example 2: `/sc:analyze` Domain-Specific Transformation
```yaml
Key Pattern: "Domain Analysis: Quality/Security/Performance → specialized assessment"

実行例:
- Quality domain選択時:
  Behavior変化: 静的解析ツールを使用、コード複雑度を測定
  期待結果: 技術的負債レポート、リファクタリング推奨事項

- Security domain選択時:
  Behavior変化: 脆弱性スキャナーを使用、依存関係を検査
  期待結果: セキュリティリスクレポート、修正優先度リスト

- Performance domain選択時:
  Behavior変化: プロファイラーを使用、ボトルネック分析
  期待結果: パフォーマンスメトリクス、最適化提案
```

#### Example 3: `/sc:implement` Multi-Persona Coordination
```yaml
Key Pattern: "Multi-Persona: Frontend + Backend + Security → comprehensive solution"

シナリオ: "/sc:implement user dashboard with real-time updates"

Pattern適用前のBehavior:
- 単一視点でのコード生成
- 基本的な機能実装のみ

Pattern適用後のBehavior変化:
- Frontend persona: WebSocket接続、React状態管理、UIコンポーネント
- Backend persona: リアルタイムAPI、データストリーミング、キャッシュ戦略
- Security persona: 認証トークン検証、レート制限、XSS対策

期待される統合結果:
- セキュアなWebSocket接続を持つダッシュボード
- 効率的なデータ更新メカニズム
- 包括的なエラーハンドリングと認証
```

### Pattern Impact Summary
**Key Patterns**は「変換ルール」として機能し、同じBehavioral Flowでも：
1. 入力に応じて異なるツール・personaを選択
2. ドメインに特化した処理方法を適用
3. 複数の専門知識を統合して包括的な解決策を生成

これにより、固定的なFlowに柔軟性と適応性を与えています。

## MCP Slash Commands
**When**: Using MCP server exposed slash commands
- Commands follow pattern: `/mcp__<server-name>__<prompt-name> [arguments]`
- Dynamically discovered from connected MCP servers
- Arguments defined by the server, not frontmatter
- Use `/mcp` command to view available servers and prompts

```
/mcp__github__list_prs
/mcp__github__pr_review 456
```

## Model Selection in Slash Commands
**When**: Commands need specific Claude models
- Use `model:` frontmatter field to override conversation model
- Supports specific model strings (claude-3-5-haiku-20241022, etc.)
- Inherits from conversation if not specified

```yaml
---
model: claude-3-5-haiku-20241022
description: Fast lightweight task
---
```

## MCP Tool Permissions
**When**: Configuring permissions for MCP tools
- **No wildcards supported**: `mcp__github__*` is invalid
- Use server name only: `mcp__github` (approves ALL tools)
- Use specific tools: `mcp__github__get_issue` (single tool)
- List each tool individually for granular control

```
# ✅ Good - All tools from server
mcp__github

# ✅ Good - Specific tool
mcp__github__get_issue

# ❌ Bad - Wildcards not supported
mcp__github__*
```

## YAML Frontmatter argument-hint
**When**: Using argument-hint in slash command YAML frontmatter
- Quote the value when it contains pipe characters (`|`)
- Pipe characters have special meaning in YAML and cause parsing errors
- Simple arguments without special chars don't need quotes
- Alternative: escape pipes with `\|` instead of quoting

```yaml
# ✅ Good - Quoted when containing pipes
argument-hint: "[hint] [--format rule|guide|knowledge] [--type <name>]"

# ✅ Good - Simple arguments without quotes
argument-hint: [message]
argument-hint: [pr-number] [priority] [assignee]

# ✅ Good - Escaped pipes
argument-hint: add [tagId] \| remove [tagId] \| list

# ❌ Bad - Unquoted pipes cause YAML parse errors
argument-hint: [hint] [--format rule|guide|knowledge] [--type <name>]
```

## System Prompt XML Structure
**When**: Designing system prompts for Claude Code
- Claude is trained to pay special attention to XML tag structures
- Use hybrid approach: flat for main flows, nested for details
- Naming conventions with prefixes (`kiro-*`) for related tags
- Balance between quick access and logical grouping

```xml
# ✅ Good - Hybrid structure
<!-- Flat: Quick access to main flows -->
<kiro-triggers>
- requirements: 要件 → <kiro-requirements-flow>
</kiro-triggers>

<kiro-requirements-flow>
1. Read existing
2. Apply template
3. Show suggestions
</kiro-requirements-flow>

<!-- Nested: Grouped details -->
<kiro-requirements-suggestions>
  <before-update>
    - "更新しますか？"
    - Show completeness
  </before-update>
  <after-update>
    - "追加要件？"
    - "次は調査？"
  </after-update>
</kiro-requirements-suggestions>

# ❌ Bad - Deep nesting
<kiro-spec-behaviors>
  <recognition>
    <triggers>
      <!-- Hard to find quickly -->
    </triggers>
  </recognition>
</kiro-spec-behaviors>
```

## XML-based Orchestration Framework
**When**: Solving Lost in the Middle problem for system prompts
- Split monolithic system prompts into flat XML tag sections
- Slash commands reference only needed tags, not entire prompt
- Separation of concerns: triggers in commands, rules in system prompt
- Enables pinpoint reference to specific behaviors

### System Prompt Structure
```xml
<kiro-spec-driven>
  <!-- Common orchestration elements -->
  <kiro-philosophy>Core reactive patterns</kiro-philosophy>
  <kiro-tasks-hub>Tasks.md central control</kiro-tasks-hub>
  <kiro-orchestration>Operation sequences</kiro-orchestration>
  <kiro-nudging>Post-action prompts</kiro-nudging>

  <!-- Action-specific elements -->
  <kiro-requirements>Requirements templates</kiro-requirements>
  <kiro-investigation>Investigation behaviors</kiro-investigation>
  <kiro-design>Design validation</kiro-design>

  <kiro-spec-files>Current spec paths</kiro-spec-files>
</kiro-spec-driven>
```

### Slash Command Reference Pattern
```markdown
# ✅ Good - Minimal command with tag references
---
name: investigate
description: "Technical investigation (investigate/research/調査)"
---

# /spec:investigate

Execute investigation following Kiro methodology.

Refer to system prompt sections:
- <kiro-philosophy> for reactive orchestration
- <kiro-tasks-hub> for critical update timing
- <kiro-orchestration> for operation sequence
- <kiro-investigation> for investigation behaviors
- <kiro-nudging> for post-investigation prompts

All execution details defined in these sections.

# ❌ Bad - Duplicating rules in command
## /spec:investigate

1. Check requirements completion
2. Update tasks.md before starting
3. Track confidence percentages
[100+ lines of detailed rules]
```

## Pattern Router System Prompt Centrism
**When**: Extending Pattern Router Framework with new features (e.g., Review Pipeline)
- All logic and templates in system prompt (pattern_router/*.md files)
- Slash commands reference XML tags only, no embedded logic
- New features as independent pipelines, not command modifications
- Reuse existing components (patterns, nudges, workflows) over creating new ones
- Zero slash command changes when possible

````markdown
# ✅ Good - Review Pipeline with system prompt centrism
Files changed:
- 03_patterns.md: Add EXPLICIT_REVIEW pattern class
- 04_workflows.md: Add Review Pipeline section
- 06_nudges.md: Add natural language templates
- Slash commands: NO CHANGES

Review activation:
User: /spec:requirements --review
→ patterns detects --review flag
→ Routes to Review Pipeline (defined in 04_workflows.md)
→ Uses nudges templates (defined in 06_nudges.md)
→ Handoff to Command Pipeline on approval

Result: Feature complete without touching slash commands

# ✅ Good - Before/After Protocol reuse
Review Pipeline:
- Generate draft (ephemeral)
- User approves
- Handoff to Command Pipeline
- Command Pipeline executes BEFORE/AFTER protocols
- No duplicate implementation

# ❌ Bad - Slash command logic duplication
Files changed:
- requirements.md: Add review behavioral flow (50+ lines)
- design.md: Add review behavioral flow (50+ lines)
- investigate.md: Add review behavioral flow (50+ lines)

Result: DRY violation, maintenance burden

# ❌ Bad - Embedding in Command Pipeline
Command Pipeline modified:
patterns → hub → gates → [review logic here] → workflows → document

Problems:
- Existing Command Pipeline changed
- No opt-in mechanism
- Affects all commands
- Hard to disable
````

**Key Principles:**
- **System Prompt = Intelligence**: All logic lives in pattern_router/*.md
- **Slash Commands = Triggers**: Minimal references to XML tags
- **Pipelines = Composition**: Combine existing components, avoid modifications
- **Protocol Reuse = DRY**: Leverage existing workflows, gates, nudges
- **Zero Command Changes**: Ideal when adding features