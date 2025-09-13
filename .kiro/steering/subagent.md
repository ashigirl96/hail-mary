# Subagent Patterns and Best Practices

## YAML Front Matter Structure
**When**: Creating new subagents for Claude Code
- Use 3-field YAML with name, description, category
- description: 簡潔な核心能力の要約（action-oriented）
- category: engineering|quality|analysis|communication|specialized
- tools field is optional (omit to inherit all tools)

```yaml
# ✅ Good
---
name: code-reviewer
description: Expert code review specialist. Proactively reviews code for quality, security, and maintainability.
category: quality
---

# ❌ Bad
---
name: CodeReviewer
desc: Reviews code
type: reviewer
---
```

## 7-Section Template
**When**: Structuring subagent content
1. `# Agent Name` (H1 header)
2. `## Triggers` - Specific activation conditions
3. `## Behavioral Mindset` - Core philosophy
4. `## Focus Areas` - 4-5 domain bullets with sub-details
5. `## Key Actions` - 5 numbered systematic steps
6. `## Outputs` - 5 specific deliverables
7. `## Boundaries` - Will/Will Not definitions

## Writing Style
**When**: Writing agent instructions
- Use imperative commands: "Think", "Focus", "Prioritize"
- Professional certainty without hedging
- Action-oriented: every statement drives to outcomes
- Domain-specific technical terminology

```markdown
# ✅ Good
Think systematically about root causes.
Prioritize security over convenience.
Focus on measurable outcomes.

# ❌ Bad
Maybe consider looking at causes.
You might want to check security.
Try to get some results.
```

## Priority Hierarchies
**When**: Defining agent decision-making
- Establish explicit priority chains
- Use > symbol for clear ranking
- Make trade-offs transparent

```markdown
# ✅ Good
Security > compliance > reliability > performance > convenience
User needs > accessibility > performance > technical elegance

# ❌ Bad
Balance security and performance
Consider all factors equally
```

## Measurable Standards
**When**: Setting quality criteria
- Include concrete metrics (95%, <200ms, WCAG 2.1 AA)
- Reference industry standards (OWASP, SOLID)
- Define validation checkpoints

```markdown
# ✅ Good
- Test coverage: 95%+ required
- API response: <200ms target
- Compliance: WCAG 2.1 AA minimum

# ❌ Bad
- Good test coverage
- Fast response times
- Accessible design
```

## Section Architecture
**When**: Organizing subagent content structure
- Triggers: Specific activation conditions with keywords
- Behavioral Mindset: Core philosophy and thinking approach
- Focus Areas: 4-5 domain bullets with hierarchical sub-details
- Key Actions: 5-step systematic process (Analysis→Design→Implementation→Validation→Documentation)
- Outputs: 5 concrete deliverables with descriptions
- Boundaries: Will/Will Not explicit capability definitions

```markdown
# ✅ Good Structure
## Triggers
- Keywords: "analyze", "investigate", "root cause"
- Debugging or troubleshooting sessions
- Systematic investigation requests

## Key Actions
1. Analysis/Assessment - Understand the situation
2. Design/Planning - Create structured approach
3. Implementation - Execute with specific techniques
4. Validation - Verify outcomes meet standards
5. Documentation - Record decisions and results

# ❌ Bad Structure
## When to Use
Sometimes when you need analysis

## What to Do
- Do some analysis
- Fix problems
- Document stuff
```

## Domain Identity
**When**: Establishing agent expertise and role
- Define domain ownership ("Backend systems", "Security vulnerabilities")
- Include philosophical stance ("Security-first mindset", "User-first decisions")
- Establish non-negotiable core principles
- Use role-based identity with professional context

```markdown
# ✅ Good
Identity: Systems architecture specialist, long-term thinking focus
Philosophy: Evidence over assumptions, scalability expert

# ❌ Bad
I help with architecture
General problem solver
```

## Evidence Philosophy
**When**: Defining problem-solving approach
- Data-driven decisions always
- "Profile before optimizing", "Evidence over assumptions"
- Systematic investigation with structured methods
- Validation requirements before claiming success

## Cross-Agent Boundaries
**When**: Defining scope and integration
- Acknowledge other agents' territories explicitly
- Clear handoff mechanisms between specialists
- Recognition of multi-agent collaboration
- Prevent overlap through explicit boundaries

```markdown
# ✅ Good
Will Not:
- Make security decisions (defer to security-engineer)
- Design UI components (defer to frontend-architect)

# ❌ Bad
Will handle everything related to the system
```

## Identity Statement Pattern
**When**: Beginning every subagent definition
- Start with clear identity declaration before any other content
- Include role, specialization, and expertise focus
- Establishes agent's self-awareness and purpose
- Place immediately after H1 header

```markdown
# ✅ Good
# Backend Architect

**Identity**: Systems architecture specialist, long-term thinking focus, scalability expert

# ❌ Bad
# Backend Architect

Let me help you with backend tasks...
```

## Priority Hierarchy Declaration
**When**: Establishing decision-making frameworks
- Define explicit trade-off priorities using > symbol
- Place in Behavioral Mindset or as separate section
- Guides consistent decision-making under constraints
- Makes agent reasoning transparent

```markdown
# ✅ Good
**Priority Hierarchy**: Reliability > security > performance > features > convenience

# ❌ Bad
We balance all concerns equally
```

## Performance Budgets & Metrics
**When**: Setting measurable success criteria
- Include specific numeric targets and thresholds
- Reference industry standards and benchmarks
- Define both minimum requirements and aspirational goals
- Enable objective performance validation

```markdown
# ✅ Good
**Performance Budgets**:
- Load Time: <3s on 3G, <1s on WiFi
- Bundle Size: <500KB initial, <2MB total
- API Response: <200ms p95
- Error Rate: <0.1% for critical operations

# ❌ Bad
- Fast loading times
- Small bundle sizes
- Quick API responses
```

## Knowledge Domain Embedding
**When**: Referencing authoritative sources
- Embed specific book knowledge and frameworks
- Reference recognized industry authorities
- Include concrete methodologies and principles
- Provide intellectual foundation for decisions

```markdown
# ✅ Good
### Clean Code Principles (Robert C. Martin)
- Meaningful Names: intention-revealing, searchable
- Single Responsibility: one reason to change
- DRY: Don't Repeat Yourself

# ❌ Bad
Follow general best practices
Use clean coding standards
```

## Evidence Chain Documentation
**When**: Systematic investigation and analysis
- Document hypothesis → testing → validation flow
- Preserve evidence trail for conclusions
- Include contradictory evidence handling
- Enable reproducible reasoning

```markdown
# ✅ Good
## Investigation Methodology
1. **Evidence Collection**: Gather logs, metrics, system state
2. **Hypothesis Formation**: Develop multiple theories
3. **Systematic Testing**: Validate each hypothesis
4. **Evidence Chain**: Document path from symptoms to cause
5. **Conclusion Validation**: Verify with reproducible tests

# ❌ Bad
Find the problem and fix it
```

## Multi-Persona Collaboration
**When**: Complex scenarios requiring multiple expertise
- Define persona panels for collective intelligence
- Specify interaction modes (Discussion/Debate/Socratic)
- Include handoff mechanisms between agents
- Enable specialized expertise combination

```markdown
# ✅ Good
### Expert Panel Configuration
- **Primary**: Systems Architect (design lead)
- **Consulting**: Security Engineer (threat analysis)
- **Validation**: Performance Engineer (metrics validation)
- **Handoff**: Architect → Security for auth design

# ❌ Bad
Multiple agents work together
```

## Session Memory & Progress Tracking
**When**: Learning and adaptation requirements
- Track discovery progress and mastery levels
- Maintain cross-session continuity
- Adapt to user patterns and preferences
- Enable progressive skill development

```markdown
# ✅ Good
### Progress Tracking
- Principle Mastery: discovered → applied → mastered
- Session Memory: retained discoveries and patterns
- User Model: learning style and pace adaptation
- Continuity: resume from previous session state

# ❌ Bad
Remember what was discussed
```

## MCP Server Coordination
**When**: Integrating with external tools
- Specify primary and secondary MCP servers
- Define coordination patterns between servers
- Include fallback strategies
- Document server-specific workflows

```markdown
# ✅ Good
### MCP Integration
**Primary**: Sequential - systematic analysis
**Secondary**: Context7 - pattern validation
**Workflow**: Sequential analyzes → Context7 validates → Apply patterns

# ❌ Bad
Use MCP servers as needed
```

## Question Crafting Strategies
**When**: Educational or discovery-oriented agents
- Design question progressions for learning
- Adapt to user knowledge levels
- Guide discovery without revealing answers
- Build understanding incrementally

```markdown
# ✅ Good
### Question Progression
1. **Observation**: "What do you notice about...?"
2. **Pattern**: "Why might this be important?"
3. **Principle**: "What rule could explain this?"
4. **Application**: "How would you apply this elsewhere?"

# ❌ Bad
Ask questions to help understanding
```

## Framework Integration Points
**When**: Connecting to larger systems
- Define auto-activation rules and triggers
- Specify command chaining patterns
- Include confidence thresholds
- Document integration with quality gates

```markdown
# ✅ Good
### Auto-Activation Rules
- **Trigger**: security keywords + risk score >0.7
- **Confidence**: 85% threshold for activation
- **Chain**: /analyze → security-engineer → remediation

# ❌ Bad
Activates when needed
```

## Multi-Hypothesis Testing Protocol
**When**: Managing parallel theories in investigation
- Maintain 3-7 competing hypotheses simultaneously
- Score confidence 0.0-1.0 per hypothesis
- Prioritize falsification over confirmation
- Document elimination rationale for each theory
- Synthesize surviving hypotheses into conclusions

```markdown
# ✅ Good
### Hypothesis Management
- **H1**: Component failure (confidence: 0.7)
- **H2**: Network latency (confidence: 0.4)
- **H3**: Race condition (confidence: 0.8)
**Falsification**: Seeking evidence against H3 first

# ❌ Bad
Investigating one possible cause at a time
Following first plausible explanation
```

## Progressive Complexity Scaling
**When**: Adapting investigation depth to user level
- Assess current knowledge and cognitive load
- Dynamically adjust tool usage and guidance
- Scale from concrete to abstract reasoning
- Maintain appropriate scaffolding levels

```markdown
# ✅ Good
### Adaptive Investigation
**Beginner**: Concrete observations → Guided tools → High support
**Intermediate**: Pattern recognition → Selective tools → Medium support
**Advanced**: Abstract synthesis → Full orchestration → Minimal support

# ❌ Bad
Same investigation depth for all users
No adaptation to cognitive load
```

## Stakeholder Perspective Matrix
**When**: Integrating multiple viewpoints in analysis
- Map primary, secondary, tertiary stakeholders
- Document each perspective independently
- Identify and resolve conflicts explicitly
- Use staged validation rounds

```markdown
# ✅ Good
### Validation Rounds
Round 1: Individual stakeholder verification
Round 2: Cross-validation between groups
Round 3: Integrated consensus validation

# ❌ Bad
Mixing all perspectives together
Ignoring conflicting requirements
```

## System Behavior Mapping
**When**: Measurement-driven investigation
- Establish baseline before investigation
- Map critical paths and dependencies
- Score by impact × frequency
- Validate with before/after metrics

```markdown
# ✅ Good
### Measurement Framework
- **Baseline**: Current state metrics (latency: 450ms)
- **Critical Path**: User login → Dashboard load
- **Impact Score**: 0.8 (high) × 1000/hour = 800
- **Validation**: Post-fix latency: 120ms (-73%)

# ❌ Bad
Investigating without baseline metrics
No quantitative validation
```

## Timeline Reconstruction
**When**: Building event sequence understanding
- Map events chronologically with timestamps
- Correlate across multiple components
- Identify patterns in temporal relationships
- Document causality chains

```markdown
# ✅ Good
### Event Timeline
T-5min: Database connection spike
T-3min: API timeout errors begin
T-1min: Cache invalidation
T+0: System failure
**Pattern**: DB overload → API cascade → failure

# ❌ Bad
Unordered event listing
Missing temporal relationships
```

## Contradiction-Seeking Validation
**When**: Testing hypothesis robustness
- Actively search for disproving evidence
- Document contradictions explicitly
- Weight contradictory vs supporting evidence
- Adjust confidence based on contradictions

```markdown
# ✅ Good
### Contradiction Analysis
**Hypothesis**: Memory leak in service A
**Supporting**: Memory growth over time (3 instances)
**Contradicting**: Service B shows same pattern (2 instances)
**Conclusion**: Systemic issue, not service-specific

# ❌ Bad
Only collecting supporting evidence
Ignoring contradictory data
```

## Investigation Methodology Patterns
**When**: Structuring systematic investigations
- Evidence Chain Protocol: Timeline → Pattern → Hypothesis → Validation
- Falsification Priority: Seek disproving evidence first
- Convergent Analysis: Multiple paths to same conclusion
- Reproducibility Focus: Document for repeatability

```markdown
# ✅ Good
## Investigation Protocol
1. Timeline Reconstruction → Event sequence
2. Pattern Recognition → Cross-correlation
3. Hypothesis Formation → 3+ theories
4. Contradiction Seeking → Active falsification
5. Chain Documentation → Evidence→Pattern→Cause

# ❌ Bad
Ad-hoc investigation approach
No systematic methodology
```

## Special Extensions
**When**: Creating specialized agent types
- Business Panel: Multiple persona specifications with voice characteristics
- Socratic Mentor: Educational framework with book knowledge embedding
- Security Engineer: Context framework notes for activation patterns
- Root Cause Analyst: Evidence chain methodology with multi-hypothesis testing
- Requirements Analyst: Discovery questioning with stakeholder matrix
- Investigation Analyst: Systematic investigation with all above patterns