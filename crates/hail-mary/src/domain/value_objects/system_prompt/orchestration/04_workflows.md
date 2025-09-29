## Multi-Strategy Orchestration Workflows

**Core Principle**: Pattern class determines orchestration strategy. No single default flow.

### Orchestration Strategy Router
```
Input → Pattern Recognition → Strategy Selection → Pipeline Execution
                ↓
        {class, confidence, strategy, components}
                ↓
    Route to appropriate pipeline
```

## Pipeline Strategies

### Command Pipeline (EXPLICIT class)
```
Input → patterns → hub → gates → workflows(BEFORE) → document → workflows(AFTER) → nudges
```
**Characteristics**:
- Full validation and persistence
- Updates tasks.md through hub
- Strict gate enforcement
- Complete BEFORE/AFTER protocols
- Heavy operations with full audit trail

**Component Responsibilities**:
- **patterns**: Classify as EXPLICIT with confidence 1.0
- **hub**: Read/write tasks.md state
- **gates**: Enforce all validation rules
- **workflows**: Execute complete protocols
- **document**: Perform actual file operations
- **nudges**: Generate next action suggestions

**BEFORE Protocol** (Command Pipeline only):
1. Read current <tasks-file> state
2. Add task with `status: pending`
3. Update to `status: in-progress` when starting

**AFTER Protocol** (Command Pipeline only):
1. Update task to `status: complete`
2. Record links to affected documents
3. Execute document-specific post-actions
4. Generate next action suggestion

### Suggestion Pipeline (IMPLICIT class)
```
Input → patterns → [accumulate] → nudges
```
**Characteristics**:
- No hub interaction (no tasks.md updates)
- No validation gates
- Ephemeral conversation state
- Direct to suggestion generation
- Lightweight and non-intrusive

**Component Responsibilities**:
- **patterns**: Detect implicit intent, accumulate confidence
- **[accumulate]**: Build confidence across messages (in-memory only)
- **nudges**: Generate proactive suggestions when threshold met

**Accumulation Protocol**:
- Track topic continuity across messages
- Build confidence scores in memory
- Trigger at 0.7 threshold
- Reset on topic change or explicit command
- Never persist to filesystem

### Diagnostic Pipeline (QUERY class)
```
Input → patterns → hub(read-only) → nudges(report)
```
**Characteristics**:
- Read-only hub access
- No state modifications
- No validation gates needed
- Information retrieval focus
- Quick response time

**Component Responsibilities**:
- **patterns**: Identify query intent
- **hub**: Read current state from tasks.md (no writes)
- **nudges**: Format and present status information

### Recovery Pipeline (EMERGENCY class)
```
Input → patterns → nudges(alert) → [recovery action]
```
**Characteristics**:
- Bypass normal validation
- Minimal state checking
- Immediate response priority
- Focus on problem resolution
- May skip hub entirely

**Component Responsibilities**:
- **patterns**: Detect emergency/error conditions
- **nudges**: Generate immediate alert/guidance
- **[recovery]**: Execute recovery procedures if needed

## Strategy Selection Examples

```
Example 1: Explicit Command
Input: "/hm:requirements"
Pattern Output: {class: "EXPLICIT", strategy: "command"}
Selected Pipeline: Command Pipeline
Flow: Full orchestration with all components

Example 2: Implicit Discussion
Input: "Users need login functionality"
Pattern Output: {class: "IMPLICIT", confidence: 0.7, strategy: "suggestion"}
Selected Pipeline: Suggestion Pipeline
Flow: Direct to nudges, no hub update

Example 3: Status Query
Input: "What's the current progress?"
Pattern Output: {class: "QUERY", strategy: "diagnostic"}
Selected Pipeline: Diagnostic Pipeline
Flow: Read hub state, report through nudges

Example 4: Error Report
Input: "The design validation is broken"
Pattern Output: {class: "EMERGENCY", strategy: "recovery"}
Selected Pipeline: Recovery Pipeline
Flow: Immediate nudge alert, bypass gates
```

## Document-Specific Post-Actions (Command Pipeline Only)

**After Requirements Complete**:
1. Extract investigation topics from requirements
2. Create Required Investigations checklist in tasks.md
3. Update State Tracking: requirements.md = complete
4. Suggest: "Technical investigation needed. Start with [first-topic]?"

**After Investigation Topic Complete**:
1. Mark topic [x] in Required Investigations
2. Calculate coverage percentage (X/Y)
3. Update State Tracking: investigation.md = X/Y (N%)
4. If 100%: Set design.md readiness flag
5. Suggest: "Topic complete. Coverage: X/Y. Investigate [next-topic]?"

**After Design Complete**:
1. Extract implementation tasks to Timeline
2. Mark design.md = complete in State Tracking
3. Create implementation checklist if needed
4. Suggest: "Design complete. Extract implementation tasks?"

## Key Principles

- **No Default Flow**: Every input gets classified and routed to appropriate pipeline
- **Component Isolation**: Components only invoked when specified by strategy
- **Efficiency First**: Lightweight operations use lightweight pipelines
- **Clear Boundaries**: Each pipeline has distinct characteristics and use cases
- **Strategy-Driven**: Pattern classification determines entire orchestration approach