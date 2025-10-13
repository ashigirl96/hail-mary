# Design: Review Pipeline for Pattern Router Framework

## Meta
- **Completeness**: 100%
- **Requirements**: Pattern Router Framework機能拡張 - ユーザーによる対話的レビューとBefore/After Protocol再利用
- **Architecture Scope**: Pattern Router Framework (System Prompt)

## Overview

**As-Is**:
- `/spec:requirements`や`/spec:design`は即座に実行され、ファイルに書き込まれる
- ユーザーが内容を事前に確認・改善する機会がない
- 誤った方針や不足がそのまま永続化される可能性

**To-Be**:
- `--review`フラグで軽量なReview Pipelineを実行
- Draft生成 → 自然言語での対話的レビュー → 改善ループ → Command Pipeline移行
- Before/After Protocolを完全再利用してDRY原則遵守
- Slash command変更なし、System prompt側で完結

## Design

### Architecture Decision

**5番目のパイプラインとして独立させる理由:**

1. **既存Command Pipelineの変更を最小化**:
   - Command Pipelineは既に安定稼働中
   - Review機能を内部コンポーネントとして組み込むと複雑化
   - 独立パイプラインで完全分離

2. **オプトイン設計**:
   - デフォルト動作は現状維持（既存ユーザーへの影響ゼロ）
   - --reviewフラグで明示的に有効化
   - 段階的導入が可能

3. **Before/After Protocol再利用**:
   - Review Pipeline完了後、Command Pipelineに移行
   - hub、gates、workflows、nudgesを全て再利用
   - 重複実装なし、メンテナンス性向上

### Component Flow

```
Input: /spec:requirements --review

Review Pipeline (NEW):
┌─────────────────────────────────────────┐
│ patterns: Detect EXPLICIT_REVIEW        │
│   ↓                                     │
│ review: Generate draft (in-memory)      │
│   ↓                                     │
│ nudges: Natural language options        │
└─────────────────────────────────────────┘
         │
         │ User: "保存して続行"
         ↓
Command Pipeline (EXISTING):
┌─────────────────────────────────────────┐
│ hub: Read tasks.md                      │
│   ↓                                     │
│ gates: Validate prerequisites           │
│   ↓                                     │
│ workflows(BEFORE): Timeline update      │
│   ↓                                     │
│ document: Write approved_draft          │
│   ↓                                     │
│ workflows(AFTER): Complete protocol     │
│   ↓                                     │
│ nudges: Next steps                      │
└─────────────────────────────────────────┘
```

### File Structure

**変更ファイル（3個のみ）:**

```
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/
├── 03_patterns.md          # EXPLICIT_REVIEW pattern追加
├── 04_workflows.md         # Review Pipeline section追加
└── 06_nudges.md           # Natural language templates追加
```

**変更不要:**
- index.md (XMLタグ追加不要)
- mod.rs (Rust code変更不要)
- All slash commands (1行も変更不要)

## Implementation Details

### 1. Pattern Recognition (03_patterns.md)

**追加内容:**

````markdown
## Pattern Classification System

| Pattern Class | Characteristics | Routing Strategy |
|--------------|-----------------|------------------|
| EXPLICIT_REVIEW | EXPLICIT + --review flag | Review Pipeline |

**EXPLICIT_REVIEW Patterns:**

| User Pattern | Strategy Output |
|-------------|-----------------|
| "/spec:requirements --review" | `{class: "EXPLICIT_REVIEW", strategy: "review", components: ["patterns", "review", "nudges"]}` |
| "/spec:design --review" | Same |
| "/spec:investigate --review" | Same |

**Routing Decision Example:**

Input: "/spec:requirements --review"
→ Class: EXPLICIT_REVIEW
→ Strategy: review
→ Components: ["patterns", "review", "nudges"]
→ Route to: Review Pipeline
````

**実装ポイント:**
- `--review`フラグの検出はpatternsコンポーネントの責務
- Base command（`/spec:requirements`）のコンテキストを保持
- Review Pipeline完了後のCommand Pipeline移行に必要な情報を保存

### 2. Review Pipeline (04_workflows.md)

**追加内容:**

````markdown
### Review Pipeline (EXPLICIT_REVIEW class)
```
Input → patterns → review → nudges → [User Decision] → Command Pipeline
```

**Characteristics:**
- Opt-in with --review flag
- Draft generation without persistence
- Natural language dialogue
- Hands off to Command Pipeline for execution
- Lightweight preview and refinement

**Component Responsibilities:**
- **patterns**: Detect EXPLICIT_REVIEW (base command + --review flag)
- **review**: Execute command logic without writing, generate draft, analyze direction
- **nudges**: Present draft summary and natural language action options

**Review Protocol:**
1. Generate draft in memory (ephemeral)
2. Analyze direction and concerns
3. Present natural language summary
4. Wait for user response (natural language)
5. Parse user intent:
   - Save intent → Handoff to Command Pipeline
   - Refine intent → Re-enter review component
   - Add intent → Incorporate additions, loop back
   - Cancel intent → Clean exit

**Handoff to Command Pipeline:**
When user approves:
1. Exit Review Pipeline
2. Enter Command Pipeline with:
   - Original command (without --review flag)
   - Approved draft content
   - Command context preserved
3. Execute full Command Pipeline:
   - hub → gates → workflows(BEFORE) → document → workflows(AFTER) → nudges
4. Document component uses approved draft (skips generation)
5. All protocols (BEFORE/AFTER) execute normally

**Key Behaviors:**
- Stateless until approved: No hub updates during review
- Clean cancellation: Exit without side effects
- Protocol reuse: Command Pipeline handles all persistence
- Natural dialogue: No rigid command syntax
````

**実装ポイント:**
- Review Pipelineは一時的な状態のみ保持（ephemeral）
- Command Pipeline移行時、approved_draftをconversation contextから取得
- document componentで条件分岐: approved_draft存在時は生成スキップ

### 3. Natural Language Nudges (06_nudges.md)

**追加内容:**

````markdown
## Review Pipeline Templates (Conversational)

**After Draft Generation:**

```
📋 {Document Type} Draft Ready

Here's the direction I'm taking:
• {Key point 1 from draft}
• {Key point 2 from draft}
• {Key point 3 from draft}
• {Key point 4 from draft}

A few things to consider:
• {Concern or suggestion 1}
• {Concern or suggestion 2}
• {Concern or suggestion 3}

Would you like to proceed, or shall we refine this together?
```

**Natural Language Response Parsing:**

**Save Intent Detection:**
Keywords: "save", "proceed", "go ahead", "looks good", "continue", "yes", "保存", "続行", "いいね", "はい"
Action: Handoff to Command Pipeline with approved draft

**Refine Intent Detection:**
Keywords: "refine", "improve", "think about", "reconsider", "改善", "考える", "見直す", "もう少し"
Action: Return to review component for dialogue

**Add Intent Detection:**
Pattern: "add [something]", "include [something]", "can we add", "追加", "含める"
Action: Parse what to add, incorporate, loop back to review

**Cancel Intent Detection:**
Keywords: "cancel", "stop", "nevermind", "start over", "キャンセル", "やり直し", "やめる"
Action: Clean exit without any persistence

**Examples by Document Type:**

**Requirements Review:**
```
📋 Requirements Draft Ready

Here's the direction I'm taking:
• User authentication with email and password
• Password reset via email verification link
• JWT-based session management
• Basic role-based access control

A few things to consider:
• Should we specify password complexity requirements?
• Email verification flow for new accounts isn't detailed yet
• Rate limiting for login attempts might be important
• OAuth integration scope unclear

Would you like to proceed, or shall we refine this together?
```

**Design Review:**
```
📋 Design Draft Ready

Here's the direction I'm taking:
• JWT service using jose library (based on investigation)
• Bcrypt for password hashing (aligns with codebase)
• Session management with Redis cache
• RESTful API endpoints for auth operations

A few things to consider:
• Error handling strategy for token expiration not specified
• Refresh token rotation mechanism could be detailed
• Database migration for user table missing
• Integration tests approach undefined

Would you like to proceed, or shall we refine this together?
```
````

**実装ポイント:**
- テンプレート的な構造を避け、自然な会話調
- コマンド構文（[S]/[R]/[A]/[C]）を使わない
- Intent parsingで柔軟に対応
- Multi-turn dialogueサポート

### 4. Document Component Enhancement

**既存document componentの条件分岐追加:**

```
Document Component Logic:

IF conversation context contains approved_draft:
  content = approved_draft from previous Review Pipeline
  SKIP generation step
  WRITE content to file
ELSE (normal Command Pipeline):
  content = generate new content using command logic
  WRITE content to file
```

**実装方法:**
- Claudeの会話コンテキスト理解を活用
- 「直前のReview Pipelineで承認されたdraftを使う」という暗黙の指示
- 明示的なstate managementは不要（会話の流れから自然に判断）

### State Management

**Ephemeral State (Review Pipeline中):**
- approved_draft: String
- original_command: String
- command_context: Map

**Persistent State (Command Pipeline後):**
- tasks.md (hub経由)
- requirements.md / design.md / investigation.md (document経由)

**重要:** Review Pipeline完了までは一切の永続化なし。ユーザーが承認した時点でCommand Pipelineに移行し、既存のhub/workflows protocolが永続化を担当。

## Trade-offs and Decisions

### Decision 1: 独立パイプライン vs Command Pipeline内統合

**選択:** 独立パイプライン

**理由:**
- ✅ 既存Command Pipelineを変更しない
- ✅ オプトイン設計が自然
- ✅ 責任分離が明確
- ✅ テスト独立性

**Trade-off:**
- ⚠️ パイプライン数が増加（4→5）
- ⚠️ Pipeline間のhandoffメカニズムが必要

**判断:** Trade-offを上回る利点。handoffはconversation contextで自然に実現可能。

### Decision 2: テンプレート詳細度

**選択:** シンプルな方針提示のみ

**理由:**
- ✅ 実装が簡単
- ✅ 自然な対話が可能
- ✅ メンテナンス負荷低い
- ✅ ユーザー提案に基づく

**Trade-off:**
- ⚠️ Quality Score計算なし
- ⚠️ Dimension分析なし

**判断:** 過剰な構造化を避け、本質に集中。必要なら将来追加可能。

### Decision 3: デフォルト動作

**選択:** オプトイン（--reviewフラグ必須）

**理由:**
- ✅ 既存ワークフローへの影響ゼロ
- ✅ 段階的導入が可能
- ✅ エキスパートの高速ワークフローを妨げない
- ✅ 後方互換性保証

**Trade-off:**
- ⚠️ 新規ユーザーがReview機能を見落とす可能性

**判断:** 発見性はドキュメントでカバー。既存ユーザー保護を優先。

### Decision 4: Before/After Protocol再利用

**選択:** Command Pipelineに完全移行して再利用

**理由:**
- ✅ DRY原則遵守
- ✅ メンテナンス性向上
- ✅ 一貫性保証
- ✅ テスト済みコード活用

**Trade-off:**
- ⚠️ Review Pipeline → Command Pipeline handoffが必要

**判断:** Handoffコストを上回るメンテナンス性。これがユーザー提案の核心。

## Testing Strategy

### Phase 1: Manual Testing
```
$ hail-mary code
→ Select spec
→ Claude session start

User: /spec:requirements --review

Expected:
1. Review Pipeline実行
2. Draft生成
3. Natural language nudge表示
4. User: "保存"
5. Command Pipeline移行
6. requirements.md書き込み
7. Timeline更新
8. Post-action実行
```

### Phase 2: Integration Testing
- [ ] 全ドキュメントタイプ（requirements, design, investigation）
- [ ] 各intent（save, refine, add, cancel）
- [ ] Multi-turn refinement
- [ ] Edge cases（曖昧な応答、エラー時）

### Phase 3: Protocol Verification
- [ ] Before Protocol正常実行
- [ ] After Protocol正常実行
- [ ] tasks.md更新確認
- [ ] State Tracking一貫性

## Implementation Phases

### Phase 1: Core (3-5日)
- [ ] Update 03_patterns.md
- [ ] Update 04_workflows.md
- [ ] Update 06_nudges.md
- [ ] Manual testing with `/spec:requirements --review`

### Phase 2: Refinement (3-5日)
- [ ] Multi-turn dialogue
- [ ] Context-aware suggestions
- [ ] Draft regeneration

### Phase 3: Full Support (2-3日)
- [ ] `/spec:design --review`
- [ ] `/spec:investigate --review`

### Phase 4: Polish (2-3日)
- [ ] Intent parsing robustness
- [ ] Edge case handling
- [ ] Performance optimization

**Total: 2-3週間**

## Success Criteria

- [ ] `/spec:requirements --review`でReview Pipeline実行
- [ ] 自然言語nudge表示
- [ ] "保存"でCommand Pipeline移行
- [ ] Before/After Protocol正常実行
- [ ] requirements.md正しく書き込み
- [ ] Slash command変更ゼロ
- [ ] mod.rs変更ゼロ
- [ ] ユーザー評価: "自然で邪魔にならない"

## Links

- Requirements: N/A (この設計はbrainstormから生まれたため)
- Investigation: N/A (既存Pattern Router Frameworkの理解に基づく)
- Related Files:
  - `crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/03_patterns.md`
  - `crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/04_workflows.md`
  - `crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/06_nudges.md`
