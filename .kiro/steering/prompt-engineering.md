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