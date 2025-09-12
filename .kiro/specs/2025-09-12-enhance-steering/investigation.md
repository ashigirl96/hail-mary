# Investigation

## Research for enhance-steering

## Core Purpose of Steering Command

The steering command aims to **systematically analyze the entire project and maintain living documentation** in steering files that capture project-specific patterns, decisions, and knowledge.

### Key Goals
1. **Automated Discovery** - Scan the entire codebase to discover patterns and practices
2. **Pattern Categorization** - Match discovered patterns to defined steering types from config.toml
3. **Knowledge Persistence** - Store discoveries in version-controlled markdown files in `.kiro/steering/`
4. **Evolution Tracking** - Update steering files as the project evolves
5. **Gap Detection** - Identify patterns not covered by existing types and suggest new ones
6. **Team Knowledge Sharing** - Create shareable, git-trackable documentation

## Current State Analysis

### What Exists Today
- Basic steering command in `.claude/commands/hm/steering.md`
- Reads steering types from `.kiro/config.toml`
- Creates/updates steering files based on project analysis
- Has backup mechanism before modifications
- Detects uncategorized patterns

### Current Implementation Structure
```yaml
config.toml:
  [[steering.types]]
  name: "type_name"        # Determines filename
  purpose: "description"   # Human-readable purpose
  criteria: [              # Analysis patterns
    "Pattern 1: Description",
    "Pattern 2: Description"
  ]
```

## Problem Space

### Issues with Current Implementation
1. **Not Following Slash Command Best Practices**
   - Missing structured behavioral flow
   - No clear step-by-step execution pattern
   - Lacks concrete examples

2. **Poor Interactivity**
   - No user confirmation points
   - No explicit stop markers for decisions
   - Doesn't wait for user input when suggesting new types

3. **Incorrect Tool Usage**
   - Uses "Read @file" pattern instead of just "@file"
   - @ prefix should auto-load content without Read tool
   - Redundant tool calls

4. **Missing Prompt Engineering Patterns**
   - No [STOP HERE AND WAIT] markers
   - Missing bracket notation for meta-instructions
   - No ! prefix for bash command execution

5. **Lack of Evidence-Based Validation**
   - No clear boundaries preventing false reporting
   - Missing "Will Not" section to prevent claiming success without actual file modifications

## Existing Solutions

### Option 1: steering-remember.md (Good Reference)
**Pros:**
- Clear behavioral flow with numbered steps
- Interactive with user confirmations
- Auto-detection and scoring for type matching
- Format detection based on content
- Concrete examples showing usage
- Clear boundaries (Will/Will Not)
- Proper @ prefix usage for auto-loading files

**Cons:**
- Designed for single learning capture, not full project analysis
- Different scope (remembering vs discovering)

### Option 2: SuperClaude Commands
**Pros:**
- Well-structured command patterns
- Clear activation triggers
- Defined tool orchestration
- Integration with MCP servers

**Cons:**
- More complex than needed for steering
- May be overkill for this specific use case

## Technical Research

### Key Patterns from prompt-engineering.md
1. **File References**: Use `@` prefix to auto-load content
2. **Interactive Patterns**: `[STOP HERE AND WAIT FOR USER RESPONSE - DO NOT PROCEED]`
3. **Bracket Notation**: `[XXX]` for meta-instructions
4. **Bash Execution**: `!`command`` with allowed-tools in frontmatter
5. **False Reporting Prevention**: Explicit boundaries in "Will Not" section

### Slash Command Reference Patterns
- Frontmatter with allowed-tools specification
- argument-hint for expected arguments
- Clear description in frontmatter
- Structured markdown with consistent formatting

## Questions & Uncertainties
- [x] What is the core purpose of steering? (Answered: Living documentation)
- [x] How should it interact with users? (Answered: With confirmations and stop points)
- [ ] Should it run automatically or require manual triggers?
- [ ] How often should steering files be updated?
- [ ] Should it support partial updates or full regeneration?

## Resources & References
- `.claude/commands/hm/steering-remember.md` - Good interactive command example
- `.kiro/steering/prompt-engineering.md` - Prompt patterns to follow
- `reference/slash-commands.md` - Slash command documentation
- SuperClaude Commands - Framework command patterns

## 追加調査: Parallel Task Execution Pattern

### 発見した問題の詳細分析
現在の実装では、1つのTask agentが全てのsteering typesを順番に処理している：
```
Task(Investigate steering types)
  → Processes product
  → Then processes tech
  → Then processes structure
  → Then processes prompt-engineering
```

これは非効率的で、本来は各typeに対して独立したTask agentを並列実行すべき。

### Claude Codeにおける並列実行パターン

#### 1. ドキュメントからの証拠
- **RULES.md line 21**: "Batch Operations: ALWAYS parallel tool calls by default, sequential ONLY for dependencies"
- **Context7ドキュメント検索結果**: Claude Codeは単一メッセージ内で複数のtool callsを送信することで並列実行をサポート
- **slash-commands.md**: "You have the capability to call multiple tools in a single response"

#### 2. 並列Task実行の実装方法
**重要な発見**: 並列実行の鍵は、assistantが**単一のレスポンスメッセージ内で複数のTask tool callsを送信**すること。

```markdown
# ✅ 正しい実装 - 並列Task agents
[単一メッセージで複数のTask toolsを送信]
→ Task(Investigate product steering type)
→ Task(Investigate tech steering type)  
→ Task(Investigate structure steering type)
→ Task(Investigate prompt-engineering steering type)
```

これは順番に複数回Taskを呼ぶのとは異なる：
- **Sequential (間違い)**: Task呼び出し → 完了 → 次のTask呼び出し → 完了...
- **Parallel (正しい)**: 全Task呼び出しを1メッセージで送信 → 全て並列実行

### 実装に必要な変更点

#### 1. Parallel Investigation Phase セクションの再設計
```markdown
### Parallel Investigation Phase

Launch parallel Task agents for each steering type:

> 🚀 Launching parallel investigation for {n} steering types...
> 
> Spawning investigation agents:
> • [Agent 1] product - Product overview and value proposition
> • [Agent 2] tech - Technical stack and development environment  
> • [Agent 3] structure - Code organization patterns
> • [Agent 4] prompt-engineering - Claude Code best practices
> 
> [Parallel Task agents processing independently...]
```

#### 2. 各Agentへの独立したミッション
各Task agentは独自のコンテキストで動作し、特定のsteering typeのみを調査：

```markdown
Investigate the "{type.name}" steering type.

Purpose: {type.purpose}
Criteria: {type.criteria}

Your mission:
1. READ the existing steering file: .kiro/steering/{type.name}.md
2. VERIFY each documented pattern against the actual codebase
3. IDENTIFY incorrect or outdated information
4. DISCOVER new patterns matching the criteria
5. RETURN structured results
```

#### 3. Tool Coordinationセクションの更新
- **Task**: 並列investigation agentsをspawn
  - 単一メッセージで複数Task toolsを送信して同時実行
  - 各agentは独立したコンテキストで動作

#### 4. Key Patternsセクションの更新
- **Parallel Investigation**: Config.toml types → **並列Task agent spawning** → concurrent verification → aggregated results
- **Concurrent Execution**: 複数Task toolsを単一メッセージで送信 → 独立処理 → 同期的な集約

### 実装上の注意点

#### Claude Codeの並列実行の仕組み
1. **Tool Orchestration**: Claude Codeは内部でtool callsを管理し、依存関係のないものは並列実行
2. **Message Structure**: Assistantは1つのメッセージに複数のtool use blocksを含める必要がある
3. **Result Aggregation**: 全Task agentsの結果を待ってから次のステップへ進む

#### デフォルト動作の変更
- 並列実行をデフォルトに（--parallelフラグは不要）
- `--type <name>`指定時のみ単一type調査（並列実行を無効化）

### 期待される効果
1. **パフォーマンス向上**: 4つのtypesを並列調査することで実行時間を大幅短縮
2. **独立性の確保**: 各agentが他のtypeの調査に影響されない
3. **スケーラビリティ**: steering typesが増えても並列度を上げることで対応可能

### 実装確認のポイント
- `/hm:steering`実行時にTask toolが4回別々に呼ばれるか確認
- ログに各Task agentの独立した実行が記録されるか
- 結果の集約が正しく行われるか
