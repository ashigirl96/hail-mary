# Design v2 (Revised)

## Meta
- **Completeness**: 90%
- **Requirements**: Implementation timeline management - NO Linear Workflow準拠版
- **Architecture Scope**: Pattern Router Framework + Slash Commands
- **Revision**: v1からの主要変更 - 強制的表現削除、説明的記述へ、シンプル化

## Overview

**As-Is**:
- design完了後のタスク分解フローが不明確
- State Trackingに実装進捗が表示されない
- タスク分解がパターンとして認識されていない

**To-Be**:
- ユーザーが実装に進むサインを出したら、シンプルな提案
- State Trackingでtasks.md#Timeline進捗を追跡
- /hm:timelineで明示的なTimeline操作も可能
- **NO Linear Workflow**: 強制せず、ガイドし、adaptive conversation重視

## Design Principles (v1からの変更)

**削除したもの**:
- ❌ MUST, MANDATORY, PROTOCOL等の強制的表現
- ❌ "After Design Complete (MANDATORY PROTOCOL)"
- ❌ 長い会話例（Linear Workflowを暗示）
- ❌ Timeline Planning Protocolという特別視
- ❌ /hm:design.mdの詳細なBehavior説明

**新しいアプローチ**:
- ✅ 説明的であって強制的でない（"User may...", "When user signals..."）
- ✅ シンプルなnudging（ファイル順序の提案のみ）
- ✅ Gatesは警告レベル（BLOCKではない）
- ✅ Slash commandはタグ参照のみ（詳細はsystem prompt）
- ✅ フレームワークを信じる（過度な制御なし）

## Design

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/06_nudges.md

**Purpose**: 実装文脈でのシンプルな提案テンプレート

**Current Problem**:
- v1では長いphase詳細を提示（Linear Workflowの暗示）
- 詳細すぎて会話的でない

**Solution**: ファイル順序の提案のみ、詳細はTimeline追加時に決める

````markdown
### Implementation Context Detected

Ready to implement? Files from design.md:
- backend → API → frontend (suggested flow)

Shall we plan the implementation order?
````

**Rationale**:
- Nudgingは「きっかけ」のみ提供
- 詳細なタスクはユーザーと対話しながらTimeline追加時に決める
- 最初から全部提示しない = adaptive conversation

**Evidence**: 00_philosophy.md "Pattern Recognition over Process"

---

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/04_workflows.md

**Purpose**: Timeline planningの流れを説明的に記述（強制ではない）

**Current Problem**:
- v1では"MANDATORY PROTOCOL"と強制的
- MUSTだらけでNO Linear Workflowに反する

**Solution**: 既存の"After Requirements Complete"と同じトーンで説明的に記述

````markdown
**When Implementation Planning Begins**:
1. User signals readiness to implement (e.g., "let's code", "start building", `/hm:timeline`)
2. Read design.md to identify implementation files
3. Suggest general flow (e.g., backend → API → frontend)
4. Discuss with user interactively:
   - User may adjust order
   - User may group into phases
   - User may add or remove files
   - User may specify different granularity
5. Add agreed plan to tasks.md#Timeline
6. Update State Tracking: tasks.md#Timeline = pending (0%)
````

**Rationale**:
- "MANDATORY PROTOCOL"削除
- 手順は「こうなる」という説明
- "User may..."でadaptive conversationを許容
- 強制ではなくガイダンス

**Note**: "After Design Complete"という名前は避ける。なぜなら：
- design.md書込直後 ≠ 設計完了
- ユーザーと設計対話が続く可能性
- 実装に進むのはユーザーがサインを出した時

**Evidence**: 00_philosophy.md "Routing without Control"

---

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/05_gates.md

**Purpose**: Timeline planning時の軽い警告ゲート（BLOCKではない）

**Current Problem**: v1ではgatesを無視していた（誤り）

**Solution**: 警告レベルのゲートを追加

````markdown
## Timeline Planning Gates

**Timeline Planning without Design**:
- Check: design.md exists and has content
- Action: ⚠️ WARNING with guidance (not BLOCK)
- Message: "No design.md found. Create design first with `/hm:design`, or describe what you want to implement?"
- Applies to: `/hm:timeline` execution

**Rationale**: ガイダンスであってブロックではない。代替手段を提示（口頭で説明してもOK）。
````

**Rationale**:
- designなしでtimelineは不自然 → 警告必要
- しかしBLOCKしない → NO Linear Workflow
- 代替手段提示（口頭説明でもOK）→ 柔軟性

**Evidence**: 01_principles.md "Autonomy with Safety"

---

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/03_patterns.md

**Purpose**: /hm:timeline パターンを追加（gatesコンポーネント含む）

**Current Problem**: v1ではgatesを除外していた

**Solution**: EXPLICIT Patternsテーブルに追加、gatesを含める

````markdown
| User Pattern | Action | Strategy Output |
|-------------|--------|-----------------|
| "/hm:requirements", "Create requirements" | Create/Update | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "/hm:investigate", "investigate", "research" | Append | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "/hm:design", "design", "architecture" | Create (validated) | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
| "/hm:timeline", "plan implementation" | Timeline Planning | `{class: "EXPLICIT", strategy: "command", components: ["hub", "gates", "workflows", "document", "nudges"]}` |
````

**Routing Decision Example**:

````markdown
Input: "/hm:timeline"
→ Class: EXPLICIT
→ Confidence: 1.0
→ Strategy: command
→ Components: ["hub", "gates", "workflows", "document", "nudges"]
→ Route to: Command Pipeline

Input: "Let's start implementing"
→ Class: EXPLICIT (natural language pattern match)
→ Confidence: 0.8
→ Strategy: command
→ Components: ["hub", "gates", "workflows", "document", "nudges"]
→ Route to: Command Pipeline
````

**Evidence**: 03_patterns.md "Pattern class determines entire routing flow"

---

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/02_hub.md

**Purpose**: State Trackingにtasks.md#Timeline行を追加、Timeline例にphase構造を反映

**Current State**: State Trackingに実装進捗が表示されない

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
- [x] Implementation plan agreed with user
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
```
````

**Rationale**:
- requirements/investigation/design = Source documents
- tasks.md#Timeline = Implementation checklist
- Both are trackable states, different abstraction levels
- No circular reference: Timeline is data, State Tracking is meta-information

**Evidence**: 02_hub.md "Single Source of Truth"

---

### .claude/commands/hm/timeline.md (New File)

**Purpose**: オプショナルなTimeline操作コマンド

**Rationale**:
- requirements/investigate/designはドキュメント作成
- /hm:timelineはtasks.md操作
- 性質が異なるため別コマンドとして正当

**Implementation**:

````markdown
---
name: timeline
description: "Plan implementation timeline"
allowed-tools: Read, Write, MultiEdit
---

# /hm:timeline

Plan implementation order from design.md or discuss implementation approach.

Refer to system prompt sections:
- <kiro-philosophy> for reactive pattern routing
- <kiro-principles> for universal operational rules
- <kiro-workflows> for timeline planning flow
- <kiro-nudges> for implementation suggestions
- <kiro-hub> for Timeline structure and State Tracking
- <kiro-gates> for validation gates

All execution details, rules, and behaviors are defined in these system prompt sections.
````

**Rationale**:
- 詳細な説明削除（Use Cases, Behavior等）
- タグ参照のみ
- 詳細はsystem promptに委譲

**Evidence**: steering-prompt-engineering "XML-based Orchestration Framework"

---

### .claude/commands/hm/design.md (No Change)

**Current State**: 既存のタグ参照形式

**Decision**: 変更不要

**Rationale**:
- 既にsystem prompt参照の形式
- 詳細な"Post-Completion Behavior"は不要
- XML-based Orchestration原則に準拠済み

---

## Implementation Order

1. **06_nudges.md** - シンプル化（最も影響大）
2. **04_workflows.md** - 説明的記述へ変更
3. **05_gates.md** - 警告ゲート追加
4. **02_hub.md** - State Tracking + Timeline構造
5. **03_patterns.md** - /hm:timeline + gates追加
6. **.claude/commands/hm/timeline.md** - 新規作成

**Note**: mod.rsの変更は不要（既存ファイル編集 + .claude/commands/追加のみ）

---

## Validation Checklist (NO Linear Workflow準拠)

- ✅ NO Linear Workflow: 強制的表現なし、adaptive conversation重視
- ✅ Pattern Recognition: /hm:timelineをパターンとして認識
- ✅ Conditional Hub Access: Command Pipelineでtasks.md更新
- ✅ Routing without Control: ガイダンス提供、強制しない
- ✅ Evidence-Based Progress: design.md → Timeline への明確なトレーサビリティ
- ✅ Autonomy with Safety: 警告ゲート（BLOCKではない）
- ✅ Single Source of Truth: tasks.md#Timelineが実装状態の単一権威
- ✅ Simplicity: 長い会話例なし、シンプルなnudging
- ✅ Trust the Framework: 過度な制御なし、Claudeの能力を信じる

---

## Architecture Decision Records (v2)

**ADR-001: Why NO "MANDATORY PROTOCOL"?**
- Decision: 説明的記述のみ、強制的表現を完全排除
- Rationale:
  - 00_philosophy.md "NO Linear Workflow"に準拠
  - 開発は対話的でadaptive
  - MUSTは「こうしなければならない」を暗示 → Linear Workflow
- Alternative considered: MUST指示でLost in the Middle解決
- Rejected because: フレームワークの核心哲学に反する

**ADR-002: Why simple nudging template?**
- Decision: ファイル順序のみ提示、詳細なタスクリストは削除
- Rationale:
  - Nudgingは「きっかけ」のみ提供
  - 詳細はTimeline追加時にユーザーと対話
  - 長い詳細はLinear Workflowを暗示
- Alternative considered: 完全なphase詳細を事前提示
- Rejected because: 会話的でない、adaptive conversationを阻害

**ADR-003: Why WARNING instead of BLOCK?**
- Decision: Timeline planning時のdesign不在は警告のみ
- Rationale:
  - Routing without Control - 強制せずガイド
  - 代替手段提示（口頭説明でもOK）
  - ユーザーの自由を制限しない
- Alternative considered: design必須でBLOCK
- Rejected because: 柔軟性を失う、NO Linear Workflowに反する

**ADR-004: Why no conversation examples in workflows?**
- Decision: 長い会話例を削除、簡潔な手順説明のみ
- Rationale:
  - 会話例は「こうあるべき」という固定観念を生む
  - Pattern-Based Routingを信じる
  - Claudeの自然言語理解能力で十分
- Alternative considered: 多ターン会話例で実行イメージ具体化
- Rejected because: Linear Workflowを暗示、過度な制御

**ADR-005: Why tasks.md#Timeline in State Tracking?**
- Decision: Add tasks.md#Timeline as a row in State Tracking Table
- Rationale:
  - requirements/investigation/design = Source documents
  - tasks.md#Timeline = Implementation progress
  - Both are states that need tracking
  - No circular reference: different abstraction levels
- Alternative considered: Separate "Implementation Progress" section
- Rejected because: Increases complexity, violates Single Source of Truth

**ADR-006: Why "When Implementation Planning Begins" not "After Design Complete"?**
- Decision: イベント名を変更
- Rationale:
  - design.md作成直後 ≠ 設計完了
  - ユーザーと設計対話が続く可能性（"なぜこの設計？"等）
  - 実装に進むのはユーザーがサインを出した時
- Evidence: 実際の開発フロー観察

---

## Comparison: v1 vs v2

| 項目 | v1 (強制的) | v2 (説明的) |
|-----|-----------|-----------|
| Workflow記述 | MANDATORY PROTOCOL | When ... Begins |
| 表現 | MUST, SHALL, ALWAYS | User may, Discuss interactively |
| Nudging | 長いphase詳細 | ファイル順序のみ |
| Gates | BLOCKまたは無視 | WARNING with guidance |
| 会話例 | 多ターン詳細 | なし（信頼） |
| /hm:design.md | 詳細なBehavior | タグ参照のみ |
| 哲学 | Lost in the Middle解決重視 | NO Linear Workflow準拠 |

---

## Summary

v2では、v1の「Lost in the Middle問題を力で解決する」アプローチから、「フレームワークの哲学に忠実」なアプローチへ転換しました。

**核心的変更**:
1. 強制から提案へ
2. 詳細から簡潔へ
3. プロトコルからガイダンスへ
4. ブロックから警告へ
5. 会話例削除でadaptive conversation重視

これにより、NO Linear Workflow、Pattern Recognition over Process、Routing without Controlという核心哲学に完全準拠した設計となります。
