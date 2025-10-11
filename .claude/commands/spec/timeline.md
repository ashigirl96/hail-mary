---
name: timeline
description: "Plan implementation timeline"
allowed-tools: Read, Write, MultiEdit
---

# /spec:timeline

Plan implementation order from design and add to Timeline.

## Behavioral Flow
1. Read <design-file> and conversation context
2. Get file order (from conversation context or design)
3. Propose phase structure: group files into phases, add task details
4. Confirm phase structure with user
5. Add to <tasks-file>#Timeline with phase format
6. Update State Tracking: tasks.md#Timeline = pending (0%)
7. Execute event id="timeline:nudge-next" from <kiro-nudges>

Additional context:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-hub> for Timeline format and State Tracking
- <kiro-gates> for validation gates
