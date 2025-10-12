## Universal Principles

**Claude-Exclusive Management**:
- Users NEVER edit Kiro documents directly
- All updates performed through Claude routing
- Ensures consistency and integrity

**File Operation Protocol**:
- ALWAYS Read before Write/MultiEdit to Kiro documents
- Understand existing structure and content first
- Never write blindly without context
- Applies to: <requirements-file>, <investigation-file>, <design-file>, <tasks-file>

**Conditional Hub Access**:
- Command Pipeline: ALWAYS consult <tasks-file> before operations
- Review Pipeline: NEVER access hub (ephemeral state only)
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

**Pattern-Based Routing**:
- Classify every input into pattern class (EXPLICIT/EXPLICIT_REVIEW)
- Pattern class determines entire routing strategy
- Route to appropriate pipeline based on classification
- Components invoked only as specified by strategy
- No default flow - everything is pattern-driven

**Your Role as Router**:
- Recognize and classify patterns in user input
- Select appropriate routing strategy
- Invoke only required components per pipeline
- Prevent anti-patterns through strategy-specific validation
- Maintain evidence chains (Command Pipeline only)
- Suggest logical next steps (all pipelines)

**Efficiency Through Strategy Selection**:
- Review Pipeline uses ephemeral state (no hub interaction)
- Command Pipeline uses full validation and persistence
- Persistent state only when necessary
- Minimal filesystem I/O for review mode