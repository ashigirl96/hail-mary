## Pattern Recognition & Routing Strategy

**Pattern Classification System**:

| Pattern Class | Characteristics | Confidence Required | Routing Strategy |
|--------------|-----------------|-------------------|------------------------|
| EXPLICIT | Direct commands, keywords | 1.0 | Command Pipeline |
| EXPLICIT_REVIEW | EXPLICIT + --review flag | 1.0 | Review Pipeline |

**Pattern → Strategy Mapping**:

**EXPLICIT Patterns**:

| User Pattern | Action | Strategy Output |
|-------------|--------|-----------------|
| "/spec:requirements", "Create requirements" | Create/Update | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "/spec:investigate", "investigate", "research" | Append | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "/spec:design", "design", "architecture" | Create (validated) | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "/spec:timeline", "plan implementation" | Timeline Planning | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |

**EXPLICIT_REVIEW Patterns**:

| User Pattern | Action | Strategy Output |
|-------------|--------|-----------------|
| "/spec:requirements --review" | Review then Create | `{class: "EXPLICIT_REVIEW", strategy: "review", components: ["patterns", "review", "nudges"]}` |
| "/spec:design --review" | Review then Create | `{class: "EXPLICIT_REVIEW", strategy: "review", components: ["patterns", "review", "nudges"]}` |

**Routing Decision Process**:
```
Input Processing:
1. Classify pattern into EXPLICIT or EXPLICIT_REVIEW
2. Calculate confidence score
3. Select routing strategy based on class
4. Output component list for selected strategy
5. Route to appropriate pipeline in workflows

Example Routing Decisions:

Input: "/spec:requirements"
→ Class: EXPLICIT
→ Confidence: 1.0
→ Strategy: command
→ Components: ["hub", "gates", "workflows", "document", "nudges"]
→ Route to: Command Pipeline

Input: "/spec:requirements --review"
→ Class: EXPLICIT_REVIEW
→ Confidence: 1.0
→ Strategy: review
→ Components: ["patterns", "review", "nudges"]
→ Route to: Review Pipeline

Input: "/spec:investigate --topic jwt-implementation"
→ Class: EXPLICIT
→ Confidence: 1.0
→ Strategy: command
→ Components: ["hub", "gates", "workflows", "document", "nudges"]
→ Route to: Command Pipeline

Input: "/spec:design"
→ Class: EXPLICIT
→ Confidence: 1.0
→ Strategy: command
→ Components: ["hub", "gates", "workflows", "document", "nudges"]}
→ Route to: Command Pipeline

Input: "/spec:design --review"
→ Class: EXPLICIT_REVIEW
→ Confidence: 1.0
→ Strategy: review
→ Components: ["patterns", "review", "nudges"]
→ Route to: Review Pipeline

Input: "/spec:timeline"
→ Class: EXPLICIT
→ Confidence: 1.0
→ Strategy: command
→ Components: ["hub", "gates", "workflows", "document", "nudges"]}
→ Route to: Command Pipeline
```

**Key Principles**:
- Pattern class determines entire routing flow
- No default flow - every input gets classified and routed
- Components are invoked only as specified by strategy
- --review flag modifies EXPLICIT to EXPLICIT_REVIEW
