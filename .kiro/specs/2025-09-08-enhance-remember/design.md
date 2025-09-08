# Design: Enhanced Steering Remember Command

## Overview

`/hm:steering-remember` コマンドを改善し、学習内容を3つの形式（Rule, Guide, Knowledge）で自動分類して簡潔に保存する。

## Command Structure

```
/hm:steering-remember [title] [--format rule|guide|knowledge] [--verbose]
```

## 3つの保存形式

### 1. Rule形式（規約・パターン）
- **用途**: コーディング規約、ベストプラクティス、実装パターン
- **構造**: When条件 + 箇条書きルール + コード例（✅/❌）
- **文字数**: 7-15行（コード含む）

### 2. Guide形式（手順・方法）
- **用途**: 調査方法、セットアップ手順、ワークフロー
- **構造**: Context + 番号付き手順 + 注意点
- **文字数**: 8-12行

### 3. Knowledge形式（知識・概念）
- **用途**: ドメイン知識、ビジネスロジック、概念説明
- **構造**: Domain/Definition + Formula/Logic + Context
- **文字数**: 5-10行

## Processing Flow

```
User Input (/hm:steering-remember "title")
    ↓
Claude Code analyzes conversation context
    ↓
Auto-detects format based on content
    ↓
Generates concise formatted output
    ↓
Saves to .kiro/steering/draft/{timestamp}-{format}-{title}.md
```

## Slash Command Prompt Definition

```markdown
---
description: Save learning as concise tip in optimal format
allowed-tools: Read, Write, Glob, Bash(date:*)
argument-hint: [title] [--format rule|guide|knowledge] [--verbose]
---

## Triggers
- User identifies new learning or pattern to remember
- Context needs to be preserved for future reference  
- Information not already documented in existing steering files

## Usage
```
/hm:steering-remember [title] [--format rule|guide|knowledge] [--verbose]
```

## Behavioral Flow

1. **Extract Core Learning**: Identify the essential insight from recent conversation
   - Focus on actionable knowledge, not general observations
   - Extract concrete examples if code was discussed
   - Capture the "why" behind decisions

2. **Auto-Detect Format**: Analyze content to choose optimal format
   ```
   if (contains code snippets OR "always/never/must/should/avoid"):
     → Rule format (with ✅/❌ examples)
   elif (contains "step/how to/investigate" OR numbered lists):
     → Guide format (with numbered steps)
   elif (contains "definition/concept/formula/domain"):
     → Knowledge format (with domain context)
   else:
     → Rule format (default)
   ```

3. **Generate Concise Output**: Create formatted content based on detected type
   
   **Rule Format** (7-15 lines with code):
   ```markdown
   ## [Concise Title]
   **When**: [Specific trigger condition]
   - [Key rule 1]
   - [Key rule 2]
   - [Key rule 3 if essential]
   
   ```language
   # ✅ Good
   [correct example]
   
   # ❌ Bad  
   [incorrect example]
   ```
   ```
   
   **Guide Format** (8-12 lines):
   ```markdown
   ## [Action-Oriented Title]
   **Context**: [When/why to use this guide]
   1. [First step]
   2. [Second step]
   3. [Third step]
   ⚠️ [Critical warning if any]
   ```
   
   **Knowledge Format** (5-10 lines):
   ```markdown
   ## [Concept Name]
   **Domain**: [Business/Technical domain]
   **Definition**: [One-line definition]
   **Formula**: `[If applicable]`
   **Context**: [Why this matters]
   ```

4. **Generate Filename**: Create descriptive filename
   - Get timestamp: `date +%Y%m%d-%H%M%S`
   - Sanitize title: lowercase, alphanumeric, max 30 chars
   - Pattern: `{timestamp}-{format}-{title}.md`
   - Example: `20250108-143022-rule-batch-memory.md`

5. **Save to Draft**: Write to `.kiro/steering/draft/` directory
   - Check if draft directory exists
   - Write formatted content to file
   - Confirm successful save

## Key Behaviors

### Content Compression
- **Maximum brevity**: Remove all unnecessary explanation
- **Concrete over abstract**: Include specific examples, not theory
- **Action-oriented**: Focus on what to DO, not background
- **One learning per file**: Don't combine multiple insights

### Format Selection Logic
- **Code present** → Always use Rule format with examples
- **Procedural content** → Guide format with clear steps  
- **Conceptual content** → Knowledge format with definitions
- **When uncertain** → Default to Rule format

### Code Example Handling
- Extract actual code from conversation if available
- Create minimal reproducible examples
- Show contrast between good and bad approaches
- Use appropriate language identifier for syntax highlighting

### Japanese Content Support
- Detect Japanese keywords (使うべき、禁止、手順、定義)
- Preserve Japanese error messages and comments
- Support mixed language content naturally

## Examples

### Example 1: Rule Format Output
```
User: "OpenStructは使わずにplain hashを返すべき"
Output file: 20250108-143022-rule-service-return-values.md

## Service Return Values
**When**: Creating service objects in this codebase
- Return plain hashes for performance
- Wrap in transactions for consistency
- Use Japanese error messages

```ruby
# ✅ Good
def call
  { success: true, data: @result }
end

# ❌ Bad
OpenStruct.new(success: true)
```
```

### Example 2: Guide Format Output
```
User: "BigQueryデバッグは、まずログ確認、次にEXTERNAL_QUERY実行"
Output file: 20250108-143523-guide-bigquery-debug.md

## BigQuery Debug Process
**Context**: Troubleshooting query failures
1. Check Cloud Logging for errors
2. Verify connection string format
3. Run EXTERNAL_QUERY with minimal scope
⚠️ No Japanese comments in SQL files
```

### Example 3: Knowledge Format Output
```
User: "ヘッジ計算は反対売買によるリスク回避"
Output file: 20250108-144012-knowledge-hedge-calculation.md

## Hedge Calculation
**Domain**: Risk Management
**Definition**: リスク回避のための反対売買
**Formula**: `ヘッジ量 = Σ(取引量 × 売買符号)`
**Context**: 30分コマ単位で価格変動リスクを相殺
```

## Boundaries

**Will:**
- Create one file per learning in draft directory
- Auto-detect optimal format from content
- Keep output under 15 lines maximum
- Include concrete examples when relevant

**Will Not:**
- Combine multiple learnings in one file
- Create verbose explanations
- Modify existing steering files directly
- Process without clear learning to capture
```

## Integration Points

### 1. With existing steering system
- Drafts saved to `.kiro/steering/draft/` directory
- Compatible with `/hm:steering` categorization command
- Maintains backward compatibility

### 2. With Claude Code context
- Extracts learning from current conversation
- References code changes and discussions
- Preserves important context

### 3. With version control
- Markdown files are git-friendly
- Atomic commits per learning
- Easy to review and merge

## Quality Assurance

### Validation Rules
1. **Title**: Required, max 50 characters
2. **Content**: Must fit format guidelines (5-15 lines)
3. **Code examples**: Must be syntactically valid
4. **File naming**: Must be filesystem-safe

### Error Handling
```typescript
enum SteeringRememberError {
  NO_LEARNING_FOUND = "No clear learning identified in context",
  TITLE_TOO_LONG = "Title exceeds 50 characters",
  CONTENT_TOO_VERBOSE = "Content exceeds format guidelines",
  INVALID_FORMAT = "Format must be 'rule', 'guide', or 'knowledge'"
}
```

## Migration Strategy

### Phase 1: Update command definition
1. Modify `.claude/commands/hm/steering-remember.md`
2. Add format detection logic
3. Implement new formatters

### Phase 2: Backward compatibility
1. Support existing verbose format with `--legacy` flag
2. Auto-convert old drafts to new format
3. Maintain file naming compatibility

### Phase 3: Full rollout
1. Default to new concise format
2. Update documentation
3. Remove legacy code after 30 days

## Testing Scenarios

### Test Case 1: Auto-detect Rule format
```
Input: "Always use UPSERT for CSV uploads to preserve IDs"
Expected: Rule format with implementation guidance
```

### Test Case 2: Auto-detect Guide format
```
Input: "To debug BigQuery, first check logs, then run EXTERNAL_QUERY"
Expected: Guide format with numbered steps
```

### Test Case 3: Auto-detect Knowledge format
```
Input: "Hedge calculation is risk mitigation through opposite trades"
Expected: Knowledge format with domain context
```

### Test Case 4: Manual format override
```
Input: "--format guide" with rule-like content
Expected: Guide format despite rule indicators
```

## Success Metrics

1. **Conciseness**: 90% of outputs under 15 lines
2. **Accuracy**: 85% correct auto-format detection
3. **Utility**: Increased draft → steering conversion rate
4. **Speed**: < 2 seconds generation time
