# Memo: enhance-remember

- 調査のコツ
- このプロジェクトのコーディングのコツ
- 単純にドメイン知識
- 忘れそうな機能の実装方法

## 学習内容の保存形式

### 8つの形式と使い分け

| 形式 | 最適な用途 | 文字数目安 | キーワード |
|------|-----------|------------|------------|
| **Tips** | 実装の小技、最適化テクニック | 5-7行 | use, optimize, improve |
| **Rules** | 必須の規約、禁止事項 | 8-10行 | always, never, must |
| **Pattern** | 設計判断、アーキテクチャ | 10-12行 | pattern, architecture, design |
| **Guide** | 調査方法、セットアップ手順 | 15-20行 | how to, setup, configure, investigate |
| **Knowledge** | ドメイン知識、概念説明 | 10-15行 | what is, definition, formula, concept |
| **Recipe** | 実装手順、コードレシピ | 12-18行 | implement, create, build, recipe |
| **Gotcha** | 落とし穴、注意点 | 6-8行 | avoid, gotcha, pitfall, warning |
| **Workflow** | 作業フロー、プロセス | 10-15行 | workflow, process, steps, flow |

## 各形式の例

### Tips形式 (実装のコツ)
```markdown
## Use Garbage Collection for Memory-Intensive Batches
**When**: Processing 1000+ records with ORMs
- Chunk data into batches of 100-1000
- Call `global.gc()` between batches
- Monitor with `process.memoryUsage()`
✅ `chunk(data, 100).forEach(batch => { process(); gc(); })`
❌ `Promise.all(largeArray.map(process))`
```

### Rules形式 (守るべきルール)
```markdown
## Batch Processing Memory Management
**Priority**: 🔴 **Triggers**: OOMKilled errors, datasets >1000 records

- **Always chunk large arrays**: Never process all at once
- **Force GC between batches**: Use `--expose-gc` flag
- **Monitor memory usage**: Log after each batch
- **Set batch size constants**: UPSERT=100, CREATE_MANY=1000

✅ **Right**: Sequential batch processing with GC
❌ **Wrong**: Parallel processing entire dataset
```

### Pattern形式 (設計パターン)
```markdown
## Pattern: Chunked Processing with GC
**Context**: Node.js app hitting memory limits with large datasets
**Problem**: OOMKilled when processing 10,000+ Prisma records
**Solution**: 
```typescript
for (const batch of chunk(data, BATCH_SIZE)) {
  await processBatch(batch);
  if (global.gc) global.gc();
}
```
**Trade-off**: Slower but stable (99% memory reduction)
```

### Guide形式 (調査・実装ガイド)
```markdown
## BigQuery EXTERNAL_QUERY Guide
**Context**: PostgreSQL to BigQuery queries for risk analysis
**Prerequisites**: GCP auth, proper connection string

### Step-by-Step
1. Auth: `gcloud auth login`
2. Query structure: `EXTERNAL_QUERY("connection", '''SQL''')`
3. Type casting: PostgreSQL types → VARCHAR/INT
4. UUID handling: Single quotes only (no escaping)

### Common Pitfalls
- No comments in SQL files (causes encoding errors)
- Use double quotes for column names
- Cast PostgreSQL-specific types

### Examples
✅ `WHERE id = '5665b28f-2bb8-4a86-90ba-f5622cec34f4'`
❌ `WHERE id = ''5665b28f-2bb8-4a86-90ba-f5622cec34f4''`
```

### Knowledge形式 (ドメイン知識)
```markdown
## Hedge Calculation Domain
**Domain**: Risk Management / Energy Trading
**Definition**: ヘッジ = リスク回避のための反対売買

### Core Formula
```
ヘッジ量[kWh] = Σ(取引量[kWh] × 売買符号)
売買符号: Buy=+1, Sell=-1
```

### Business Context
- 需要リスクをカバーする先物取引
- 価格変動リスクの相殺メカニズム
- 30分コマ単位での電力量管理
```

### Recipe形式 (実装レシピ)
```markdown
## CSV Upload Implementation Recipe
**Use Case**: Partial replacement upload pattern
**Stack**: Prisma, PostgreSQL, TypeScript

### Ingredients
- Unique key: organizationId + name
- Transaction scope: entire operation
- Related tables: delete → recreate pattern

### Instructions
1. Delete related records (by CSV names only)
2. UPSERT main records (update existing, insert new)
3. Recreate relationships from CSV data

### Result
- IDs preserved ✓
- Audit trail maintained ✓
- Partial update capability ✓
```

### Gotcha形式 (落とし穴・注意点)
```markdown
## Platform-Specific Restaurant ID Gotcha
**Problem**: Web uses URL params, Native uses Redux store
**Impact**: Component reusability issues

### ⚠️ Avoid
```typescript
const id = Platform.OS === 'web' ? params.id : redux.id
```

### ✅ Solution
Pass as props from platform-specific parent components
```

### Workflow形式 (作業フロー)
```markdown
## Performance Investigation Workflow
**When**: API response >500ms, OOMKilled errors

### Investigation Flow
1. Profile: `process.memoryUsage()` monitoring
2. Identify: Find memory accumulation points
3. Batch: Chunk large arrays (100-1000 items)
4. GC: Add `global.gc()` between batches
5. Verify: 99% memory reduction achieved
```

## 自動形式選択ロジック

```yaml
format_selection:
  # 手順・調査方法
  - keywords: ["how to", "setup", "configure", "investigate"]
    → Guide format
  
  # ドメイン・概念説明
  - keywords: ["what is", "definition", "formula", "concept"]
    → Knowledge format
  
  # 実装方法・コード例
  - keywords: ["implement", "create", "build", "recipe"]
    → Recipe format
  
  # 注意点・罠
  - keywords: ["avoid", "gotcha", "pitfall", "warning"]
    → Gotcha format
  
  # 作業手順・プロセス
  - keywords: ["workflow", "process", "steps", "flow"]
    → Workflow format
  
  # 実装テクニック
  - keywords: ["use", "optimize", "improve"]
    → Tips format
  
  # 必須ルール
  - keywords: ["always", "never", "must"]
    → Rules format
  
  # 設計パターン
  - keywords: ["pattern", "architecture", "design"]
    → Pattern format
```

## プロンプト改善提案

```markdown
## Behavioral Flow

1. **Content Analysis**: 
   - Implementation technique → Tips
   - Mandatory practice → Rules
   - Design decision → Pattern
   - Investigation/Setup → Guide
   - Domain/Concept → Knowledge
   - Implementation steps → Recipe
   - Warnings/Pitfalls → Gotcha
   - Process/Flow → Workflow

2. **Auto-Format Selection**:
   ```
   if (contains("investigate", "how to", "setup")) → Guide
   if (contains("definition", "concept", "formula")) → Knowledge
   if (contains("implement", "create", "recipe")) → Recipe
   if (contains("avoid", "gotcha", "warning")) → Gotcha
   if (contains("workflow", "process", "steps")) → Workflow
   if (contains("use", "optimize", "improve")) → Tips
   if (contains("always", "never", "must")) → Rules
   if (contains("pattern", "architecture", "design")) → Pattern
   ```

3. **Conciseness Target**:
   - Tips/Gotcha: 5-8 lines
   - Rules/Pattern: 8-12 lines
   - Guide/Knowledge/Recipe/Workflow: 10-20 lines

4. **Filename Convention**:
   - Tips: `{date}-tip-{feature}.md`
   - Rules: `{date}-rule-{domain}.md`
   - Pattern: `{date}-pattern-{name}.md`
   - Guide: `{date}-guide-{topic}.md`
   - Knowledge: `{date}-knowledge-{domain}.md`
   - Recipe: `{date}-recipe-{feature}.md`
   - Gotcha: `{date}-gotcha-{issue}.md`
   - Workflow: `{date}-workflow-{process}.md`
```

## 実際のプロジェクトでの活用例

- **BigQuery調査のコツ** → Guide形式
- **コーディングパターン** → Pattern/Recipe形式
- **ドメイン知識（ヘッジ計算など）** → Knowledge形式
- **CSVアップロード実装** → Recipe形式
- **プラットフォーム別の罠** → Gotcha形式
- **メモリ管理の最適化** → Tips/Rules形式
- **パフォーマンス調査手順** → Workflow形式

