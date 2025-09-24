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
/hm:requirements [--type prd|bug] [--issue <github-url>]
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

## Parallel Task Tool Execution
**When**: Designing commands that require concurrent Task agent execution
- Use explicit "parallel" keywords 15+ times throughout documentation
- Include bold instruction: **[send multiple Task tool calls in one response]**
- Structure independent agent missions with clear boundaries
- Repeat "independently/concurrent" keywords for reinforcement
- Visual representation with bullet lists of agents

### Required Section Elements
```markdown
# Behavioral Flow
- Step: "Launch **parallel** Task agents **independently**"
- Step: "Process tasks **concurrently** across multiple agents"

# Execution Phase  
**Title**: Include "Parallel" (e.g., "Parallel Execution Phase")
**Bold Instruction**: **[The implementation will send multiple Task tool calls in one response]**
**Agent List**: Show numbered/bulleted list of parallel agents

# Independent Agent Structure
Each parallel task must:
- Have clear, self-contained scope
- Operate without dependencies on other agents
- Define specific deliverables
- Include "independent" or "concurrent" in description

# Tool Coordination
- Task: Spawn **parallel** agents for **concurrent** execution
- "Multiple Task tools sent in single message"
- "Each agent operates independently with its own context"

# Key Patterns
- First pattern: "**Parallel Execution**" or "**Concurrent Processing**"
- Include: "Multiple agents → simultaneous execution → aggregated results"
```

### ✅ Good - Parallel Execution
```markdown
> 🚀 Launching parallel investigation for 3 types...
> Spawning investigation agents:
> • [Agent 1] type1 - purpose1
> • [Agent 2] type2 - purpose2  
> • [Agent 3] type3 - purpose3
> [Parallel Task agents processing independently...]
```

### ❌ Bad - Sequential Execution
```markdown
> Starting investigation...
> Processing type1...
> Then processing type2...
> Finally processing type3...
```

## Task Tool Parameterization
**When**: Passing arguments to subagents via Task tool
- Use detailed prompt parameter to pass context dynamically
- Single subagent can adapt behavior based on received parameters
- Enables reusable subagents for multiple contexts

```python
# ✅ Good - Dynamic argument passing
Task(
    subagent_type="steering-investigator",
    description="Verify tech steering type",
    prompt=f"""
    Steering Type: {type_name}
    Purpose: {purpose}
    Criteria: {criteria_list}
    File Path: .kiro/steering/{type_name}.md

    Your mission: Investigate against these criteria
    """
)

# ❌ Bad - Multiple specialized subagents
Task(subagent_type="tech-steering-investigator")
Task(subagent_type="product-steering-investigator")
```

## Framework Standards vs Execution Reproducibility
**When**: Designing complex slash commands with detailed behavioral flows
- Framework compliance favors concise 5-step Behavioral Flow structure
- Execution reproducibility requires detailed implementation sections with explicit instructions
- Complex commands prioritize reproducibility over framework adherence
- Simple commands can follow standard framework structure

```markdown
# ✅ Good - Complex command prioritizing reproducibility
## Behavioral Flow
1-5 overview steps

### Detailed Implementation Phase
Execute backup command: !`hail-mary steering backup`
**[The implementation will send multiple Task tool calls...]**
[Specific code examples and stop markers]

# ✅ Good - Simple command following framework standards
## Behavioral Flow
1. Step one overview
2. Step two overview
3. Step three overview
4. Step four overview
5. Step five overview

Key behaviors:
- Behavior description
- Implementation approach

# ❌ Bad - Complex command sacrificing reproducibility for framework compliance
## Behavioral Flow
1. Backup: Create timestamped backup
2. Load: Parse allowed steering types
3. Investigate: Launch parallel Task agents
4. Aggregate: Collect results
5. Update: Apply changes
```

## Preventing Unnecessary File Exploration
**When**: Handling file-not-found errors in slash commands
- Attempt Read once, then proceed if error
- Never use ls/Bash/Glob to search for missing files
- Let Write/MultiEdit create directories automatically
- Make error handling explicit in Behavioral Flow

### Tool Coordination
```markdown
# ✅ Good
**Claude Code Tools:**
- **Read**: Attempt to read (ignore errors if file doesn't exist)
- **Write/MultiEdit**: Create or update (Write creates parent directories automatically)

# ❌ Bad
**Claude Code Tools:**
- **Read**: Read file to understand context
- **Bash**: Use ls to find files if needed
```

### Behavioral Flow
```markdown
# ✅ Good
1. **Initialize**: Parse arguments and setup
- **Attempt** to Read <file_path> for existing content:
  - If file exists: Load and analyze
  - If file not found: Skip silently and proceed to step 2
  - **DO NOT**: Use ls, Bash, or Glob to search for files
  - **DO NOT**: Create directories or investigate structure

# ❌ Bad
1. **Initialize**: Parse arguments and setup
- Read existing file for context
- If error, check directory structure
- Create directories if needed
```

## Behavioral Flow Flag Conditions
**When**: Writing conditional logic in Behavioral Flow sections based on command flags
- Use clear If statements with bold formatting: **If `--flag` provided:**
- Group related conditions under same If block
- Separate different flag scenarios with clear visual hierarchy
- Include "If no flags provided" as catch-all scenario

```markdown
# ✅ Good
1. **Initialize & Topic Gathering**: Parse arguments and determine topic
   - Read existing <kiro_investigation_path>

   - **If `--for requirements` or `--for design` provided:**
     - Read corresponding document
     - Analyze for technical gaps
     - Display suggestions
     - Ask: "What would you like to investigate?"

   - **If `--topic <name>` provided:**
     - Search for existing topic section
     - Load previous investigation
     - Display follow-up suggestions
     - Ask: "What would you like to investigate?"

   - **If no flags provided:**
     - Ask: "What would you like to investigate?"
     - Sub-prompt: "[Provide technical question]"

# ❌ Bad
1. **Initialize**: Check flags
   - Check if --for flag exists
   - Check if --topic flag exists
   - Handle different cases
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