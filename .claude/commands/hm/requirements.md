---
name: requirements
description: "Generate structured requirement documents from user needs or GitHub issues"
category: workflow
complexity: standard
mcp-servers: [github]
personas: [analyst, architect]
allowed-tools: Read, Write, MultiEdit, mcp__github__get_issue
argument-hint: "[--type prd|bug] [--issue <github-url>]"
---

# /hm:requirements - Requirements Document Generator

## Triggers
- Starting new feature development requiring structured documentation
- Bug reporting that needs formal issue documentation
- GitHub issue needs to be converted to actionable requirements
- Project planning phase initiation

## Usage
```
/hm:requirements [--type prd|bug] [--issue <github-url>]
```
- `--type`: Document type (defaults to prd if omitted)
- `--issue`: Optional GitHub issue URL for auto-population

## Key Patterns
- **Type Detection**: --type prd → PRD template activation
- **Type Detection**: --type bug → Bug template activation
- **Source Detection**: --issue present → GitHub MCP activation
- **Source Detection**: no --issue → Interactive mode activation
- **Complexity Assessment**: PRD → high complexity → multiple iterations
- **Persona Activation**: Requirements analysis → analyst + architect

## Boundaries
**Will:**
- Generate and update <kiro_requirements_path> only
- Fetch and parse GitHub issues when provided
- Calculate and display completeness scores
- Achieve maximum 70% completeness through interactive refinement with user
- Iterate based on user feedback until satisfaction or reaching 70% completion
- Structure content differently for PRD vs Bug types
- Include references to source documents and materials used
- Think in English, document in Japanese

**Will Not:**
- Exceed 70% completeness (requires `/hm:investigate` for technical discovery)
- Attempt to fill technical sections without codebase investigation
- Perform investigation, design, or task decomposition
- Modify files other than <kiro_requirements_path>
- Auto-generate content without user input
- Proceed without explicit user confirmation at [STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED] points
- Make assumptions about technical implementation details
- End iterative refinement without explicit user approval

## Tool Coordination
**Claude Code Tools:**
- **Read**: Attempt to read <kiro_requirements_path> (ignore errors if file doesn't exist)
- **Write/MultiEdit**: Create or update <kiro_requirements_path> (Write creates parent directories automatically)

**MCP Integration:**
- **GitHub Server**: Use `mcp__github__get_issue` to fetch issue details
  - Extract title, description, labels, comments
  - Parse into appropriate requirement sections
  - Maintain issue link for traceability

## Document Templates

### PRD Template
```markdown
# Requirements - [Feature Name]

## Metadata
- **Completeness**: [0-100%]
- **Source**: [user-input|github-issue: URL]
- **References**:
  - [List of consulted documents]
  - [Will be populated by /hm:investigate]

## 1. Overview
- Problem statement
- Proposed solution

## 2. Purpose
- [Why this feature is needed]

## 3. User Stories
- As a [user], I want [feature] so that [benefit]
- Priority: [P0/P1/P2]

## 4. Acceptance Criteria
- Given [context], When [action], Then [outcome]
- Edge cases and error conditions

## 5. Technical Requirements
[TBD - populated by /hm:investigate]
- Architecture decisions
- Dependencies
- Integration points
- Impact analysis

---

## Completeness Scoring Rule
- **0-70%**: User requirements and business context
  - Problem definition, user stories, acceptance criteria
  - Maximum achievable through user interaction alone
- **70-100%**: Technical completion
  - Dependencies, constraints, impact analysis
  - Requires codebase investigation
```

### Bug Template
```markdown
# Bug Report - [Title]

## Metadata
- **Severity**: [Critical/High/Medium/Low]
- **Completeness**: [0-100%]
- **Source**: [user-input|github-issue: URL]
- **References**:
  - [Error logs/screenshots]
  - [Will be populated by /hm:investigate]

## 1. Problem
- **What's broken**: [user description]
- **How to reproduce**:
  1. [Step by step]
- **Error/Logs**: [if any]

## 2. Expected
- **Should do**: [expected behavior]
- **Success criteria**: [how to verify fix]

## 3. Technical Details
[TBD - populated by /hm:investigate]
- Root cause
- Affected files
- Fix approach
- Impact analysis

---

## Completeness Scoring Rule
- **0-70%**: Problem documentation
  - Symptoms, reproduction steps, expected behavior
  - Maximum achievable through user reporting
- **70-100%**: Root cause identification
  - Root cause, affected components, technical context
  - Requires codebase investigation
```

## Behavioral Flow

1. **Initialize**: Parse command arguments and determine document type
- **Attempt** to Read <kiro_requirements_path> for existing content:
  - If file exists: Load and analyze current completeness
  - If file not found: Skip silently and proceed to step 2
  - **DO NOT**: Use ls, Bash, or Glob to search for files
  - **DO NOT**: Create directories or investigate structure

2. **Interactive Requirements Gathering**: Present type-specific questions
- **For PRD**: "What new feature or capability would you like to develop? Please describe the problem you're solving, target users, and desired outcomes."
- **For Bug**: "Please describe the current problematic behavior and what the expected behavior should be. Include steps to reproduce if possible."

**[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

3. **Document Generation**: Create initial <kiro_requirements_path> draft
- Parse user input and structure into appropriate sections
- Calculate completeness score (weighted by section importance)
- Display generated document with completeness percentage
- Show which sections are complete (✅) vs pending (⏳)
- Display what's missing to reach 70% maximum
- Ask: "Here's the initial requirements document (Completeness: XX%). Is this accurate? [Y/n] or provide clarification:"

**[STOP HERE AND WAIT FOR USER CONFIRMATION - DO NOT PROCEED]**

4. **Iterative Refinement**: Process user feedback and save
- If "Y" or Enter → Write to <kiro_requirements_path> and proceed to step 5
- If "n" or clarification provided → Update document based on feedback
- Recalculate completeness score
- Return to step 3 for re-confirmation

5. **Summary**: Display final results and next steps
- Show final completeness score
- Display document summary
- Confirm successful save with path
- **Show next steps**:
  - If completeness < 70%: "Continue refining or run `/hm:investigate` for technical details"
  - If completeness = 70%: "Run `/hm:investigate` to complete technical sections (70% → 100%)"
  - Always mention: "After investigation, proceed with `/hm:design` for implementation planning"

Key behaviors:
- **Completeness Tracking**: Display document completeness percentage after each generation
- **GitHub Integration**: Auto-extract and structure issue content when URL provided
- **Interactive Refinement**: Multiple feedback rounds until user satisfaction
- **Type-Specific Templates**: Different structures for PRD vs Bug documentation
- **Progressive Enhancement**: Document grows through conversation, not all at once

## Examples

### Example 1: PRD Creation
```
/hm:requirements --type prd

> 📋 Starting PRD creation...
> What new feature or capability would you like to develop?
> Please describe:
> - The problem you're solving
> - Target users
> - Desired outcomes
>
> [STOP AND WAIT FOR USER INPUT]

User: "We need a dashboard for monitoring system health..."

> 📝 Generated requirements document (Completeness: 65%):
> ✅ Overview: Complete
> ✅ Purpose: Complete
> ✅ User Stories: Complete
> ✅ Acceptance Criteria: Complete
> ⏳ Technical Requirements: Pending investigation
>
> Maximum 70% achievable through user input.
> [Document content...]
> Is this accurate? [Y/n] or provide clarification:

User: Y

> ✅ Requirements saved to <kiro_requirements_path>
>
> **Next steps:**
> - Run `/hm:investigate --for requirements` to complete technical sections (65% → 100%)
> - After investigation, proceed with `/hm:design` for implementation planning
```

### Example 2: Bug Documentation from GitHub Issue
```
/hm:requirements --type bug --issue https://github.com/org/repo/issues/123

> 🔍 Fetching GitHub issue #123...
> 📋 Analyzing issue content...
>
> Based on the issue, here's the bug documentation (Completeness: 70%):
> ✅ Problem: Complete
> ✅ Expected: Complete
> ⏳ Technical Details: Pending investigation
>
> Maximum 70% achievable without codebase analysis.
> [Generated bug report...]
>
> Would you like to add any additional context? [Y/n]:

User: n

> ✅ Bug requirements saved to <kiro_requirements_path>
>
> **Next steps:**
> - Run `/hm:investigate --for requirements` to identify root cause (70% → 100%)
> - After investigation, proceed with `/hm:design` for fix planning
```

### Example 3: Iterative Refinement
```
/hm:requirements --type prd

[Initial gathering...]

> 📝 Generated requirements (Completeness: 50%):
> ✅ Overview: Complete
> ✅ Purpose: Complete
> ⏳ User Stories: Partial
> ⏳ Acceptance Criteria: Partial
> ⏳ Technical Requirements: Pending investigation
>
> [Document content...]
> Is this accurate? [Y/n] or provide clarification:

User: n, we also need to consider mobile users and API integration

> 📝 Updated requirements (Completeness: 70%):
> ✅ Overview: Complete
> ✅ Purpose: Complete
> ✅ User Stories: Complete (mobile added)
> ✅ Acceptance Criteria: Complete (API criteria added)
> ⏳ Technical Requirements: Pending investigation
>
> Maximum 70% reached - technical details require `/hm:investigate --for requirements`.
> [Updated document with mobile and API sections...]
> Is this accurate? [Y/n]:

User: Y

> ✅ Requirements finalized (Completeness: 70%)
>
> **Next steps:**
> - Run `/hm:investigate --for requirements` to complete technical analysis (70% → 100%)
> - After investigation, proceed with `/hm:design` for implementation planning
```
