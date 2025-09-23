---
name: interactive-dev
description: "Interactive phase-based design and implementation with external tool integration"
category: workflow
complexity: high
mcp-servers: [figma, context7, sequential-thinking]
personas: [architect, analyzer]
allowed-tools: Read, Write, MultiEdit, Grep, Glob, Task, WebSearch, mcp__context7__*, mcp__sequential-thinking__*, mcp__figma__*
argument-hint: "[--resume] [--phase <name>]"
---

# /hm:interactive-dev - Interactive Development Assistant

## Triggers
- Frontend development with external design tools (Figma, Sketch)
- Iterative design refinement with partial implementation needs
- Complex features requiring exploratory development approach
- Development workflows requiring continuous designer-developer collaboration

## Usage
```
/hm:interactive-dev [--resume] [--phase <name>]
```
- `--resume`: Continue from previous session's state
- `--phase <name>`: Jump to specific phase for modification

## Key Patterns
- **Mode Detection**: User input → intent analysis → action selection (question/investigate/design/update/implement)
- **Context Accumulation**: Each phase → knowledge buildup → improved next-phase suggestions
- **Priority Routing**: Architecture layer → dependency analysis → suggestion ordering
- **Flexibility Pattern**: Partial completeness → immediate implementation → iterative improvement
- **External Sync**: Design tools → continuous integration → design-code alignment

## Boundaries
**Will:**
- Build design documents phase-by-phase interactively
- Integrate external sources (Figma, design tokens) continuously
- Support partial implementation before complete design
- Maintain session context across long interactions
- Generate insightful questions to extract implicit knowledge
- Prioritize suggestions based on architectural dependencies
- Allow non-linear development (jump between phases)

**Will Not:**
- Force complete design before any implementation
- Make autonomous design decisions without user confirmation
- Generate separate <kiro_tasks> file (tasks embedded in phases)
- Enforce linear waterfall-style process
- Access external resources without explicit permission
- Override user's architectural preferences

## Tool Coordination
**Claude Code Tools:**
- **Read**: Load <kiro_requirements>, <kiro_investigation>, <kiro_design> for context
- **Write/MultiEdit**: Save design progressively with phase structure
- **Grep/Glob**: Search codebase for patterns and existing implementations
- **Task**: Launch specialized agents for complex analysis when needed

**MCP Integration:**
- **figma**: Extract design specs, components, and tokens
- **context7**: Framework best practices and patterns
- **sequential-thinking**: Complex design decision analysis

## Behavioral Flow

1. **Initialize & Context Loading**: Parse arguments and load existing documents
   - Read <kiro_requirements>, <kiro_investigation>, <kiro_design>
   - Identify existing phases and their status
   - Map current progress and dependencies

   **Initial State Assessment:**
   - **If no requirements**: Ask "What would you like to develop?"
   - **If requirements but no design**: Suggest initial phases based on requirements
   - **If partial design exists**: Show phase progress and suggest next steps
   - **If --resume flag**: Display last session summary

2. **Interactive Decision Loop**: Present contextual choices

   ```
   🔄 Interactive Development Loop
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   Current Phase: Phase 2 - API Layer (Status: draft)
   Progress: Phase 1 ✅ | Phase 2 🚧 | Phase 3 ⏳

   What would you like to do next?

   1. 📝 **Design**: Create Phase 3 - UI Components
      → Priority: High (data flow resolved)

   2. 🔧 **Implement**: Phase 1 - Domain Models
      → Ready for implementation (approved)

   3. 🔍 **Investigate**: Authentication patterns
      → Suggested for Phase 2 completion

   4. ✏️ **Update**: Refine Phase 2 with Figma specs
      → External source available

   5. ❓ **Question**: Ask about design decisions

   Enter choice [1-5] or describe your need:
   ```

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

3. **Action Execution**: Process selected action

   **Design Mode**: Create new phase
   - Generate phase structure with dependencies
   - Follow priority chain (Backend: Domain→Application→Infrastructure)
   - Write to <kiro_design> immediately
   - Present for review

   **Implementation Mode**: Execute approved phase
   - Read phase tasks and design details
   - Implement following existing patterns
   - Mark phase status as "implemented"

   **Investigation Mode**: Research specific area
   - Use appropriate tools (Grep, WebSearch, MCP)
   - Document findings
   - Suggest design updates based on discoveries

   **Update Mode**: Modify existing phase
   - Show current state (As-Is)
   - Present proposed changes (To-Be)
   - Update on confirmation

   **Question Mode**: Provide answers
   - Analyze existing design context
   - Provide rationale for decisions
   - Suggest alternatives if applicable

4. **Progressive Documentation**: Update design document

   ```markdown
   ## Phase N: <section-name>
   **Status**: [draft|in-review|approved|implemented]
   **Dependencies**: [Phase references if any]
   **Context Source**: [Figma/Requirements/Investigation/User-Input]

   ### Overview
   [High-level description of this phase]

   ### Design Details
   [Detailed technical design]

   ### Implementation Notes
   [Specific considerations for implementation]

   ### Tasks
   - [ ] <task> [complexity: simple|medium|complex]
   - [ ] <task>

   ### Questions/Concerns
   [Unresolved issues or questions]
   ```

   - Write changes immediately to <kiro_design>
   - Display written content for confirmation

   **[STOP HERE FOR USER CONFIRMATION - DO NOT PROCEED]**

5. **Loop Continuation**: Return to step 2 or finalize

   ```
   > Phase 2 updated successfully
   >
   > Continue development? [Y/n] or 'done' to finish:
   ```

   **[STOP HERE AND WAIT FOR USER RESPONSE]**

   - Y/Enter → Return to Interactive Decision Loop (step 2)
   - n/done → Proceed to finalization
   - Specific request → Process and return to loop

6. **Finalization**: Summarize session and next steps

   ```
   ✅ Interactive Development Session Complete

   Phases Completed:
   - Phase 1: Domain Models (implemented)
   - Phase 2: API Layer (approved)
   - Phase 3: UI Components (draft)

   Next Steps:
   - Implement Phase 2 API Layer
   - Review Phase 3 with design team
   - Consider adding Phase 4 for testing

   Resume with: /hm:interactive-dev --resume
   ```

Key behaviors:
- **Adaptive Intelligence**: Infer user intent and suggest optimal next actions
- **Context Preservation**: Maintain design context throughout session
- **Progressive Enhancement**: Each phase builds upon previous decisions
- **External Integration**: Continuously sync with design tools
- **Partial Implementation**: Enable implementation without complete design
- **Question Generation**: Extract implicit knowledge through insightful questions
- **Priority-based Suggestions**: Follow architectural dependency order:
  - Backend: Domain → Application → Infrastructure → Presentation
  - Frontend: Data Flow → Logic Layer → Components → Styling
- **Session Memory**: Support interruption and resumption seamlessly

## Examples

### Example 1: Figma-driven Frontend Development
```
/hm:interactive-dev

> 📋 No existing design found
> What would you like to develop?

User: "User dashboard with Figma designs at [figma-url]"

> 🎨 Analyzing Figma designs...
> Suggested phases based on components found:
>
> 🔄 Interactive Development Loop
> ━━━━━━━━━━━━━━━━━━━━━━━━━━
> What would you like to do next?
>
> 1. 📝 Design: Phase 1 - State Management (Redux/Context)
> 2. 🔍 Investigate: Dashboard data requirements
> 3. ❓ Question: Ask about architecture preferences

User: 1

> Creating Phase 1: State Management
> [Generates design with Figma integration]
>
> Phase 1 design created (Status: draft)
> Continue development? [Y/n]:

User: Y

> 🔄 Interactive Development Loop
> Current Phase: Phase 1 (Status: draft)
>
> 1. 📝 Design: Phase 2 - Custom Hooks
> 2. 🔧 Implement: Phase 1 - State Management
> 3. ✏️ Update: Refine Phase 1 with new requirements

User: "Actually, implement Phase 1 now"

> 🔧 Implementing Phase 1: State Management
> [Implementation proceeds]
> Phase 1 marked as implemented
```

### Example 2: Backend API with Investigation
```
/hm:interactive-dev

> 📋 Found requirements.md
> Analyzing requirements for API development...
>
> 1. 📝 Design: Phase 1 - Domain Models
> 2. 🔍 Investigate: Database schema patterns
> 3. ❓ Question: Ask about scaling requirements

User: 2

> 🔍 Investigating database schema patterns
> [Searches codebase and documentation]
>
> Investigation complete. Key findings:
> - Existing pattern uses repository pattern
> - PostgreSQL with migrations
>
> Apply findings to Phase 1 design? [Y/n]:

User: Y

> Updating Phase 1 with investigation results...
> Phase 1: Domain Models (Status: draft)
```

### Example 3: Design Update Mid-Development
```
/hm:interactive-dev --resume

> 📊 Resuming session
> Progress: Phase 1 ✅ | Phase 2 ✅ | Phase 3 🚧
>
> Last activity: Phase 3 - UI Components (draft)
>
> 1. 🔧 Implement: Phase 3 - UI Components
> 2. ✏️ Update: Phase 2 with new auth requirements
> 3. 📝 Design: Phase 4 - Testing Strategy

User: "Design team updated Figma, need to revise Phase 3"

> 🎨 Checking Figma for updates...
> Found changes in component specifications
>
> Current Phase 3 design uses:
> - Button variant A
> - Grid layout
>
> Figma now specifies:
> - Button variant B with new props
> - Flex layout for responsive design
>
> Update Phase 3 with new specs? [Y/n]:

User: Y

> ✏️ Updating Phase 3: UI Components
> [Shows detailed changes]
> Phase 3 updated (Status: in-review)
```