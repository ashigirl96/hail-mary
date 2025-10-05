## Design Document Structure

**Boundaries**:
- **Will**: Provide template, ensure evidence-based design
- **Will Not**: Validate prerequisites, check completion status

**Design Template**:
````markdown
## Meta
- **Completeness**: [0-100%]
- **Requirements**: [Brief summary]
- **Architecture Scope**: [Backend/Frontend/Full-stack]

## Overview
**As-Is**: [Current state]
**To-Be**: [Target state]

## Design
[Comprehensive changes description]

### [file-path-1]
[Purpose (requirements.md#section), investigation findings (investigation.md#section), solution approach]

```language
// Complete implementation
```

### [file-path-2] (New File)
[Purpose, investigation basis, complete implementation]

```language
// Full code
```
````

**Evidence Requirements**:
- Every decision → investigation.md#section
- Every component → requirements.md#requirement
- No placeholders or TODOs
- Production-ready implementations

**Key Behaviors**:
- Complete implementation focus
- State documentation (As-Is/To-Be)
- Code quality standards
- Requirements traceability

**Example Section**:
````markdown
### src/auth/jwt-service.ts (New File)
JWT service based on investigation (investigation.md#jwt-implementation).
Implements requirement for token-based auth (requirements.md#authentication).

```typescript
import { SignJWT, jwtVerify } from 'jose';
// Complete implementation...
```
````

**Interactive Confirmation**:
Show summary and ask: "Save to design.md?"