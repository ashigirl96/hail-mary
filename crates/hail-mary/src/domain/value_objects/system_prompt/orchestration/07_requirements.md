## Requirements Document Structure

**Boundaries**:
- **Will**: Provide templates, ensure completeness, maintain structure
- **Will Not**: Define orchestration rules, manage state transitions

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

**Key Behaviors**:
- Template selection based on context (feature vs bug)
- Completeness verification before marking done
- Testable criteria requirement
- User story format enforcement

**Interactive Confirmation**:
Show summary and ask: "Save to requirements.md? (Y to save / or provide feedback)"