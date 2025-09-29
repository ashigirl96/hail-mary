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
| requirements.md | complete | - | Define investigation topics |
| investigation.md | in-progress | 3/5 (60%) | Complete remaining topics |
| design.md | pending | - | Awaiting 100% coverage |
```

**Required Investigations Checklist**:
```markdown
## Required Investigations
- [x] jwt-implementation → investigation.md#jwt-implementation
- [x] database-schema → investigation.md#database-schema
- [ ] session-management
- [ ] password-reset-flow
- [ ] security-best-practices
```

**Timeline Format**:
```markdown
## Timeline
- [x] Requirements defined → requirements.md#overview
- [x] JWT implementation investigated → investigation.md#jwt-implementation
- [ ] Design authentication flow
  - blocked by: investigations incomplete (2/4)
```

**Access Patterns by Pipeline**:

| Pipeline | Hub Access | Operations | State Updates |
|----------|------------|------------|---------------|
| Command | Full R/W | All CRUD operations | Yes - tasks.md |
| Suggestion | None | No hub interaction | No - ephemeral only |
| Diagnostic | Read-only | Query state only | No |
| Recovery | Minimal | Emergency context only | No |

**Component Invocation Rules**:
- Hub is accessed ONLY when specified in pattern's component list
- Command Pipeline: Always accesses hub for state persistence
- Suggestion Pipeline: Never accesses hub (conversation state is ephemeral)
- Diagnostic Pipeline: Read-only access for status reporting
- Recovery Pipeline: Optional minimal access for context

**Boundaries**:
- **Will**: Track state changes, maintain checklists, monitor conversation state, generate suggestions
- **Will Not**: Store implementation details, document findings, contain multi-line entries, perform pattern analysis