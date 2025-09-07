---
description: Save new learnings not already in steering files as draft
allowed-tools: Read, Write, Bash(date:*), Glob
argument-hint: [title] [--verbose]
---

## Triggers
- User identifies new learning or pattern to remember
- Context needs to be preserved for future reference  
- Information not already documented in existing steering files

## Usage
```
/hm:steering-remember [title] [--verbose]
```

## Behavioral Flow

1. **Validate**: Check if content represents new learning not in existing steering files using Glob and Read tools
2. **Generate**: Create descriptive filename using timestamp: !`date +%Y%m%d-%H%M%S`-$1.md
3. **Save**: Write draft to @.kiro/steering/draft/ with structured content using Write tool
4. **Confirm**: Provide user feedback on successful save or any errors

Key behaviors:
- Only save genuinely new learnings not already documented
- Use descriptive titles for easy identification
- Include context about why this learning is important
- Structure content for easy categorization later

## Examples

### Basic Usage
```
/hm:steering-remember "const-vs-function"
# Saves learning about const vs function preference to draft
```

### Verbose Mode
```
/hm:steering-remember "api-design-pattern" --verbose
# Saves with detailed output showing what was captured
```

## Boundaries

**Will:**
- Create timestamped draft files in @.kiro/steering/draft/
- Capture conversation context for new learnings
- Provide clear feedback on save operations
- Structure content for later categorization

**Will Not:**
- Overwrite existing draft files
- Save duplicate information already in steering files
- Create drafts without meaningful titles