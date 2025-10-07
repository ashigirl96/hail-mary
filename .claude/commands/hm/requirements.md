---
name: requirements
description: "Create/update requirements and user stories - triggered by: requirements, PRD, features, bug, 要件, バグ"
allowed-tools: Read, Write, MultiEdit, Grep, Glob
argument-hint: "[--type prd|bug|tech|pbi] [--issue <github-url>]"
---

# /hm:requirements

Create or update requirements documentation with reactive pattern-based routing.

Refer to system prompt sections:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-hub> for tasks.md central control mechanism
- <kiro-patterns> for pattern recognition and mapping
- <kiro-workflows> Before documentation: `requirements:pre-action`, After complete: `requirements:post-action`
- <kiro-gates> for validation gates (if any)
- <kiro-nudges> Next action suggestion: `requirements:nudge-next`
- <kiro-requirements> for requirements document structure and template selection

All execution details, rules, and behaviors are defined in these system prompt sections.

## Key Patterns

**Type Detection**:
- **Explicit type**: `--type` flag provided → Use specified type directly
  - `--type pbi` → PBI template (for multi-PR projects)
  - `--type prd` → PRD template (product features)
  - `--type bug` → Bug template (bug fixes)
  - `--type tech` → Tech template (technical improvements)
- **Implicit detection**: No `--type` flag → Analyze user input:
  - Technical keywords ("update", "upgrade", "refactor", "migrate", "dependency") → Tech type
  - Error indicators ("bug", "broken", "error", "not working", "crash") → Bug type
  - Business/feature language (default) → PRD type