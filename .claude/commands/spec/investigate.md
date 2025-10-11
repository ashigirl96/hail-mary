---
name: investigate
description: "Technical investigation for Kiro specs - triggered by: investigate, research, verify, explore, 調査, 検証"
argument-hint: "[--topic <name>] [--deep]"
---

# /spec:investigate

Conduct technical research and append findings with reactive pattern-based routing.

**Depth-Based Routing**:
- Timeline with (deep-dive) label → Use Task agent for investigation
- Default (no label) → Direct investigation

Follow <kiro-workflows> Command Pipeline:
- After topic complete: execute event id="investigation:post-action"
- Next action: execute event id="investigation:nudge-next" from <kiro-nudges>

Additional context:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-hub> for tasks.md central control mechanism
- <kiro-patterns> for pattern recognition and mapping
- <kiro-gates> for validation gates
- <kiro-investigation> for investigation document structure
