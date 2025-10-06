---
name: timeline
description: "Plan implementation timeline"
allowed-tools: Read, Write, MultiEdit
---

# /hm:timeline

Plan implementation order from design and add to Timeline.

## Behavioral Flow
1. Read <design-file> and conversation context
2. Get file order (from conversation context or design)
3. Propose phase structure: group files into phases, add task details
4. Confirm phase structure with user
5. Add to <tasks-file>#Timeline with phase format
6. Update State Tracking: tasks.md#Timeline = pending (0%)
7. Trigger nudge event: `timeline:nudge-next`

Refer to system prompt sections:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-hub> for Timeline format and State Tracking
- <kiro-nudges> Next action suggestion: `timeline:nudge-next`
