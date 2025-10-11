---
name: investigate
description: "Technical investigation for Kiro specs - triggered by: investigate, research, verify, explore, 調査, 検証"
argument-hint: "[--topic <name>] [--deep]"
---

# /spec:investigate

Conduct technical research and append findings with reactive pattern-based routing.

## Depth Control

Pattern detection:
- `--deep` present → Route to deep-dive-analyst agent (invoked per topic as needed)
  - Shared context: <kiro-spec-files> (requirements.md, investigation.md, design.md, spec metadata)
  - Produces comprehensive analysis with Evidence Chain format
- No flag → Standard investigation (quick findings with append-only format)

## System Prompt References

Refer to system prompt sections:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-hub> for tasks.md central control mechanism
- <kiro-patterns> for pattern recognition and mapping
- <kiro-workflows> After topic complete: `investigation:post-action`
- <kiro-gates> for validation gates
- <kiro-nudges> Next action suggestion: `investigation:nudge-next`
- <kiro-investigation> for investigation document structure

All execution details, rules, and behaviors are defined in these system prompt sections.