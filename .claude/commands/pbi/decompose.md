---
name: decompose
description: "Decompose PBI into separate SBI specifications"
allowed-tools: Read, Write, Glob, Bash, MultiEdit
---

# /decompose

Decompose Product Backlog Item into Sprint Backlog Items.

## Boundaries

### Will
- **Parse PBI requirements.md** - Extract all `### sbi-X-[title]` sections with types
- **Validate SBI names** - Enforce lowercase kebab-case format
- **Create SBI directories** - One directory per SBI with requirements.md only
- **Apply correct templates** - Use PRD/Bug/Tech templates based on type

### Will Not
- **Proceed without PBI requirements.md** - Must have PBI requirements first
- **Create tasks.md/memo.md** - SBI files created when developer starts working
- **Create PBI tasks.md** - Pattern Router doesn't manage PBI-level tasks.md
- **Overwrite existing SBIs** - Error if SBI directory already exists

## Behavioral Flow

1. **Read PBI requirements.md**
   - Verify PBI requirements.md(<requirements-file>) exists
   - Parse all `### sbi-X-[title]` sections
   - Extract `requirements type: [prd|bug|tech]` from each section
   - Validate: All SBI titles must be lowercase kebab-case

2. **Interactive Confirmation**
   ```
   Found 3 SBIs in requirements.md:
     1. sbi-1-backend-api (type: prd)
     2. sbi-2-frontend-ui (type: prd)
     3. sbi-3-error-handling (type: tech)

   Decompose into separate directories?
   ```

3. **Create SBI Directories**
   For each SBI:
   - Create `sbi-X-[title]/` directory
   - Generate `requirements.md` using appropriate template from <kiro-requirements>:
     - type=prd → PRD Template
     - type=bug → Bug Template
     - type=tech → Tech Template
   - Populate with detailed requirements from PBI section content

4. **Nudge**
   ```
   ✅ SBIs created! Select specific SBI to work on:
   $ hail-mary code → [pbi-name] → sbi-1-backend-api

   Each SBI will generate tasks.md/memo.md when you start working with /hm:investigate or /hm:design.
   ```

## Key Patterns

- **Section Parsing**: `### sbi-X-[title]` → directory name `sbi-X-[title]/`
- **Type-based Template**: Extract type → Apply PRD/Bug/Tech template
- **Content Transfer**: PBI section content → SBI requirements.md detailed content
- **Validation First**: Check all SBI names before creating any directories

Refer to system prompt sections:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-requirements> for template selection (PRD/Bug/Tech)
