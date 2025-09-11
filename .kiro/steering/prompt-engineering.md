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