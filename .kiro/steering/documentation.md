# documentation

## Markdown Code Block Nesting
**When**: Writing documentation with nested code blocks in markdown
- Parent blocks must use 4 backticks (````)
- Child blocks must use 3 backticks (```)
- Ensures proper rendering in markdown preview

### ✅ Good
````
## Example Section

```javascript
const x = 1
```

```python
def hello():
    pass
```
````

### ❌ Bad
```
## Example Section

```javascript
const x = 1
```
```

## UTF-8 File Encoding
**When**: Claude Code writes files with Japanese or non-ASCII characters
- Always use UTF-8 encoding when creating files with Write tool
- Japanese text will become corrupted (文字化け) without proper encoding
- If file shows garbled characters, rewrite with explicit UTF-8

```markdown
# ✅ Good - File written with UTF-8
# Steering Remind 軽量化設計
設計書を作成する

# ❌ Bad - File written without UTF-8
# Steering Remind ��-
������3��Y�
```

## Slash Command Design Process
**Context**: Implementing new slash commands for Kiro specs
1. Read reference documentation (`@reference/slash-commands.md`, `@reference/slash-command-structure.md`)
2. Understand user requirements through dialogue
3. Create design document following `@.kiro/archive/2025-09-16-investigate/design.md` format
4. Read existing `<kiro_design>` first, then Write/MultiEdit to save
5. Include Japanese overview and embedded command markdown
⚠️ Use 4-backtick blocks when embedding command markdown in design
