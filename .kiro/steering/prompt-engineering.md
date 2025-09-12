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

## SuperClaude Framework Slash Command Structure
**When**: Creating slash commands following SuperClaude Framework standards
- YAML frontmatter with specific required fields
- 8 mandatory sections in consistent order
- MCP Integration section when MCP servers used
- Tool Coordination naming over Tool Usage

### Required Frontmatter Structure
```yaml
---
name: command-name                    # Command identifier
description: "Brief command purpose"  # Quoted description
category: utility|workflow|special|session
complexity: basic|standard|advanced|high
mcp-servers: [list]                  # Empty array if none
personas: [list]                     # Empty array if none
---
```

### Mandatory Section Order
1. **## Triggers** - When/why command is used, specific scenarios
2. **## Usage** - Command syntax with options inline (no separate Options section)
3. **## Behavioral Flow** - 5-step numbered process with Key behaviors subsection
4. **## MCP Integration** - Only if mcp-servers specified in frontmatter
5. **## Tool Coordination** - Tools used
6. **## Key Patterns** - Arrow notation patterns (A → B → C)
7. **## Examples** - 3-4 concrete usage examples with code blocks
8. **## Boundaries** - Will/Will Not format

### Section Content Guidelines
- **Triggers**: 4 bullet points covering main use cases
- **Usage**: Options explained directly under usage block, no separate section
- **Behavioral Flow**: 5 steps + Key behaviors paragraph
- **Tool Coordination**: Tool names with descriptions, not usage instructions
- **Key Patterns**: 4 patterns using arrow notation (→)
- **Examples**: Realistic scenarios with actual command syntax
- **Boundaries**: Clear Will/Will Not statements

````markdown
# ✅ Good - SuperClaude Framework compliance
---
name: analyze
description: "Code analysis across quality, security, performance domains"
category: utility
complexity: basic
---

## Triggers
- Code quality assessment requests
- Security vulnerability scanning needs
- Performance bottleneck identification
- Architecture review requirements

## Usage
```
/sc:analyze [target] [--focus quality|security] [--depth quick|deep]
```
- `--focus`: Analysis domain focus
- `--depth`: Analysis thoroughness level

## Tool Coordination
- **Read**: Source code inspection and analysis
- **Grep**: Pattern analysis and code search
- **Write**: Report generation and documentation

# ❌ Bad - Non-compliant structure
## Options
- `--focus`: Analysis focus
- `--depth`: Analysis depth

## Tool Usage
- Use Read tool to inspect source code
- Use Grep tool to search for patterns
````

## Key Behaviors vs Key Patterns
**When**: Writing SuperClaude Framework slash commands
- **Key Behaviors**: コマンドの動作特性（どう動作するか）
- **Key Patterns**: 処理フローパターン（どう処理するか）
- Key Behaviors is embedded within Behavioral Flow section
- Key Patterns is independent section with arrow notation (→)

### Format Differences
```markdown
# ✅ Good - Key Behaviors (within Behavioral Flow)
Key behaviors:
- Interactive type selection with intelligent suggestions
- Auto-detect format based on content patterns
- Maximum brevity with concrete examples

# ✅ Good - Key Patterns (independent section)
## Key Patterns
- **Learning Extraction**: Conversation analysis → actionable knowledge → title generation
- **Type Matching**: Content analysis → criteria comparison → confidence scoring

# ❌ Bad - Mixing concepts
## Key Behaviors
- **Learning Flow**: Analysis → extraction → formatting (should be Key Pattern)
- Interactive selection (correct for behaviors)
```

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