# Design

## Meta
- **Completeness**: 95%
- **Requirements**: Implementation timeline management after design completion
- **Architecture Scope**: Full-stack (Pattern Router Framework + Slash Commands)

## Overview

**As-Is**:
- design完了後、タスク分解が「副作用」として曖昧に処理される
- nudgingが機械的（YES/NO選択）で、phase概念なし
- タスク分解がパターンとして認識されていない
- State Trackingに実装進捗が見えない
- Lost in the Middle問題: design-completeイベントが見落とされる可能性

**To-Be**:
- design完了後、明示的なタスク分解プロトコルで会話的にphased plan作成
- 自動フロー（design後）+ オプショナルコマンド（/hm:timeline）のハイブリッド
- State Trackingでtasks.md#Timelineの進捗を追跡
- MUST指示と会話例でLost in the Middle問題を解決
- phase概念導入で実装順序を柔軟に管理

## Design

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/06_nudges.md

**Purpose**: design-complete nudgingテンプレートをphased plan形式に改善

**Current Problem**: 機械的な確認（"Shall I add these to Timeline?"）で、phase概念なし

**Solution**: 会話的なphased plan提示形式に変更

````markdown
### Event: design-complete (After)

**Phased Implementation Proposal Format**:
```
Design complete! I've analyzed the design and suggest this phased implementation approach:

**Phase 1: [Name]** (design.md#section-ref)
- [ ] [Task 1 with details]
- [ ] [Task 2 with details]
- [ ] [Validation step]

**Phase 2: [Name]** (design.md#section-ref)
- [ ] [Task 1]
- [ ] [Task 2]

**Phase 3: [Name]** (design.md#section-ref)
- [ ] [Task 1]

What do you think? You can:
- Adjust the phase grouping
- Change the order
- Modify task granularity
- Add or remove tasks
- Proceed as-is
```

**Alternative phrasing** (if design is simple):
- "I've identified [N] implementation tasks. Would you like me to add them to Timeline, or shall we discuss the approach first?"

### Event: timeline-planned (After Timeline update)
- "Implementation plan added to Timeline! [N] phases, [M] total tasks."
- "Ready to start with phase1: [first-task]?"
- "Current focus: tasks.md#Timeline tracking implementation progress"
````

**Evidence**: NO Linear Workflow哲学に合致（会話的インタラクション、調整可能）

---

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/04_workflows.md

**Purpose**: After Design Completeプロトコルを強化、Lost in the Middle問題を解決

**Current Problem**:
- プロトコルが曖昧で、実行が保証されない
- 会話例がなく、実行イメージが不明確

**Solution**: MUST指示と多ターン会話例を追加

````markdown
**After Design Complete** (MANDATORY PROTOCOL):
1. Mark design.md = complete in State Tracking
2. Add to Timeline: `[x] Design completed → design.md#overview`
3. **MUST analyze design.md and extract implementation tasks**
4. **MUST group tasks into logical phases with clear dependencies**
5. **MUST present phased implementation plan to user** (use <kiro-nudges> design-complete template)
6. **MUST engage in interactive discussion**:
   - User may adjust phase grouping
   - User may change order
   - User may add/remove tasks
   - User may request different granularity
7. **MUST wait for user confirmation before Timeline update**
8. After confirmation: Add finalized plan to Timeline with phase structure
9. Update State Tracking: tasks.md#Timeline = pending (0%)
10. Trigger nudge event: `timeline-planned`

**Conversation Flow Example**:
```
Claude: [Presents phased plan using <kiro-nudges> design-complete template]
       Design complete! I've analyzed the design and suggest this phased approach:

       **Phase 1: Core Authentication** (design.md#auth-service)
       - [ ] Implement JWT service with jose library
       - [ ] Add unit tests (coverage >80%)
       - [ ] Run bun lint

       **Phase 2: API Endpoints** (design.md#api-endpoints)
       - [ ] Create login endpoint
       - [ ] Create refresh endpoint
       - [ ] Integration tests with Playwright

       What do you think?

User: "Phase 2 seems too big, can we split refresh endpoint separately?"

Claude: Good idea! Here's the revised plan:

       **Phase 2a: Login Endpoint**
       - [ ] Create login endpoint (/api/auth/login)
       - [ ] Basic integration tests

       **Phase 2b: Refresh Endpoint**
       - [ ] Create refresh endpoint (/api/auth/refresh)
       - [ ] Complete integration test suite

       Shall I add this to Timeline?

User: "Yes, perfect"

Claude: Implementation plan added to Timeline! 4 phases, 8 tasks total.
       State Tracking updated: tasks.md#Timeline = pending (0%)
       Ready to start with phase1: Implement JWT service?
```
````

**New Section: Timeline Planning Protocol** (Manual Invocation):

````markdown
## Timeline Planning Protocol (Manual Invocation)

**Trigger**: `/hm:timeline` or natural language patterns ("plan implementation", "organize tasks")

**Pre-conditions**: design.md should exist and be complete (warning if not)

**Flow**:
1. Read design.md to understand implementation scope
2. Read current tasks.md#Timeline to check existing plan
3. Analyze existing phases (if any) or propose new plan
4. Present current plan or propose new phased approach
5. **MUST engage in interactive adjustment with user**:
   - Review phase grouping
   - Adjust task order
   - Break down or combine tasks
   - Add dependencies
6. After user confirmation: Update tasks.md#Timeline
7. Update State Tracking: tasks.md#Timeline status and coverage
8. Trigger nudge event: `timeline-updated`

**Use Cases**:
- Re-planning after design changes
- Adjusting task order mid-implementation
- Breaking down phases into smaller tasks
- Reviewing implementation progress and reorganizing remaining work
- Creating implementation plan if skipped during design phase

**Interactive Patterns**:
- User: "Let's reorganize phase 2" → Show current phase 2 → Discuss changes
- User: "Break down phase 1 into smaller tasks" → Propose detailed breakdown
- User: "Can we do phase 3 before phase 2?" → Analyze dependencies → Confirm/warn
````

**Evidence**: Pattern-Based Routing原則、Conditional Hub Access（Command Pipeline）

---

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/02_hub.md

**Purpose**: State Trackingにtasks.md#Timeline行を追加、Timeline例にphase構造を反映

**Current Problem**:
- 実装進捗がState Trackingに表示されない
- Timeline例にphase概念がない

**Solution**: State Tracking TableとTimeline Formatを更新

````markdown
**State Tracking Structure**:
```markdown
## State Tracking
| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | complete | - | - |
| investigation.md | complete | 5/5 (100%) | - |
| design.md | complete | - | - |
| tasks.md#Timeline | in-progress | phase1: 2/3 (67%) | Implement refresh endpoint |
```

**Note**: tasks.md#Timeline represents implementation progress tracking:
- Status: pending (0%) → in-progress (N%) → complete (100%)
- Coverage: "phaseX: M/N (P%)" format showing current phase and task completion
- Next Action: Specific next task from current phase

**Timeline Format**:
```markdown
## Timeline
- [x] Requirements defined → requirements.md#overview
- [x] All investigations complete → investigation.md
- [x] Design completed → design.md#overview
- [x] Implementation plan confirmed with user
- [x] phase1: Core Authentication → design.md#auth-service
  - [x] Implement JWT service (jose library)
  - [x] Add unit tests (coverage >80%)
  - [x] Run bun lint
- [ ] phase2: API Endpoints → design.md#api-endpoints
  - [x] Create login endpoint (/api/auth/login)
  - [ ] Create refresh endpoint (/api/auth/refresh)
  - [ ] Integration tests with Playwright
- [ ] phase3: Frontend Integration → design.md#frontend
  - [ ] Login form component
  - [ ] Token storage with httpOnly cookies
  - [ ] Error handling and user feedback
```
````

**Rationale**:
- requirements/investigation/design = Source documents
- tasks.md#Timeline = Implementation checklist
- Both are trackable states, different abstraction levels
- No circular reference: Timeline is data, State Tracking is meta-information

**Evidence**: Single Source of Truth原則、tasks.mdはCentral Hub

---

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/03_patterns.md

**Purpose**: /hm:timeline パターンを追加してTimeline操作を明示的に認識

**Current Problem**: タスク分解がパターンとして認識されていない

**Solution**: EXPLICIT Patternsテーブルに追加

````markdown
| User Pattern | Action | Strategy Output |
|-------------|--------|-----------------|
| "/hm:requirements", "Create requirements" | Create/Update | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "/hm:investigate", "investigate", "research" | Append | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "/hm:design", "design", "architecture" | Create (validated) | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "/hm:timeline", "plan implementation", "organize tasks" | Timeline Planning | `{class: "EXPLICIT", strategy: "command", components: ["hub", "workflows", "document", "nudges"]}` |
```

**Note**: /hm:timeline uses same components except "gates" (no validation gates for timeline planning)

**Routing Decision Example**:

```markdown
Input: "/hm:timeline"
→ Class: EXPLICIT
→ Confidence: 1.0
→ Strategy: command
→ Components: ["hub", "workflows", "document", "nudges"]
→ Route to: Command Pipeline (Timeline Planning Protocol)

Input: "Let's reorganize the implementation phases"
→ Class: EXPLICIT (natural language pattern match)
→ Confidence: 0.9
→ Strategy: command
→ Components: ["hub", "workflows", "document", "nudges"]
→ Route to: Command Pipeline (Timeline Planning Protocol)
````

**Evidence**: Pattern Recognition over Process原則

---

### .claude/commands/hm/timeline.md (New File)

**Purpose**: オプショナルなTimeline操作コマンドを提供

**Rationale**:
- requirements/investigate/designはドキュメント作成コマンド
- /hm:timelineはtasks.md操作コマンド
- 性質が異なるため、別コマンドとして正当
- design完了後の自動フローに加え、いつでも再調整可能にする

**Implementation**:

````markdown
---
name: timeline
description: "Plan and manage implementation timeline - triggered by: timeline, plan implementation, organize tasks"
allowed-tools: Read, Write, MultiEdit
---

# /hm:timeline

Plan and manage implementation timeline with reactive pattern-based routing.

## Use Cases
- Plan implementation after design completion
- Re-organize existing implementation phases
- Adjust task order mid-implementation
- Break down phases into smaller tasks
- Review implementation progress

## Behavior

This command invokes the Timeline Planning Protocol defined in system prompt.

Refer to system prompt sections:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-hub> for tasks.md Timeline structure and State Tracking
- <kiro-patterns> for timeline planning pattern recognition
- <kiro-workflows> for Timeline Planning Protocol execution
- <kiro-nudges> for phased plan templates

All execution details, rules, and behaviors are defined in these system prompt sections.
````

**Evidence**: Routing without Control（明示的コマンドで柔軟性提供）

---

### .claude/commands/hm/design.md (Enhancement)

**Purpose**: design完了後の動作をリマインド（オプショナル）

**Current State**: System prompt sectionsを参照するのみ

**Enhancement**: Post-Completion Behaviorセクションを追加

````markdown
# /hm:design

Create technical design documentation with reactive pattern-based routing.

## Post-Completion Behavior

After design is complete, the workflow will automatically:
1. Extract implementation tasks from design.md
2. Propose a phased implementation plan
3. Discuss and adjust the plan with you interactively
4. Add the finalized plan to tasks.md#Timeline

You can always re-plan or adjust using `/hm:timeline`.

Refer to system prompt sections:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-hub> for tasks.md central control mechanism
- <kiro-patterns> for pattern recognition and mapping
- <kiro-workflows> for **After Design Complete protocol and Timeline Planning**
- <kiro-gates> for validation gates and prerequisites
- <kiro-nudges> for **phased plan templates**
- <kiro-design> for design document structure

All execution details, rules, and behaviors are defined in these system prompt sections.
````

**Note**: このenhancementはオプショナル。XML-based Orchestration原則では、詳細はsystem promptに委譲すべき。ただし、軽いリマインダーとして有用。

**Evidence**: Clarity（ユーザー期待値管理）

---

## Implementation Order

**Priority-based execution**:

1. **06_nudges.md** (Highest Impact)
   - design-completeテンプレート改善
   - 即座にnudgingの質が向上

2. **04_workflows.md** (Core Solution)
   - After Design Complete強化（MUST指示 + 会話例）
   - Timeline Planning Protocol追加
   - Lost in the Middle問題の核心的解決

3. **02_hub.md** (Clarity)
   - State Tracking拡張
   - Timeline例更新
   - 設計の責務分離を明確化

4. **03_patterns.md** (Pattern Recognition)
   - /hm:timeline パターン追加
   - Pattern-Based Routingの完全化

5. **.claude/commands/hm/timeline.md** (Flexibility)
   - 新コマンド作成
   - オプショナルな明示的操作を提供

6. **.claude/commands/hm/design.md** (Optional)
   - リマインダー追加
   - ユーザー期待値管理

**Note**: mod.rsの変更は不要（既存ファイル編集のみ）

---

## Validation Checklist

- ✅ NO Linear Workflow: 会話的インタラクション、phase調整の柔軟性
- ✅ Pattern Recognition: /hm:timelineをパターンとして認識
- ✅ Conditional Hub Access: Command Pipelineでtasks.md更新
- ✅ Routing without Control: ユーザーは順序を自由に調整可能
- ✅ Evidence-Based Progress: design.md → Timeline への明確なトレーサビリティ
- ✅ Autonomy with Safety: 自動フロー + 明示的コマンドのハイブリッド
- ✅ Single Source of Truth: tasks.md#Timelineが実装状態の単一権威

---

## Architecture Decision Records

**ADR-001: Why tasks.md#Timeline in State Tracking?**
- Decision: Add tasks.md#Timeline as a row in State Tracking Table
- Rationale:
  - requirements/investigation/design = Source documents
  - tasks.md#Timeline = Implementation progress
  - Both are states that need tracking
  - No circular reference: different abstraction levels
- Alternative considered: Separate "Implementation Progress" section
- Rejected because: Increases complexity, violates Single Source of Truth

**ADR-002: Why /hm:timeline as separate command?**
- Decision: Create /hm:timeline in addition to automatic post-design flow
- Rationale:
  - requirements/investigate/design = Document creation
  - /hm:timeline = tasks.md operation
  - Different nature justifies separate command
  - Hybrid approach (auto + manual) provides flexibility
- Alternative considered: IMPLICIT pattern only
- Rejected because: Needs Command Pipeline (hub write access), IMPLICIT uses Suggestion Pipeline

**ADR-003: Why MUST instructions in workflows?**
- Decision: Use strong imperative language (MUST, SHALL, ALWAYS)
- Rationale: Solve Lost in the Middle problem
- Claude Code may skip steps in long system prompts
- Strong directives reduce skip rate
- Alternative considered: Priority override wrapper in slash command
- Rejected because: Too heavy, should be emergency-only pattern

**ADR-004: Why conversation examples in 04_workflows.md?**
- Decision: Embed multi-turn conversation examples in workflows
- Rationale:
  - /hm:design references <kiro-workflows>
  - Examples in referenced sections are more likely to be executed
  - Concrete examples provide execution blueprint
- Alternative considered: Examples in 06_nudges.md
- Rejected because: nudges.md is for templates, not execution flow
