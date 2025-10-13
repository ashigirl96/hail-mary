---
name: design
description: "Technical design from requirements - triggered by: design, architecture, implementation, 設計, アーキテクチャ"
argument-hint: "[--review]"
---

# /spec:design

Design implementation to fulfill <requirements-file> using <investigation-file> evidence, following reactive pattern-based routing.

Follow <kiro-workflows> Command Pipeline:
- After design complete: execute event id="design:post-action"
- Next action: execute event id="design:nudge-next" from <kiro-nudges>

Additional context:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-hub> for tasks.md central control mechanism
- <kiro-patterns> for pattern recognition and mapping
- <kiro-gates> for validation gates and prerequisites
- <kiro-design> for design document structure