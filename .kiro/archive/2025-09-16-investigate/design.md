# Design Document - `/hm:investigate` Slash Command

## 概要

`/hm:investigate` コマンドは、プロジェクトの技術的側面を体系的に調査し、構造化された調査結果を生成するslash commandです。steering、codebase、Context7、webから自動優先度で情報を収集し、インタラクティブな対話を通じて調査を深化させます。

## Slash Command 仕様書 (investigate.md)

````markdown
---
name: investigate
description: "Comprehensive technical investigation with multi-source research and interactive refinement"
category: workflow
complexity: advanced
mcp-servers: [context7, sequential-thinking]
personas: [analyzer, architect]
allowed-tools: Read, Write, MultiEdit, Grep, Glob, Task, WebSearch, mcp__context7__*, mcp__sequential-thinking__*
argument-hint: "[--topic [name]] [--for requirements|design]"
---

# /hm:investigate - Technical Investigation Tool

## Triggers
- Technical research needed for requirements or design documents
- Deep dive into specific technical areas or problems
- Codebase exploration for implementation patterns
- Architecture and design decision investigation

## Usage
```
/hm:investigate [--topic <name>] [--for requirements|design]
```
- `--topic <name>`: Resume/update existing topic by name
- `--for`: Link investigation to <kiro_requirements_path> or <kiro_design_path>

## Key Patterns
- **Topic Resolution**: --topic <name> → resume/update existing | no --topic → new investigation
- **Topic Analysis**: User input → title generation → scope determination
- **Depth Detection**: Simple question → standard depth | Complex/multi-system → deep investigation
- **Source Priority**: steering scan → codebase search → Context7 docs → web (automatic)
- **Format Detection**: Code → Technical Pattern | System → Architecture Flow | Issue → Problem Analysis
- **Confidence Scoring**: Source trust × content match × recency = confidence level
- **Interactive Loop**: Investigate → Present → Refine → Document

## Boundaries
**Will:**
- Create new investigation section when no --topic flag provided
- Resume/update existing topic section when --topic <name> matches existing topic
- Automatically prioritize sources (steering > codebase > Context7 > web)
- Launch parallel Task agents for comprehensive investigation
- Save to <kiro_investigation_path> after each investigation round
- Link findings to <kiro_requirements_path> or <kiro_design_path> when --for flag present
- Calculate and display confidence scores for findings
- Maintain investigation history and corrections within same topic section

**Will Not:**
- Create new section when --topic <name> is provided (always resume/update)
- Override automatic source prioritization (no manual source selection)
- Replace existing investigation sections (always append or update)
- Mix different topics in the same section
- Generate speculative technical details without evidence
- Continue to new topic within same command invocation
- Skip source verification or confidence scoring

## Tool Coordination
**Claude Code Tools:**
- **Read**: Load existing <kiro_investigation_path> for continuity
- **Write/MultiEdit**: Save investigation findings progressively
- **Grep/Glob**: Search codebase for patterns and implementations
- **Task**: Spawn parallel investigation agents

**MCP Integration:**
- **Context7**: Official documentation and best practices lookup
- **Sequential-thinking**: Complex analysis and systematic investigation
- **WebSearch**: Fallback for latest information and community solutions

## Document Template

### Investigation Structure
```markdown
# Investigation - [Spec Name]

## Topic: [Auto-generated Topic Title]
**Confidence**: [level] ([percentage]%)
**Primary Sources**: steering:file.md ([%]), src/path/* ([%]), Context7:lib ([%])

### Summary
[1-2 line executive summary of findings]

### Root Cause / Core Finding
[Main discovery - flexible format based on content type]
- Architecture diagrams (mermaid)
- Code implementations
- System designs
- Data flows

### Evidence
[Source-prioritized evidence with attribution]

**From Steering (file.md:lines)**:
- [Key findings from project documentation]

**From Codebase (path/file.ts:lines)**:
```language
// Actual implementation code
```

**From Context7 (library docs)**:
- [Official patterns and best practices]

**From Web (as last resort)**:
- [Recent developments or community solutions]

### Recommendations
1. [Actionable recommendation]
2. [Implementation approach]
3. [Consideration or trade-off]

### Investigation Notes
- **Update [time]**: [Additional findings or corrections]
- **Correction**: [Fixed understanding or updated information]
- **Note**: [Important observations or caveats]
```

## Behavioral Flow

1. **Initialize**: Parse arguments and load existing investigation
   - Check for existing <kiro_investigation_path>
   - Determine mode: standalone or linked (--for)
   - If --topic <name>: search for matching section to resume/update
   - If no --topic: prepare for new investigation
   - Load existing topics for reference

2. **Topic Gathering**: Determine investigation topic
   - If --topic <name> provided: Resume specified existing topic
   - Otherwise, ask for new topic:
   ```
   > 🔍 What would you like to investigate?
   > [Provide specific technical question or area]
   ```

   **[STOP HERE AND WAIT FOR USER INPUT - DO NOT PROCEED]**

   - Auto-generate concise title (2-4 words) from user input
   - Create new section for this investigation

3. **Parallel Investigation**: Launch Task agents with plan display
   ```
   > 🚀 Investigation Plan for "[Topic]":
   >
   > Launching parallel investigators:
   > • [Steering Analyzer] Check steering files for patterns
   > • [Code Explorer] Search implementation in codebase
   > • [Docs Researcher] Query Context7 for best practices
   > • [Web Searcher] Find recent solutions and updates
   >
   > Priority: steering > codebase > Context7 > web
   ```

   - Execute parallel Task agents
   - Aggregate findings with source priority
   - Calculate confidence scores

4. **Progressive Documentation**: Save findings immediately
   - Auto-select format based on content type
   - Append to <kiro_investigation_path>
   - Display save confirmation

   ```
   > 📝 Investigation saved to <kiro_investigation_path>
   > Topic: "[Title]" (Section #[n])
   > Confidence: [level] ([percentage]%)
   ```

5. **Interactive Continuation**: Topic refinement loop
   ```
   > 🔄 Continue investigating "[Topic]"?
   > - [Y/Enter]: Deepen current topic
   > - [n/done]: Finish investigation
   >
   > Or provide specific follow-up question:
   ```

   **[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]**

   - Y/follow-up → Update same section with new findings (return to step 3)
   - n/done → Proceed to step 6

6. **Finalization**: Link to other documents if --for present
   - If --for requirements: Extract relevant → Update <kiro_requirements_path>
   - If --for design: Extract architectural → Update <kiro_design_path>

   ```
   > ✅ Investigation complete
   > • Topics investigated: [count]
   > • Overall confidence: [level] ([percentage]%)
   > [if --for] • Updated: [document].md
   ```

Key behaviors:
- **Source Automation**: Always use steering > code > docs > web priority
- **Parallel Execution**: Multiple Task agents investigate simultaneously
- **Progressive Save**: Write to <kiro_investigation_path> after each round, not just at end
- **Session Scope**: Each command invocation handles one topic (with deepening)
- **Topic Management**: --topic <name> resumes existing, no flag creates new
- **Section Management**: Same topic updates section, new command creates new section
- **Confidence Tracking**: Display trust level for all findings
- **History Preservation**: Maintain investigation notes and corrections within topic

## Examples

### Example 1: New Topic Investigation
```
/hm:investigate

> 🔍 What would you like to investigate?
> [Provide specific technical question or area]

[STOP AND WAIT]

User: "JWT authentication implementation"

> 🚀 Investigation Plan for "JWT Authentication":
> [Parallel agents launch...]

> 📝 Investigation saved to <kiro_investigation_path>
> Topic: "JWT Authentication" (Section #1)
> Confidence: High (90%)

> 🔄 Continue investigating "JWT Authentication"?

User: Y, what about refresh token rotation?

> 📝 Updated investigation for "JWT Authentication"
> Added findings about refresh token rotation
> Confidence: High (92%)

> 🔄 Continue investigating "JWT Authentication"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1
> • Overall confidence: High (92%)
```

### Example 2: Resume Existing Topic
```
/hm:investigate --topic "JWT Authentication" --for requirements

> 📝 Resuming existing topic "JWT Authentication" from investigation.md
> Previous confidence: High (92%)

> 🚀 Continuing investigation for "JWT Authentication":
> [Parallel agents focus on gaps/updates...]

> 📝 Updated investigation for "JWT Authentication"
> Topic: "JWT Authentication" (Section #1 - Updated)
> Confidence: High (95%)

> 🔄 Continue investigating "JWT Authentication"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1 (resumed)
> • Overall confidence: High (95%)
> • Updated: requirements.md (Technical Requirements section)
```

### Example 3: Multiple Separate Investigations
```
# First investigation
/hm:investigate

> 🔍 What would you like to investigate?

User: "API rate limiting"

[Investigation process...]

> 🔄 Continue investigating "API Rate Limiting"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1
> • Overall confidence: High (85%)

# Second investigation (new command)
/hm:investigate

> 🔍 What would you like to investigate?

User: "caching strategy"

> 🚀 Investigation Plan for "Caching Strategy":
> [New parallel investigation...]

> 📝 Investigation saved to <kiro_investigation_path>
> Topic: "Caching Strategy" (Section #2)
> Confidence: Medium (75%)

> 🔄 Continue investigating "Caching Strategy"?

User: done

> ✅ Investigation complete
> • Topics investigated: 1
> • Overall confidence: Medium (75%)

# Third investigation (resuming first topic)
/hm:investigate --topic "API Rate Limiting"

> 📝 Resuming existing topic "API Rate Limiting" from investigation.md
> Previous confidence: High (85%)

[Continue investigation with new angle...]
```
````

## 設計の解説

### 1. **コマンド構造の設計思想**

#### シンプルなトピック管理インターフェース
```bash
/hm:investigate [--topic <name>] [--for requirements|design]
```
- **フラグなし**: 新規調査を開始
- **`--topic <name>`フラグ**: 既存トピックを再開・更新
- **`--for`フラグ**: 他のKiroドキュメントとの明示的な連携
- **sourcesフラグの意図的排除**: 自動優先度システムによる一貫性確保

### 2. **自動ソース優先度システム**

#### 信頼度階層の設計
```
steering (100%) > codebase (90%) > Context7 (80%) > web (70%)
```

**設計根拠**:
1. **steering**: プロジェクト固有の決定事項・合意事項として絶対的優先
2. **codebase**: 実装の現実を反映する最も信頼できる一次情報源
3. **Context7**: 公式ドキュメント・ベストプラクティスの権威ある情報
4. **web**: 最新だが検証されていない可能性のある補完的情報

#### 透明性の確保
```markdown
**Primary Sources**: steering:tech.md (60%), src/auth/* (30%), Context7 (10%)
```
各ソースの貢献度を可視化し、調査結果の信頼性を明確化。

### 3. **Problem Analysisベースの統一フォーマット**

#### 構造の一貫性と柔軟性の両立
- **Summary**: エグゼクティブサマリー（意思決定者向け）
- **Root Cause / Core Finding**: 調査の核心（形式は内容に応じて柔軟）
  - アーキテクチャ → Mermaidダイアグラム
  - 実装詳細 → コードスニペット
  - システム設計 → フロー図
  - 概念説明 → 構造化テキスト
- **Evidence**: ソース優先度順の証拠提示
- **Recommendations**: 実践可能なアクションアイテム
- **Investigation Notes**: 調査の進化と学習の記録

### 4. **インタラクティブな深化プロセス**

#### 継続的対話による知識の深化
```
初回調査 → 即座に保存 → 「続けますか？」→ 追加質問 → 同一セクション更新
```

**設計メリット**:
- **段階的深化**: ユーザーの理解度に応じて調査を深められる
- **進捗の保護**: 各段階で保存され、作業が失われない
- **知識の集約**: 同一トピックの情報が一箇所に整理される

### 5. **並列調査アーキテクチャ**

#### Task Agentによる効率化
```yaml
並列実行:
  - Steering Analyzer: プロジェクト固有知識の探索
  - Code Explorer: 実装パターンの発見
  - Docs Researcher: 公式パターンの確認
  - Web Searcher: 最新ソリューションの調査
```

**並列化の利点**:
- **時間効率**: 4つの調査を同時実行で大幅な時間短縮
- **専門化**: 各エージェントが得意分野に集中
- **統合知識**: 複数視点からの包括的理解

### 6. **累積型ナレッジ構築**

#### Investigation Notesの重要性
```markdown
### Investigation Notes
- **Update 15:45**: Redis実装の詳細が判明
- **Correction**: 当初の理解を修正 - ハイブリッド方式
- **Security Note**: CSRF対策の必要性を発見
```

**設計意図**:
- **透明性**: 調査の進化過程を記録
- **学習**: 間違いからの学びを資産化
- **信頼性**: なぜその結論に至ったかの根拠を保持

### 7. **他コマンドとの連携設計**

#### 役割分担の明確化
```yaml
/hm:requirements:
  責任: ユーザー要件の収集と構造化
  生成: requirements.md (0-70% completeness)

/hm:investigate:
  責任: 技術的調査と分析
  生成: <kiro_investigation_path>
  更新: <kiro_requirements_path> or <kiro_design_path> (--for flag)

/hm:design:
  責任: アーキテクチャと実装設計
  参照: investigation.md
```

### 8. **Key Patternsの設計**

#### 変換ルールとしての機能
- **Topic Analysis**: 自由形式の質問 → 構造化された調査
- **Source Priority**: 複数ソース → 優先度による統合
- **Format Detection**: 内容タイプ → 最適な表現形式
- **Confidence Scoring**: 複数要因 → 信頼度スコア
- **Interactive Loop**: 単発調査 → 継続的深化

### 9. **セクション管理戦略**

#### 情報の組織化原則
- **トピック単位**: 各トピックが独立したセクション
- **時系列管理**: 作成日時と更新日時の記録
- **継続性**: `--topic <name>`で同一セクションを再開・深化
- **新規追加**: フラグなし実行は常に新セクションとして追加
- **1コマンド1トピック**: コマンド実行ごとに単一トピックに集中

### 10. **信頼度スコアリングシステム**

#### 複合的な信頼度評価
```yaml
confidence_factors:
  source_trust:      # steering=1.0, code=0.9, Context7=0.8, web=0.7
  content_match:     # キーワードと内容の一致度
  recency:          # 情報の新しさ
  consistency:      # 複数ソース間の一貫性
```

この設計により、`/hm:investigate`は技術調査の中核ツールとして機能し、他のKiroコマンドと連携しながら、プロジェクトの技術的理解を体系的に深化させます。

## 追加設計: 段階的ワークフロー

### 三段階ワークフロー
```bash
# Stage 1: 要件収集
/hm:requirements --type prd
# → ユーザーとの対話で要件定義
# → 完成度: 0-70%

# Stage 2: 技術調査
/hm:investigate --topic --for requirements
# → 技術的な実現可能性調査
# → <kiro_requirements_path>の[TBD]セクション更新

# Stage 3: 設計
/hm:design --based-on requirements,investigation
# → アーキテクチャと実装設計
# → 調査結果を参照した技術設計
```

### 調査フォーマットの適応性
```yaml
format_selection:
  implementation_question: Technical Pattern
  architecture_question: Architecture Flow + Mermaid
  performance_issue: Problem Analysis + Metrics
  concept_exploration: Knowledge Summary
  debugging_session: Root Cause Analysis
```

この設計により、`/hm:investigate`は柔軟かつ強力な調査ツールとして、Kiroワークフローの中核を担います。