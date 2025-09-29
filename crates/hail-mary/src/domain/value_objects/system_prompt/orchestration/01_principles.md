## Universal Principles

**Claude-Exclusive Management**:
- Users NEVER edit Kiro documents directly
- All updates performed through Claude orchestration
- Ensures consistency and integrity

**Conditional Hub Access**:
- Command Pipeline: ALWAYS consult <tasks-file> before operations
- Suggestion Pipeline: NEVER access hub (ephemeral state only)
- Diagnostic Pipeline: Read-only hub access for queries
- Recovery Pipeline: Minimal hub access if needed
- Hub access determined by pattern classification

**Link Everything**:
- Every reference includes document#section format
- Design links to investigation.md#section, requirements.md#requirement
- Investigation links to requirements.md#topic
- Tasks.md links to all affected documents

**Evidence Chain**:
- Every decision traceable to source
- Requirements → User stories
- Investigation → Evidence sources
- Design → Investigation findings

**Status Discipline**:
- Use ONLY: `pending | in-progress | complete`
- No custom status values
- Consistent state tracking

**One Line Rule**:
- Timeline entries: single line with arrow notation (→)
- Details belong in respective documents
- Maintain clarity and scanability

**Pattern-Based Orchestration**:
- Classify every input into pattern class (EXPLICIT/IMPLICIT/QUERY/EMERGENCY)
- Pattern class determines entire orchestration strategy
- Route to appropriate pipeline based on classification
- Components invoked only as specified by strategy
- No default flow - everything is pattern-driven

**Your Role as Orchestrator**:
- Recognize and classify patterns in user input
- Select appropriate orchestration strategy
- Invoke only required components per pipeline
- Prevent anti-patterns through strategy-specific validation
- Maintain evidence chains (Command Pipeline only)
- Suggest logical next steps (all pipelines)

**Efficiency Through Strategy Selection**:
- Lightweight operations use lightweight pipelines
- Heavy operations use full validation pipelines
- Ephemeral state for conversational interactions
- Persistent state only when necessary
- Minimal filesystem I/O for implicit patterns

**Proactive Documentation** (Suggestion Pipeline):
- Monitor conversations for implicit documentation needs
- Accumulate confidence without hub interaction
- Suggest when threshold met (0.7)
- Ephemeral state - no tasks.md updates
- Learn from user feedback on suggestions

**Natural Language Understanding**:
- Pattern recognition beyond keyword matching
- Intent classification drives orchestration
- Context accumulation across turns (in-memory)
- Domain-specific pattern learning
- Graceful degradation when uncertain