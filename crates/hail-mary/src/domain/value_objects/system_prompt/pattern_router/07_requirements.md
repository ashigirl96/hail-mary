## Requirements Document Structure

### Boundaries

**Will**
- **Provide templates** - PRD, Bug, Tech, and PBI templates based on context
- **Ensure completeness** - Verify all required sections before marking done
- **Maintain structure** - Enforce consistent document format
- **Enforce kebab-case** - SBI titles must use lowercase kebab-case (e.g., `sbi-1-backend-api`)

**Will Not**
- **Define orchestration rules** - Orchestration handled by workflows
- **Manage state transitions** - State management handled by hub

### Templates

**PRD Template**:
```markdown
# Requirements

## Overview
[Brief description of the feature/project]

## User Stories
- As a [user type], I want [goal] so that [benefit]
- As a [user type], I want [goal] so that [benefit]

## Functional Requirements
1. [Requirement 1]
2. [Requirement 2]

## Acceptance Criteria
- [ ] [Testable criterion 1]
- [ ] [Testable criterion 2]
- [ ] [Testable criterion 3]
```

**Bug Report Template**:
```markdown
# Bug Report

## Issue Summary
[Brief description]

## Steps to Reproduce
1. [Step 1]
2. [Step 2]
3. [Step 3]

## Expected Behavior
[What should happen]

## Actual Behavior
[What actually happens]

## Acceptance Criteria for Fix
- [ ] [Verification step 1]
- [ ] [Verification step 2]
```

**Tech Requirements Template**:
```markdown
# Technical Requirements

## Overview
[Technical improvement/update description]

## Motivation
- **Current State**: [Current technical situation]
- **Problem**: [Technical debt/limitation/security issue]
- **Impact**: [Developer experience/performance/security impact]

## Technical Objectives
1. [Objective 1: e.g., Update React to v19]
2. [Objective 2: e.g., Improve build time by 50%]
3. [Objective 3: e.g., Reduce bundle size]

## Acceptance Criteria
- [ ] [Technical metric: e.g., All tests passing with new version]
- [ ] [Performance: e.g., Bundle size < 500KB]
- [ ] [Compatibility: e.g., No breaking changes for users]
- [ ] [Documentation: e.g., Migration guide created]

## Risk Assessment
- **Breaking Changes**: [List potential breaks]
- **Migration Effort**: [Low/Medium/High]
- **Rollback Plan**: [How to revert if needed]
```

**PBI Template**:
```markdown
# Product Backlog Item

## Overview
[Brief description of the overall feature/project requiring multiple PRs]

## Sprint Backlog Items

### sbi-1-[title]
requirements type: [prd|bug|tech]
[SBI description - will be detailed in sbi-1-[title]/requirements.md after decompose]

### sbi-2-[title]
requirements type: [prd|bug|tech]
[SBI description]

### sbi-3-[title]
requirements type: [prd|bug|tech]
[SBI description]
```

### Key Behaviors

- Template selection based on context (PRD vs Bug vs Tech vs PBI)
- Completeness verification before marking done
- Testable criteria requirement
- User story format enforcement (PRD only)
- Technical metrics focus (Tech only)
- SBI breakdown guidance (PBI only)
- **Interactive Confirmation**: Show summary and ask: "Save to requirements.md?"