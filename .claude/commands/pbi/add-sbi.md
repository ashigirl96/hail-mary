---
name: add-sbi
description: "Add new SBI to existing PBI"
allowed-tools: Read, Write, Edit
argument-hint: "<sbi-name>"
---

# /add-sbi

Add a new Sprint Backlog Item to the current Product Backlog Item.

## Usage

```
/add-sbi monitoring        # Creates sbi-4-monitoring (auto-numbered)
/add-sbi error-handling    # Creates sbi-5-error-handling
```

## Boundaries

### Will
- **Verify PBI context** - Check current spec is a PBI
- **Auto-number SBI** - Calculate next available SBI number automatically
- **Validate SBI name** - Enforce lowercase kebab-case format
- **Interactive type selection** - Let user choose PRD/Bug/Tech
- **Create SBI directory** - With requirements.md only
- **Update PBI requirements.md** - Add new SBI section

### Will Not
- **Proceed without argument** - Must provide `<sbi-name>`
- **Work outside PBI** - Error if not in PBI context
- **Accept invalid names** - Reject non-kebab-case names
- **Create tasks.md/memo.md** - SBI files created when developer starts working
- **Create PBI tasks.md** - Pattern Router doesn't manage PBI-level tasks.md

## Behavioral Flow

1. **Validate Arguments**
   ```
   If $ARGUMENTS is empty:
     "Usage: /add-sbi <sbi-name>
      Example: /add-sbi monitoring"
     Exit

   If $ARGUMENTS not kebab-case:
     "SBI name must be lowercase kebab-case
      Invalid: $ARGUMENTS
      Valid examples: backend-api, error-handling, monitoring"
     Exit
   ```

2. **Context Verification**
   - Verify current spec is a PBI:
     - Check if PBI requirements.md exists with SBI sections, OR
     - Check if sbi-X/ directories exist
   - If not PBI context:
     ```
     "Error: Not in PBI context
      This command requires a PBI specification.
      Use /hm:requirements --type pbi first to create a PBI."
     Exit
     ```

3. **Auto-number SBI**
   - List existing SBIs: Use Glob to find `sbi-*/` directories
   - Extract numbers: `sbi-1`, `sbi-2`, `sbi-3` → [1, 2, 3]
   - Calculate next: `max([1, 2, 3]) + 1` → 4
   - Generate full name: `sbi-4-{$ARGUMENTS}`
   - Example: `monitoring` → `sbi-4-monitoring`

4. **Interactive Type Selection**
   ```
   Creating: sbi-4-monitoring

   Select SBI type:
   1. PRD (Product feature)
   2. Bug (Bug fix)
   3. Tech (Technical improvement)
   →
   ```

5. **Create SBI**
   - Create `sbi-{N}-{$ARGUMENTS}/` directory
   - Generate `requirements.md` with selected template from <kiro-requirements>

6. **Update PBI requirements.md**
   - Append new section at the end:
     ```markdown
     ### sbi-{N}-{$ARGUMENTS}
     requirements type: {selected-type}
     [Description placeholder - edit this section with details]
     ```

7. **Nudge**
   ```
   ✅ SBI created: sbi-4-monitoring

   Next steps:
   1. Edit description in PBI requirements.md (section ### sbi-4-monitoring)
   2. Select SBI to work: $ hail-mary code → [pbi-name] → sbi-4-monitoring
   3. Start with /hm:investigate or /hm:design
   ```

## Key Patterns

- **Auto-numbering**: Sequential SBI numbers based on existing SBIs
- **Template Selection**: Interactive type selection → PRD/Bug/Tech template
- **Validation First**: Check name format and context before any operations
- **PBI Update**: Append section to PBI requirements.md for consistency

Refer to system prompt sections:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-requirements> for template selection (PRD/Bug/Tech)
