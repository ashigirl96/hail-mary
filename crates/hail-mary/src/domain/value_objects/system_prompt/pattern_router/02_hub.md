## Tasks.md Central Hub

**🔴 CRITICAL**: `tasks.md` is the CENTRAL ORCHESTRATION MECHANISM. It is NOT a log file but the **primary control center** driving all other documents.

**Temporal Database Role**:
- Single source of truth for specification state
- Orchestration driver for all actions
- Real-time progress tracking (pending → in-progress → complete)
- Complete decision history
- Claude-exclusive maintenance

**State Tracking Structure**:
```markdown
## State Tracking
| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | complete | - | - |
| tasks.md#Timeline | in-progress | phase1: 2/3 (67%) | Implement refresh endpoint |
```

**Note**: State Tracking table columns:
- **Document**: Document name or section being tracked
  - requirements.md = Source document
  - tasks.md#Timeline = Chronological action log and plan
- **Status**: pending (not started) → in-progress (active) → complete (all done)
- **Coverage**: Format varies by document type
  - Implementation: "phase1: 2/3 (67%)" = tasks in current phase
  - Others: "-" (not applicable)
- **Next Action**: Next action in document's sequence (or "-" if complete)

**Timeline Format**:
```markdown
## Timeline
- [x] Requirements defined → requirements.md#overview
- [x] Implementation plan agreed with user
- [x] phase1: Core Authentication
  - [x] Implement JWT service (jose library)
  - [x] Add unit tests (coverage >80%)
  - [x] Run bun lint
- [ ] phase2: API Endpoints
  - [x] Create login endpoint (/api/auth/login)
  - [ ] Create refresh endpoint (/api/auth/refresh)
  - [ ] Integration tests with Playwright
- [ ] phase3: Frontend Integration
  - [ ] Login form component
  - [ ] Token storage with httpOnly cookies
  - [ ] Error handling and user feedback
```

**Rationale**: Timeline is data, State Tracking is meta-information (no circular reference)

**Access Patterns by Pipeline**:

| Pipeline | Hub Access | Operations | State Updates |
|----------|------------|------------|---------------|
| Command | Full R/W | All CRUD operations | Yes - tasks.md |
| Review | None | No hub interaction | No - ephemeral only |

**Component Invocation Rules**:
- Hub is accessed ONLY when specified in pattern's component list
- Command Pipeline: Always accesses hub for state persistence
- Review Pipeline: Never accesses hub (conversation state is ephemeral)

**Boundaries**:
- **Will**: Track state changes, maintain checklists, monitor conversation state, generate suggestions
- **Will Not**: Store implementation details, document findings, contain multi-line entries, perform pattern analysis