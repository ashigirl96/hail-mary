---
description: Process and categorize steering drafts into appropriate files
allowed-tools: Read, Write, MultiEdit, Glob, Bash(ls:*, rm:*, cp:*, mkdir:*)
argument-hint: [--verbose] [--dry-run]
---

## Triggers
- Drafts accumulated in @.kiro/steering/draft/ need processing
- User wants to organize and categorize saved learnings
- Periodic maintenance of draft directory

## Usage
```
/hm:steering-merge [--verbose] [--dry-run]
```

## Behavioral Flow

1. **Discover**: Find all draft files in @.kiro/steering/draft/ using Glob tool
   - Draft count: !`ls .kiro/steering/draft/*.md 2>/dev/null | wc -l`
   
2. **Backup**: Create @.kiro/steering/backup/ and copy existing steering files
   - !`mkdir -p .kiro/steering/backup && cp .kiro/steering/*.md .kiro/steering/backup/ 2>/dev/null || true`
   
3. **Categorize**: For each draft:
   - Read content and analyze against criterions in @.kiro/config.toml
   - Match draft content to most appropriate steering type
   - Use $1 for verbose output of categorization logic
   
4. **Append**: Use MultiEdit tool to add categorized content to steering files
   - Preserve existing content
   - Add new content with appropriate formatting
   - Use $2 for dry-run mode (preview without changes)
   
5. **Clean**: Remove successfully processed drafts using Bash(rm:*)
   - Delete draft files after successful categorization
   - Keep failed drafts with error log

Key behaviors:
- Never overwrite existing steering content
- Always backup before modifications  
- Clear feedback on categorization decisions
- Graceful handling of categorization failures

## Examples

### Standard Processing
```
/hm:steering-merge
# Processes all drafts, categorizes, and updates steering files
```

### Verbose Mode
```
/hm:steering-merge --verbose
# Shows detailed categorization logic for each draft
```

### Dry Run
```
/hm:steering-merge --dry-run
# Preview categorization without making changes
```

### Combined Options
```
/hm:steering-merge --verbose --dry-run
# Detailed preview of what would happen
```

## Boundaries

**Will:**
- Process all draft files in @.kiro/steering/draft/
- Categorize based on criterions in @.kiro/config.toml
- Backup existing steering files before modification
- Use MultiEdit for safe content appending
- Provide verbose categorization reasoning when requested

**Will Not:**
- Overwrite existing steering file content
- Process drafts without proper categorization
- Delete drafts that failed to categorize
- Modify files without backing them up first