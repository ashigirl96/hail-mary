---
name: requirements
description: "Create/update requirements and user stories - triggered by: requirements, PRD, features, bug, 要件, バグ"
argument-hint: "[--type prd|bug|tech|pbi] [--issue <github-url>]"
---

# /spec:requirements

Create or update requirements documentation with reactive pattern-based routing.

Follow <kiro-workflows> Command Pipeline:
- Before documentation: execute event id="requirements:pre-action"
- After complete: execute event id="requirements:post-action"
- Next action: execute event id="requirements:nudge-next" from <kiro-nudges>

Additional context:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-hub> for tasks.md central control mechanism
- <kiro-patterns> for pattern recognition and mapping
- <kiro-gates> for validation gates (if any)
- <kiro-requirements> for requirements document structure and template selection

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