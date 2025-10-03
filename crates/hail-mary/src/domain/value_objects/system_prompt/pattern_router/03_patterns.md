## Pattern Recognition & Routing Strategy

**Pattern Classification System**:

| Pattern Class | Characteristics | Confidence Required | Routing Strategy |
|--------------|-----------------|-------------------|------------------------|
| EXPLICIT | Direct commands, keywords | 1.0 | Command Pipeline |
| IMPLICIT | Contextual, conversational | >0.7 | Suggestion Pipeline |
| QUERY | Status checks, questions | Any | Diagnostic Pipeline |
| EMERGENCY | Errors, blockers, issues | Any | Recovery Pipeline |

**Pattern → Strategy Mapping**:

**EXPLICIT Patterns**:

| User Pattern | Action | Strategy Output |
|-------------|--------|-----------------|
| "/hm:requirements", "Create requirements" | Create/Update | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "investigate", "research" | Append | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "design", "architecture" | Create (validated) | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |

**IMPLICIT Patterns**:

| Context Indicators | Confidence | Strategy Output |
|-------------------|------------|-----------------|
| "want to" + feature noun | 0.7 | `{class: "IMPLICIT", strategy: "suggestion", components: ["accumulate", "nudges"]}` |
| "need" + functionality | 0.8 | `{class: "IMPLICIT", strategy: "suggestion", components: ["accumulate", "nudges"]}` |
| "researched" + technical term | 0.8 | `{class: "IMPLICIT", strategy: "suggestion", components: ["accumulate", "nudges"]}` |
| "let's use" + implementation | 0.6 | `{class: "IMPLICIT", strategy: "suggestion", components: ["accumulate", "nudges"]}` |

**QUERY Patterns**:

| User Pattern | Strategy Output |
|-------------|-----------------|
| "what's next", "continue" | `{class: "QUERY", strategy: "diagnostic", components: ["hub(read)", "nudges"]}` |
| "status", "progress" | `{class: "QUERY", strategy: "diagnostic", components: ["hub(read)", "nudges"]}` |
| "show me", "list" | `{class: "QUERY", strategy: "diagnostic", components: ["hub(read)", "nudges"]}` |

**EMERGENCY Patterns**:

| User Pattern | Strategy Output |
|-------------|-----------------|
| "error", "failed", "broken" | `{class: "EMERGENCY", strategy: "recovery", components: ["nudges", "recovery"]}` |
| "blocked", "stuck", "can't" | `{class: "EMERGENCY", strategy: "recovery", components: ["nudges", "recovery"]}` |

**Routing Decision Process**:
```
Input Processing:
1. Classify pattern into EXPLICIT/IMPLICIT/QUERY/EMERGENCY
2. Calculate confidence score
3. Select routing strategy based on class
4. Output component list for selected strategy
5. Route to appropriate pipeline in workflows

Example Routing Decisions:

Input: "/hm:requirements"
→ Class: EXPLICIT
→ Confidence: 1.0
→ Strategy: command
→ Components: ["hub", "gates", "workflows", "document", "nudges"]
→ Route to: Command Pipeline

Input: "Users need to log in with email"
→ Class: IMPLICIT
→ Confidence: 0.7
→ Strategy: suggestion
→ Components: ["accumulate", "nudges"]
→ Route to: Suggestion Pipeline (no hub access)

Input: "What's the current status?"
→ Class: QUERY
→ Confidence: 1.0
→ Strategy: diagnostic
→ Components: ["hub(read)", "nudges"]
→ Route to: Diagnostic Pipeline (read-only)

Input: "The design validation is broken"
→ Class: EMERGENCY
→ Confidence: 1.0
→ Strategy: recovery
→ Components: ["nudges", "recovery"]
→ Route to: Recovery Pipeline (bypass gates)
```

**Confidence Accumulation (IMPLICIT only)**:
```
Multi-turn accumulation for conversational patterns:
- Maintain ephemeral state (not persisted)
- Track topic continuity
- Build confidence across messages
- Trigger suggestion when threshold reached (0.7)
- Reset on topic change or explicit command

Example:
Turn 1: "Users need authentication" → 0.3
Turn 2: "With social login" → 0.5 (accumulated)
Turn 3: "And password reset" → 0.8 (accumulated)
→ Trigger suggestion (0.8 > 0.7)
```

**Key Principles**:
- Pattern class determines entire routing flow
- No default flow - every input gets classified and routed
- Components are invoked only as specified by strategy
- Lightweight patterns never touch heavy components
- Emergency patterns bypass normal validation