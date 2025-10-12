---
name: timeline
description: "Plan implementation timeline"
allowed-tools: Read, Write, MultiEdit
---

# /spec:timeline

Plan implementation order from design and add to Timeline.

## Key Patterns

**File Ordering Priority:**

1. **Test Pairing**: Implementation → Test (same phase)
   - Immediate feedback loop, early bug detection
   - Example: `auth.rs` + `auth_test.rs` in phase1

2. **Layer Ordering**: Backend → Frontend
   - API contract established first
   - Enables parallel development after backend stable
   - Example: phase1-3 (Backend) → phase4 (Frontend)

3. **Backend Dependencies**: Clean Architecture layers (inner → outer)
   - Domain → Application → Infrastructure → CLI
   - Satisfies compile-time dependencies
   - Example: entities → use_cases → repositories → commands

4. **Frontend Dependencies**: Reusable logic → UI
   - React: hook → component
   - Shared components → Specialized components
   - Example: useAuth.ts → LoginForm.tsx

5. **Test Strategy**:
   - Unit tests: Same phase as implementation
   - Integration tests: After dependent layers complete
   - E2E tests: Final phase (all layers ready)

6. **Quality Checks**: Each phase ends with validation
   - Formatter: Auto-format code (cargo fmt, prettier, etc.)
   - Linter: Check code quality (clippy, eslint, etc.)
   - Type checker: Verify type safety (cargo check, tsc, etc.)
   - Tests: Run unit tests for implemented files
   - Example phase ending: "Run cargo fmt && cargo clippy && cargo test domain::"

**Special Cases:**
- Shared utilities: Before dependent files (or phase0 if extensive)
- Database migrations: Before repository implementations
- Config files: Phase0 or earliest dependent phase
- Full-stack routes: API route → Frontend page

## Behavioral Flow
1. Read <design-file> and conversation context
2. Determine file order:
   - Extract files from design or conversation
   - Apply File Ordering Priority rules (see Key Patterns above)
   - Group into phases based on dependencies
   - Verify no circular dependencies
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
