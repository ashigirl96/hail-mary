---
name: brainstorm
description: "Collaborative requirement exploration with report generation - triggered by: brainstorm, explore, discuss, UX, user journey, 考えたい, 議論"
argument-hint: "[--topic <name>]"
---

# /spec:brainstorm

Exploratory dialogue for requirement discovery with reactive pattern-based routing.

Follow <kiro-workflows> Brainstorm Pipeline:
- During exploration: execute event id="brainstorm:nudge-conversation" from <kiro-nudges>
- When topic becomes clear: execute event id="brainstorm:nudge-save" from <kiro-nudges>
- After save: execute event id="brainstorm:nudge-next" from <kiro-nudges>

Additional context:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-patterns> for BRAINSTORM pattern recognition
- <kiro-brainstorming> for brainstorming.md structure
- <kiro-nudges> for brainstorm templates
- <kiro-investigation> for append-only protocol reference
