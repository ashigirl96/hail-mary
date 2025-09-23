# Design Document - `/hm:interactive-investigate` Slash Command

## 概要

`/hm:interactive-investigate` コマンドは、リアルタイムのユーザー対話を通じて技術調査を実施し、調査結果の保存/破棄を選択できるインタラクティブなslash commandです。`[STOP HERE AND WAIT FOR USER SELECTION]`パターンを活用し、真の対話型体験を提供します。

## Slash Command 仕様書 (interactive-investigate.md)

````markdown
---
name: interactive-investigate
description: "Interactive technical investigation with real-time save/discard options"
category: workflow
complexity: advanced
mcp-servers: [context7, sequential-thinking]
personas: [analyzer, investigator]
allowed-tools: Read, Write, MultiEdit, Grep, Glob, Task, WebSearch, mcp__context7__*, mcp__sequential-thinking__*
argument-hint: "<investigation topic>"
---

# /hm:interactive-investigate - Interactive Investigation Tool

## Triggers
- Technical research requiring user feedback and refinement
- Exploratory investigation where direction may change
- Complex problems needing iterative discovery
- Investigations where save/discard decision comes after seeing results

## Usage
```
/hm:interactive-investigate <investigation topic>
```
- Direct topic specification as argument
- No flags needed - pure interactive experience

## Key Patterns
- **Stop Marker Usage**: [STOP HERE AND WAIT] → genuine user interaction
- **Response Parsing**: Y → save | N → discard | A,question → continue
- **History Preservation**: Full conversation including mistakes → <kiro_investigation>
- **Loop Continuation**: Investigation → Present → Decision → Loop (if A)
- **Session Memory**: Context maintained across investigation rounds
- **Append Strategy**: Timestamp-separated sections in <kiro_investigation>

## Boundaries
**Will:**
- Wait for actual user input at decision points
- Save complete conversation history including corrections
- Allow unlimited investigation rounds within session
- Preserve full context from display to storage
- Append to existing <kiro_investigation> with timestamps
- Track investigation evolution chronologically

**Will Not:**
- Proceed without user selection at stop points
- Lose any displayed information when saving
- Create multiple files for single investigation
- Auto-save without explicit user confirmation
- Mix different investigation topics in same session
- Skip user interaction for convenience

## Tool Coordination
**Claude Code Tools:**
- **Read**: Load existing <kiro_investigation> content
- **MultiEdit**: Append session to <kiro_investigation>
- **Grep/Glob**: Search codebase during investigation
- **Task**: Launch parallel investigation agents

**MCP Integration:**
- **Context7**: Documentation and best practices
- **Sequential-thinking**: Complex analysis flows
- **WebSearch**: Latest information as needed

## Behavioral Flow

1. **Initialize Investigation**: Parse topic from arguments
   ```
   ## Starting Investigation: [topic]

   🔍 Investigating: [parsed topic]
   ```
   - Set up investigation context
   - Initialize session tracking

2. **Conduct Investigation**: Execute research
   - Use appropriate tools and MCP servers
   - Document steps and findings
   - Track any misconceptions
   - Format results clearly

3. **Present Interactive Decision Point**: Show results with options
   ```
   ## Investigation Results

   [Detailed findings displayed here]

   ---
   [Y] Save, [N] Discard, [A, <question>] Continue:
   ```

   **[STOP HERE AND WAIT FOR USER SELECTION - DO NOT PROCEED]**

4. **Handle User Response**: Process selection

   **If response = "Y" or "y":**
   - Read existing <kiro_investigation> for current content
   - Format session with timestamp header:
     - Initial question
     - Investigation steps
     - Misunderstandings/corrections
     - User feedback
     - Final conclusions
   - MultiEdit <kiro_investigation> to append session
   - Display: "✅ Investigation saved to <kiro_investigation>"

   **If response = "N" or "n":**
   - Display: "❌ Investigation discarded"
   - Exit command cleanly

   **If response starts with "A," or "a,":**
   - Extract question: "A, question" → "question"
   - Continue investigation with follow-up question
   - Build upon previous findings (cumulative knowledge)
   - Append new discoveries to session memory (not saved to file yet)
   - Display updated results including all rounds
   - Return to step 3 (Interactive Decision Point)

   **For any other response:**
   - Display: "Invalid input. Please enter Y, N, or A,<question>"
   - Re-display prompt: "[Y] Save, [N] Discard, [A, <question>] Continue:"
   - **[STOP HERE AND WAIT FOR USER SELECTION - DO NOT PROCEED]**
   - Re-evaluate response

5. **Session Completion**: Final status
   ```
   ✅ Investigation complete
   • Rounds conducted: [count]
   • Decision: [saved/discarded]
   • Topic: "[title]"
   ```

Key behaviors:
- **Genuine Interactivity**: Real stops for user decisions
- **Complete Preservation**: Everything shown is saved
- **Flexible Continuation**: Multiple rounds supported
- **Clear Navigation**: Options always visible
- **Robust Parsing**: Handles input variations
- **Chronological Record**: Time-ordered documentation

## Examples

### Example 1: Simple Investigation with Save
```
/hm:interactive-investigate "How does the steering backup system work?"

> 🔍 Investigating: How does the steering backup system work?
>
> [Investigation results...]
>
> [Y] Save, [N] Discard, [A, <question>] Continue:

User: Y

> ✅ Investigation saved to <kiro_investigation>
> • Rounds conducted: 1
> • Decision: saved
> • Topic: "Steering Backup System"
```

### Example 2: Multi-round Investigation
```
/hm:interactive-investigate "Authentication flow in the system"

> [Initial investigation...]
> [Y] Save, [N] Discard, [A, <question>] Continue:

User: A, what about OAuth integration?

> [OAuth investigation added...]
> [Y] Save, [N] Discard, [A, <question>] Continue:

User: A, how does token refresh work?

> [Token refresh investigation added...]
> [Y] Save, [N] Discard, [A, <question>] Continue:

User: Y

> ✅ Investigation saved to <kiro_investigation>
> • Rounds conducted: 3
> • Decision: saved
> • Topic: "Authentication Flow"
```

### Example 3: Investigation with Correction
```
/hm:interactive-investigate "Database connection pooling"

> [Investigation with assumption about c3p0...]
> [Y] Save, [N] Discard, [A, <question>] Continue:

User: A, that's not right, we use HikariCP not c3p0

> [Corrected investigation with HikariCP...]
> [Y] Save, [N] Discard, [A, <question>] Continue:

User: Y

> ✅ Investigation saved to <kiro_investigation> (including correction history)
> • Rounds conducted: 2
> • Decision: saved
> • Topic: "Database Connection Pooling"
```
````

## 設計の解説

### 1. **コマンド構造の設計思想**

#### 純粋なインタラクティブ体験
```bash
/hm:interactive-investigate <topic>
```
- **引数直接指定**: トピックを引数として直接受け取る
- **フラグ不要**: シンプルなインターフェース
- **状態管理不要**: セッション内で完結

### 2. **Stop Markerパターンの活用**

#### 真の対話型実装
```
[STOP HERE AND WAIT FOR USER SELECTION - DO NOT PROCEED]
```

**設計根拠**:
1. **実際の待機**: AIが本当にユーザー入力を待つ
2. **明確な指示**: ユーザーが次のアクションを理解
3. **エラー処理**: 無効入力への対応も含む

### 3. **3つの選択肢アーキテクチャ**

#### ユーザー選択の設計
```yaml
Y: 保存して終了
N: 破棄して終了
A,<question>: 追加調査を継続
```

**設計メリット**:
- **シンプル**: 3つの明確な選択肢
- **柔軟**: A選択で無限の継続が可能
- **安全**: 明示的な保存/破棄の選択

### 4. **完全履歴保存の原則**

#### 表示内容の完全保存
```markdown
保存される内容:
- 初回の質問
- 調査の手順と方法
- 発見した内容
- 間違いや誤解
- ユーザーのフィードバック
- 修正された理解
- 最終的な結論
```

**重要な設計原則**:
- **表示=保存**: ユーザーが見た内容が全て保存される
- **時系列**: 調査の進化が追跡可能
- **透明性**: 間違いも含めて記録

### 5. **ループ継続メカニズム**

#### 調査の深化プロセス
```
調査 → 結果表示 → [STOP] → ユーザー選択
         ↑                      ↓
         └── A (継続) ←─────────┘
```

**ループ設計の利点**:
- **段階的深化**: 理解度に応じた調査
- **文脈維持**: セッション内で情報蓄積
- **柔軟な終了**: いつでも保存/破棄可能

### 6. **セッションメモリ管理**

#### 調査コンテキストの維持
```yaml
session_context:  # Claude Codeのメモリ内で管理
  initial_topic: "Authentication flow"
  rounds:
    - round_1: "Basic auth investigation"
    - round_2: "OAuth integration"
    - round_3: "Token refresh"
  corrections:
    - "Fixed: c3p0 → HikariCP"
  final_understanding: "Comprehensive auth flow"

保存タイミング:
  - 各ラウンド後: メモリ内のみ (ファイル保存なし)
  - ユーザーが"Y"選択時: 全セッション内容を<kiro_investigation>に保存
  - ユーザーが"N"選択時: メモリクリア、ファイル保存なし
```

### 7. **既存investigateとの差別化**

#### 役割の明確化
```yaml
/hm:investigate:
  特徴: 自動保存、複数トピック、フラグベース
  用途: 計画的な調査、バッチ処理

/hm:interactive-investigate:
  特徴: 選択的保存、単一トピック、対話型
  用途: 探索的調査、試行錯誤
```

### 8. **レスポンス解析の堅牢性**

#### 入力バリエーションへの対応
```python
valid_responses:
  save: ["Y", "y", "yes", "YES"]
  discard: ["N", "n", "no", "NO"]
  continue: ["A,*", "a,*", "A, *", "a, *"]

edge_cases:
  "A": 追加質問を促す
  "A,": 質問内容を要求
  " Y ": 空白をトリムして処理
```

### 9. **ファイル操作の安全性**

#### <kiro_investigation>の更新戦略
```yaml
append_strategy:
  1. Read <kiro_investigation> (必ず存在)
  2. タイムスタンプヘッダー追加
  3. セッション全体を追記
  4. MultiEditで安全に更新
  5. 既存内容は保持
```

### 10. **ユーザー体験の最適化**

#### 明確なフィードバック
```markdown
視覚的フィードバック:
- 🔍 調査開始
- ✅ 保存成功
- ❌ 破棄完了
- 🔄 継続中

進捗表示:
- ラウンド数
- 決定内容
- トピック名
```

## 他コマンドとの連携

### Kiroエコシステムでの位置づけ
```yaml
/hm:investigate:
  責任: 計画的・体系的な技術調査
  特徴: 自動保存、並列調査

/hm:interactive-investigate:
  責任: 探索的・対話的な技術調査
  特徴: 選択的保存、逐次深化

/hm:requirements:
  連携: 調査結果を要件に反映

/hm:design:
  連携: 調査結果を設計に活用
```

## 成功基準

1. **真のインタラクティビティ**: ユーザーが実際に選択できる
2. **完全な履歴**: 表示内容の喪失なし
3. **柔軟な継続**: 無制限の調査ラウンド
4. **明確なナビゲーション**: 常に選択肢が明確
5. **堅牢な解析**: 様々な入力形式に対応
6. **コンテキスト保存**: 間違いと修正の記録
7. **クリーンな統合**: Kiroシステムとのシームレスな連携

この設計により、`/hm:interactive-investigate`は探索的な技術調査において、ユーザーとの対話を通じて知識を深化させる強力なツールとして機能します。