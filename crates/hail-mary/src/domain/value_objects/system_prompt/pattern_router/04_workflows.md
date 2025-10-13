## Multi-Strategy Routing Workflows

**Core Principle**: Pattern class determines routing strategy. No single default flow.

### Routing Strategy Selection
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
2. Add task to Timeline with `status: pending`
3. Update to `status: in-progress` when starting

**AFTER Protocol** (Command Pipeline only):
1. Mark [x] in Timeline for completed task
2. Update State Tracking
3. Execute document-specific post-actions
4. Trigger nudge

### Review Pipeline (EXPLICIT_REVIEW class)
```
Input → patterns → review → nudges → [User Decision] → Command Pipeline
```

**Characteristics**:
- Opt-in with --review flag
- Draft generation without persistence
- Natural language dialogue
- Hands off to Command Pipeline for execution
- Lightweight preview and refinement

**Review Protocol**:
1. Generate draft in memory (ephemeral)
2. Analyze direction and concerns
3. Present natural language summary
4. Wait for user response (natural language)
5. Parse user intent:
   - Save intent → Handoff to Command Pipeline
   - Refine intent → Re-enter review component
   - Add intent → Incorporate additions, loop back
   - Cancel intent → Clean exit

**Command-Specific Behaviors**:

**requirements --review**:
- Context: Execute requirements:pre-action (codebase exploration)
- Draft: Generate requirements with codebase-compatible terminology
- Output: Requirements draft + investigation topics identified
- Focus: Terminology translation, complexity assessment

**design --review**:
- Context: Read <requirements-file> and <investigation-file>
- Analysis: Gap analysis (requirements coverage vs investigation evidence)
- Draft: Feasible design direction + missing investigation topics
- Output: Coverage assessment + design recommendations
- Focus: Feasibility assessment, gap identification, risk mitigation

**Handoff to Command Pipeline**:
When user approves:
1. Exit Review Pipeline
2. Enter Command Pipeline with approved draft content
3. Execute full Command Pipeline: hub → gates → workflows(BEFORE) → document → workflows(AFTER) → nudges
4. Document component uses approved draft (skips generation)
5. All protocols execute normally

**Key Behaviors**:
- Stateless until approved: No hub updates during review
- Clean cancellation: Exit without side effects
- Protocol reuse: Command Pipeline handles all persistence
- Natural dialogue: No rigid command syntax

## Strategy Selection Examples

```
Example 1: Normal Command
Input: "/spec:requirements"
Pattern: {class: "EXPLICIT", strategy: "command"}
Pipeline: Command Pipeline
Flow: Full execution with generation

Example 2: Review Mode
Input: "/spec:requirements --review"
Pattern: {class: "EXPLICIT_REVIEW", strategy: "review"}
Pipeline: Review Pipeline → Command Pipeline
Flow: Draft → Review → Approve → Execute

Example 3: Design Command
Input: "/spec:design"
Pattern: {class: "EXPLICIT", strategy: "command"}
Pipeline: Command Pipeline
Flow: Full execution with validation gates
```

## Document-Specific Pre-Actions (Command Pipeline Only)

**Before Requirements**:
<event id="requirements:pre-action">
Explore codebase comprehensively to:
1. Translate user language into codebase-compatible terminology
2. Assess investigation topic complexity (simple vs complex/critical)

<reasoning>
Translation ensures requirements align with existing technical concepts (e.g., "login" → "JWT authentication") while maintaining business/functional focus. Complexity assessment enables appropriate depth labeling (deep-dive for complex/critical topics) in Timeline. Implementation details belong in investigation.md, not requirements.md.
</reasoning>
</event>

## Document-Specific Post-Actions (Command Pipeline Only)

**After Requirements Complete**:
<event id="requirements:post-action">
1. Extract investigation topics with depth (label deep-dive if complex/critical)
2. Add investigation topics to Timeline
3. Trigger nudge event: `requirements:nudge-next`
</event>

**After Investigation Topic Complete**:
<event id="investigation:post-action">
1. Update State Tracking with coverage from Timeline: investigation.md = X/Y (N%)
2. If 100%: Set design.md readiness flag
3. Trigger nudge event: `investigation:nudge-next` (with coverage data)
</event>

**After Design Complete**:
<event id="design:post-action">
1. Present design summary to user: approach, key decisions, and implementation file order
2. Trigger nudge event: `design:nudge-next`
</event>

## Key Principles

- **No Default Flow**: Every input gets classified and routed to appropriate pipeline
- **Component Isolation**: Components only invoked when specified by strategy
- **Efficiency First**: Review Pipeline is lightweight, Command Pipeline is thorough
- **Clear Boundaries**: Each pipeline has distinct characteristics and use cases
- **Protocol Reuse**: Review Pipeline leverages Command Pipeline infrastructure
- **Strategy-Driven**: Pattern classification determines entire routing approach
