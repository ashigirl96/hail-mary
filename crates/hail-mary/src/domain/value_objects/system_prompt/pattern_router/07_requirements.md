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

**Key Behaviors**:
- Template selection based on context (PRD vs Bug vs Tech)
- Completeness verification before marking done
- Testable criteria requirement
- User story format enforcement (PRD only)
- Technical metrics focus (Tech only)

**Interactive Confirmation**:
Show summary and ask: "Save to requirements.md?"