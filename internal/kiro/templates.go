package kiro

import (
	"fmt"
	"time"
)

// GetRequirementsTemplate returns the default template for requirements.md
func GetRequirementsTemplate(featureTitle string) string {
	timestamp := time.Now().Format("2006-01-02")
	
	return fmt.Sprintf(`# Requirements Document: %s

## Overview
Date: %s
Feature: %s
Status: Draft

## Problem Statement
[Describe the problem this feature solves]

## Goals
- [ ] Primary goal 1
- [ ] Primary goal 2
- [ ] Primary goal 3

## Non-Goals
- What this feature will NOT address
- Scope limitations

## User Stories
### As a [user type]
I want to [action]
So that [benefit]

## Functional Requirements
### Must Have (P0)
- Requirement 1
- Requirement 2

### Should Have (P1)
- Requirement 1
- Requirement 2

### Nice to Have (P2)
- Requirement 1
- Requirement 2

## Non-Functional Requirements
### Performance
- Response time requirements
- Throughput requirements

### Security
- Authentication requirements
- Authorization requirements
- Data protection requirements

### Scalability
- Expected load
- Growth projections

## Technical Specifications
### Architecture
[High-level architecture description]

### Data Model
[Key entities and relationships]

### API Design
[API endpoints and contracts]

### Dependencies
- External services
- Libraries
- Infrastructure requirements

## Success Metrics
- Metric 1: [Description and target]
- Metric 2: [Description and target]
- Metric 3: [Description and target]

## Timeline
- Design Phase: [Duration]
- Development Phase: [Duration]
- Testing Phase: [Duration]
- Release: [Target date]

## Risks and Mitigations
| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Risk 1 | High/Medium/Low | High/Medium/Low | Mitigation strategy |

## Open Questions
- [ ] Question 1
- [ ] Question 2
- [ ] Question 3

## References
- [Link to related documents]
- [Link to design documents]
- [Link to technical specifications]

## Revision History
| Date | Version | Author | Changes |
|------|---------|--------|---------|
| %s | 1.0 | System | Initial draft |
`, featureTitle, timestamp, featureTitle, timestamp)
}