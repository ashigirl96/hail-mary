# Kiro Specification Management

## Recognition Patterns

When user mentions these keywords, apply corresponding action:
- 「要件」「requirements」「仕様」「PRD」→ Apply requirements template
- 「バグ」「不具合」「修正」「bug」→ Apply bug report template
- 「調査」「investigate」「検証」「research」→ Append to investigation.md
- 「設計」「design」「実装」「architecture」→ Generate design from investigation

## Document Templates

### Requirements Template (requirements.md)
```markdown
# Requirements - [Feature Name]

## Overview
[1-2 sentence description of what needs to be built]

## User Stories
- As a [user type], I want [goal] so that [benefit]
- As a [user type], I want [goal] so that [benefit]

## Acceptance Criteria
- [ ] The system shall [specific behavior]
- [ ] When [action], then [expected result]
- [ ] Performance: [metric requirement]

## Non-functional Requirements
- **Performance**: [Response time, throughput requirements]
- **Security**: [Authentication, authorization, data protection needs]
- **Scalability**: [User load, data volume expectations]
- **Compatibility**: [Browser, platform, API version requirements]

## Out of Scope
- [What this feature will NOT do]
- [Boundaries and limitations]
```

### Bug Report Template (requirements.md for bugs)
```markdown
# Bug Report - [Issue Title]

## Problem Description
[What is broken and impact on users]

## Steps to Reproduce
1. [First step]
2. [Second step]
3. [Observed behavior]

## Expected Behavior
[What should happen instead]

## Acceptance Criteria for Fix
- [ ] Issue no longer reproducible with original steps
- [ ] No regression in related functionality
- [ ] Fix includes test coverage

## Root Cause Analysis Required
- [ ] Investigate why this occurred
- [ ] Document in investigation.md
```

### Investigation Template (append to investigation.md)
```markdown
## [Topic Name] - YYYY-MM-DD HH:MM
**Confidence**: [0-100]%
**Status**: [exploring|validated|implemented]

### Findings
[Key discoveries and insights]

### Evidence
- **Source**: [file:line, URL, or documentation reference]
- **Data**: [Metrics, benchmarks, or test results]
- **Method**: [How this was investigated]

### Recommendations
- **Approach**: [Recommended solution]
- **Trade-offs**: [Pros and cons]
- **Alternatives**: [Other options considered]

### Next Steps
- [ ] [Required follow-up action]
```

### Design Template (design.md)
```markdown
# Technical Design - [Feature Name]

## Architecture Overview
[High-level system design and component interaction]

## Design Decisions

### Decision: [Decision Name]
- **Choice**: [Selected approach]
- **Evidence**: investigation.md#[section-name]
- **Requirements**: requirements.md#[section-name]
- **Trade-offs**:
  - ✅ Pros: [Benefits]
  - ⚠️ Cons: [Drawbacks]
  - 🔄 Alternatives considered: [Other options]

## Components

### [Component Name]
- **Purpose**: [What it does]
- **Interface**: [API/Contract]
- **Dependencies**: [What it needs]
- **Implementation Notes**: [Key considerations]

## Data Model
[If applicable: schemas, database design, API contracts]

## Security Considerations
[Authentication, authorization, data protection measures]

## Testing Strategy
- **Unit Tests**: [What to test at component level]
- **Integration Tests**: [Inter-component testing]
- **E2E Tests**: [User journey validation]

## Implementation Tasks
Extracted from design decisions above:
- [ ] [Task 1 - references: design.md#section]
- [ ] [Task 2 - references: design.md#section]
```

### Tasks Template (tasks.md)
```markdown
# Tasks - [Spec Name]

## State Tracking
| Document | Status | Confidence | Last Updated | Next Action |
|----------|--------|------------|--------------|-------------|
| requirements.md | none/partial/complete/outdated | - | - | Define requirements |
| investigation.md | none/partial/complete/outdated | 0-100% | - | Research [topic] |
| design.md | none/partial/complete/outdated | - | - | Create design |

## Timeline
### YYYY-MM-DD HH:MM
- ✅ [Action taken]
- 📝 [Document updated: requirements.md]
- 🔍 [Investigation: topic (confidence: X%)]
- 🎯 [Next: suggested action]

## Blocked Items
- ⏸️ [Task] blocked by: [dependency]
- ⏸️ [Decision] waiting for: [investigation topic]

## References
- Requirements: `.kiro/specs/[name]/requirements.md`
- Investigation: `.kiro/specs/[name]/investigation.md`
- Design: `.kiro/specs/[name]/design.md`
```

## Orchestration Rules

### Prerequisites and Flow Control

1. **Requirements First Rule**
   - If no requirements.md exists when user requests investigation or design:
     → 「要件が定義されていません。まず要件を明確にしましょう。何を作りたいですか？」

2. **Investigation Before Design Rule**
   - If user says 「設計して」but investigation.md is empty or confidence < 70%:
     → 「設計には十分な調査が必要です（現在の確信度: X%）。先に[topic]について調査しましょう。」
     → Auto-trigger investigation flow for missing topics

3. **Evidence-Based Design Rule**
   - All design decisions MUST reference investigation.md sections
   - If investigation lacks evidence for design choice:
     → 「この設計判断には調査が不足しています。[specific topic]を調査しますか？」

### Natural Language Type Detection

- 「バグを修正したい」「エラーが発生」「動作しない」→ Bug report template
- 「機能を追加」「作りたい」「実装したい」→ Requirements template
- 「どうやって」「なぜ」「比較」「ベストプラクティス」→ Investigation
- 「アーキテクチャ」「実装方法」「コンポーネント設計」→ Design

### Update Patterns

1. **Append Pattern (investigation.md)**
   - ALWAYS append new findings, never overwrite
   - Each entry timestamped with confidence level
   - Previous investigations remain for history

2. **Replace Pattern (requirements.md, design.md)**
   - Update entire document with new version
   - Preserve key decisions but refine details
   - Mark previous version as outdated in tasks.md

### Tasks.md Management

After EVERY Kiro operation:
1. Read current tasks.md (or create if missing)
2. Update State Tracking table
3. Append to Timeline with timestamp and emoji
4. Identify blocked items or dependencies
5. Update Next Action column

### Nudging Behaviors (80% suggestion, 20% enforcement)

**Suggestions (user can override):**
- 「要件が更新されました。調査も更新しますか？ [Y/n]」
- 「調査が完了しました（確信度: 85%）。設計に進みますか？ [Y/n]」
- 「すべてのドキュメントが揃いました。実装を開始しますか？ [Y/n]」

**Enforcement (block until resolved):**
- Cannot design without requirements (must define first)
- Cannot design with confidence < 50% (must investigate)
- Cannot implement without design (must design first)

### Confidence Scoring

Investigation confidence based on:
- Multiple sources cited: +20% per unique source (max 60%)
- Concrete evidence (metrics/benchmarks): +30%
- Implementation tested: +30%
- Alternative approaches evaluated: +10%
- Total capped at 100%

### Examples of Natural Flow

**Example 1: User says 「ユーザー認証を実装したい」**
1. Recognize: "実装したい" → Requirements needed
2. Check: No requirements.md exists
3. Response: Apply requirements template for authentication
4. Update: tasks.md with new requirement
5. Nudge: 「要件を定義しました。認証方式について調査しますか？」

**Example 2: User says 「設計して」**
1. Recognize: "設計" → Design needed
2. Check: investigation.md confidence = 40%
3. Response: 「調査の確信度が40%です。以下の項目を調査する必要があります：」
   - JWT vs Session authentication
   - Security best practices
   - Performance implications
4. Auto-trigger: Investigation for missing topics
5. After investigation: Apply design template with evidence links

**Example 3: User says 「バグがある、ログインできない」**
1. Recognize: "バグ" + "ログインできない" → Bug report
2. Apply: Bug report template
3. Update: tasks.md marking requirements as "bug"
4. Nudge: 「バグレポートを作成しました。根本原因を調査しますか？」