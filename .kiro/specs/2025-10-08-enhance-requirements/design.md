# 設計: Pattern Router Framework 要件ワークフロー改善

## メタ情報
- **完成度**: 100%
- **要件**: 会話から抽出 - PBI nudging改善、pre-action明確化、テンプレート強制
- **アーキテクチャ範囲**: ドキュメント/フレームワーク

## 概要

**現状**:
- 全ての要件タイプで同じnudgingを提供（調査開始を提案）
- Pre-actionの目的が曖昧（「文脈的に正確な」だけ）
- テンプレート遵守が弱い（境界が不明確）
- PBI decompose時に技術的内容の制限がない

**改善後**:
- PBIタイプは `/pbi:decompose` をnudging
- Pre-actionは用語変換のみに明確化
- テンプレート遵守を厳格に強制
- PBI decompose時に技術実装の転送を明示的に禁止

## 設計詳細

この設計は、Pattern Router Frameworkの要件ワークフローにおける3つの核心的な問題に対処します。要件(WHAT)、調査(WHY/HOW research)、設計(HOW implementation)の関心の分離を強化します。

### 1. 要件タイプ別の条件付きNudging

**対象ファイル**: `crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/06_nudges.md`

**目的**: 要件タイプ（PBI vs シングルスペック）に応じて、適切な次のステップにユーザーを誘導する。

**現状** (14-16行目):
```markdown
### After Requirements Complete (event: `requirements:nudge-next`)
- "Investigation topics defined: [list]"
- "Start with `/spec:investigate --topic [first-topic]` for specific topic, or `/spec:investigate` to investigate all?"
```

**改善案**: タイプ別の条件分岐を追加。

**英語版（実装用）**:
````markdown
### After Requirements Complete (event: `requirements:nudge-next`)

**If PBI type:**
- "SBI sections defined: [list of sbi-X-names with types]"
- "Next: `/pbi:decompose` to create individual SBI directories"
- "After decompose: Select SBI with `hail-mary code`, then work independently"
- "Each SBI follows its own lifecycle: requirements → investigate → design → timeline"

**If PRD/Bug/Tech type (Single Spec):**
- "Investigation topics defined: [list]"
- "Start with `/spec:investigate --topic [first-topic]` for specific topic, or `/spec:investigate` to investigate all?"
````

**日本語版（参照用）**:
````markdown
### After Requirements Complete (event: `requirements:nudge-next`)

**PBIタイプの場合:**
- "SBIセクション定義完了: [sbi-X-名前とタイプのリスト]"
- "次のステップ: `/pbi:decompose` で個別SBIディレクトリを作成"
- "decompose後: `hail-mary code` でSBIを選択し、独立して作業"
- "各SBIは独自のライフサイクルを持ちます: requirements → investigate → design → timeline"

**PRD/Bug/Tech タイプの場合（シングルスペック）:**
- "調査トピック定義完了: [リスト]"
- "開始: `/spec:investigate --topic [最初のトピック]` で特定トピックを調査、または `/spec:investigate` で全調査を開始"
````

**根拠**:
- PBIワークフロー: decompose → 個別SBIで作業
- シングルスペックワークフロー: 直接調査開始
- 文脈に応じた適切なガイダンスで混乱を防ぐ

---

### 2. Pre-Action範囲の明確化

**対象ファイル**: `crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/04_workflows.md`

**目的**: Pre-actionの目的を<reasoning>タグで明確化し、詳細なboundariesは07_requirements.mdに委譲。

**現状** (131-132行目):
```markdown
**Before Requirements** (event: `requirements:pre-action`):
Explore codebase comprehensively based on user's request to write contextually accurate requirements
```

**改善案**: 簡潔な指示 + <reasoning>で意図明示。

**英語版（実装用）**:
````markdown
**Before Requirements** (event: `requirements:pre-action`):
Explore codebase comprehensively to translate user language into codebase-compatible terminology.

<reasoning>
This ensures requirements align with existing technical concepts (e.g., "login" → "JWT authentication") while maintaining business/functional focus. Implementation details (file paths, library names, code structure) belong in investigation.md, not requirements.md.
</reasoning>
````

**日本語版（参照用）**:
````markdown
**Before Requirements** (event: `requirements:pre-action`):
コードベースを包括的に探索し、ユーザーの言葉をコードベース互換の用語に変換する。

<reasoning>
これにより、要件を既存の技術概念と整合させる（例: 「ログイン」→「JWT認証」）と同時に、ビジネス/機能焦点を維持する。実装詳細（ファイルパス、ライブラリ名、コード構造）はinvestigation.mdに記載し、requirements.mdには含めない。
</reasoning>
````

**根拠**:
- 04_workflows.mdはactionに集中（WHATとWHY）
- 具体的なboundaries（WHAT NOT）は07_requirements.mdで定義
- <reasoning>タグでClaudeの理解を促進
- 冗長性を排除しつつ、明確性を維持

---

### 3. テンプレート遵守の強化

**対象ファイル**: `crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/07_requirements.md`

**目的**: 厳格なテンプレート遵守を強制し、要件内の技術実装を禁止。

**現状** (3-13行目):
```markdown
**Will**
- **Provide templates** - PRD, Bug, Tech, and PBI templates based on context
- **Ensure completeness** - Verify all required sections before marking done
- **Maintain structure** - Enforce consistent document format
- **Enforce kebab-case** - SBI titles must use lowercase kebab-case

**Will Not**
- **Define orchestration rules** - Orchestration handled by workflows
- **Manage state transitions** - State management handled by hub
```

**改善案**: 明示的な禁止事項で境界を強化（具体例を追加）。

**英語版（実装用）**:
````markdown
**Will**
- **Provide templates** - PRD, Bug, Tech, and PBI templates based on context
- **Ensure completeness** - Verify all required sections before marking done
- **Maintain structure** - Enforce consistent document format
- **Enforce kebab-case** - SBI titles must use lowercase kebab-case (e.g., `sbi-1-backend-api`)
- **Strictly follow templates** - Use exact template structure without deviation or customization
- **Use codebase terminology** - Align with existing technical concepts (via pre-action exploration)
- **Focus on business/functional** - Requirements express WHAT needs to be built, not HOW

**Will Not**
- **Define orchestration rules** - Orchestration handled by workflows
- **Manage state transitions** - State management handled by hub
- **Deviate from templates** - Templates are mandatory requirements, not suggestions
- **Write technical implementation** - No code snippets, file paths (`src/auth/jwt.ts`), library names (jose, passport.js), or architecture details (microservices, API gateway)
- **Include investigation content** - Technical research and evidence belong in investigation.md
- **Mix concerns** - Requirements stay business/functional, investigation handles technical details
````

**日本語版（参照用）**:
````markdown
**Will（実行すること）**
- **テンプレート提供** - 文脈に応じてPRD、Bug、Tech、PBIテンプレートを提供
- **完全性確認** - 完了前に全必須セクションを検証
- **構造維持** - 一貫したドキュメント形式を強制
- **kebab-case強制** - SBIタイトルは小文字kebab-case必須（例: `sbi-1-backend-api`）
- **テンプレート厳格遵守** - カスタマイズや逸脱なく、正確なテンプレート構造を使用
- **コードベース用語使用** - 既存の技術概念と整合（pre-action探索経由）
- **ビジネス/機能焦点** - 要件はHOWではなくWHATを表現

**Will Not（実行しないこと）**
- **オーケストレーションルール定義** - オーケストレーションはworkflowsが処理
- **状態遷移管理** - 状態管理はhubが処理
- **テンプレート逸脱** - テンプレートは提案ではなく必須要件
- **技術実装記述** - コードスニペット、ファイルパス（`src/auth/jwt.ts`）、ライブラリ名（jose、passport.js）、アーキテクチャ詳細（マイクロサービス、APIゲートウェイ）は不可
- **調査内容含む** - 技術調査と証拠はinvestigation.mdに記載
- **関心の混在** - 要件はビジネス/機能、調査が技術詳細を扱う
````

**根拠**:
- "Provide templates" → "厳格遵守" (より強い強制)
- 技術コンテンツの明示的禁止
- 明確な分離: requirements.md (WHAT) vs investigation.md (HOW research)
- steering-prompt-engineering の "Preventing False Reporting" パターンを参照

---

### 4. PBI Decompose時のコンテンツ制限

**対象ファイル**: `.claude/commands/pbi/decompose.md`

**目的**: SBI requirements.mdがPBI requirements.mdと同じビジネス/機能焦点を維持することを保証。

**現状** (13-23行目):
```markdown
**Will**
- **Parse PBI requirements.md** - Extract all `### sbi-X-[title]` sections with types
- **Validate SBI names** - Enforce lowercase kebab-case format
- **Create SBI directories** - One directory per SBI with requirements.md only
- **Apply correct templates** - Use PRD/Bug/Tech templates based on type

**Will Not**
- **Proceed without PBI requirements.md** - Must have PBI requirements first
- **Create tasks.md/memo.md** - SBI files created when developer starts working
- **Create PBI tasks.md** - Pattern Router doesn't manage PBI-level tasks.md
- **Overwrite existing SBIs** - Error if SBI directory already exists
```

**改善案**: コンテンツ転送制限を追加。

**英語版（実装用）**:
````markdown
**Will**
- **Parse PBI requirements.md** - Extract all `### sbi-X-[title]` sections with types
- **Validate SBI names** - Enforce lowercase kebab-case format
- **Validate SBI content** - Ensure sections contain business/functional requirements only
- **Create SBI directories** - One directory per SBI with requirements.md only
- **Apply correct templates** - Use PRD/Bug/Tech templates based on `requirements type:` field
- **Transfer business requirements** - Move functional/user-focused content from PBI sections

**Will Not**
- **Proceed without PBI requirements.md** - Must have PBI requirements first
- **Create tasks.md/memo.md** - SBI files created when developer starts working
- **Create PBI tasks.md** - Pattern Router doesn't manage PBI-level tasks.md
- **Overwrite existing SBIs** - Error if SBI directory already exists
- **Transfer technical implementation** - No code, file paths, library names, or architecture details
- **Include investigation findings** - Technical details belong in each SBI's investigation.md
- **Proceed with invalid content** - Warn if PBI sections contain technical implementation details
````

**日本語版（参照用）**:
````markdown
**Will（実行すること）**
- **PBI requirements.md解析** - 全ての `### sbi-X-[title]` セクションとタイプを抽出
- **SBI名検証** - 小文字kebab-case形式を強制
- **SBIコンテンツ検証** - セクションにビジネス/機能要件のみが含まれることを確認
- **SBIディレクトリ作成** - SBIごとに1ディレクトリ、requirements.mdのみ作成
- **正しいテンプレート適用** - `requirements type:` フィールドに基づきPRD/Bug/Techテンプレートを使用
- **ビジネス要件転送** - PBIセクションから機能/ユーザー焦点のコンテンツを移動

**Will Not（実行しないこと）**
- **PBI requirements.md無しで実行** - PBI requirementsが先に必要
- **tasks.md/memo.md作成** - 開発者が作業開始時にSBIファイルを作成
- **PBI tasks.md作成** - Pattern RouterはPBIレベルのtasks.mdを管理しない
- **既存SBI上書き** - SBIディレクトリが既に存在する場合はエラー
- **技術実装転送** - コード、ファイルパス、ライブラリ名、アーキテクチャ詳細は不可
- **調査結果含む** - 技術詳細は各SBIのinvestigation.mdに記載
- **無効なコンテンツで実行** - PBIセクションに技術実装詳細が含まれる場合は警告
````

**根拠**:
- SBI requirements.mdはPBI requirements.mdと同じ制約を継承
- 転送前にソースコンテンツを検証
- 要件ドキュメントでの技術負債蓄積を防止
- 証跡チェーン維持: requirements → investigation → design

---

## 実装影響分析

### 修正対象ファイル

| ファイル | 変更行数 | 影響 | リスク |
|---------|---------|------|--------|
| 06_nudges.md | +8行（条件分岐） | 高（UX改善） | 低 |
| 04_workflows.md | +15行（例+境界） | 高（動作変更） | 中 |
| 07_requirements.md | Will +6, Will Not +3 | 重大（強制） | 中 |
| pbi/decompose.md | Will +2, Will Not +3 | 中（検証） | 低 |

### コンパイルとデプロイ

全ファイルは `mod.rs` の `include_str!` でコンパイル時に埋め込まれます:

```rust
// crates/hail-mary/src/domain/value_objects/system_prompt/mod.rs
const PATTERN_ROUTER_NUDGES: &str = include_str!("pattern_router/06_nudges.md");
const PATTERN_ROUTER_WORKFLOWS: &str = include_str!("pattern_router/04_workflows.md");
const PATTERN_ROUTER_REQUIREMENTS: &str = include_str!("pattern_router/07_requirements.md");
```

**デプロイ手順**:
1. `pattern_router/` ディレクトリ内のmarkdownファイルを編集
2. リビルド: `cargo build` または `just build`
3. 変更がバイナリに自動埋め込み
4. 起動: `hail-mary code` で更新されたsystem promptが有効

**コード変更不要** - 純粋なドキュメント更新のみ。

---

## 検証戦略

### 実装後チェック項目

**テストケース1: PBI Nudging**
```bash
# PBIスペック作成
hail-mary code → 新規作成 → "test-pbi"
# Claude Codeセッション内:
/spec:requirements --type pbi
# 期待されるnudge: "次のステップ: `/pbi:decompose` で個別SBIディレクトリを作成"
```

**テストケース2: Pre-Action範囲**
```bash
# シングルスペック作成
hail-mary code → 新規作成 → "test-feature"
/spec:requirements
# ユーザー入力: "ユーザーがログインできる"
# 期待: 要件に「認証」という用語を含む（用語変換）
# 期待: 要件にファイルパスやライブラリを含まない
```

**テストケース3: テンプレート強制**
```bash
# PRDスペック作成
/spec:requirements --type prd
# 期待: PRDテンプレートへの厳格な遵守
# 期待: User Stories、Functional Requirements、Acceptance Criteriaセクション
# 期待: コードスニペットや技術実装なし
```

**テストケース4: PBI Decompose検証**
```bash
# 技術的内容を含むPBI作成（意図的な違反）
/pbi:decompose
# 期待: 技術的内容に関する警告またはエラー
# 期待: 技術詳細をinvestigation.mdに移動する提案
```

---

## 移行と後方互換性

**既存スペック**: 影響なし
- 変更は新規要件作成時のみ適用
- 既存のrequirements.mdファイルは変更なし

**ユーザー体験**: 明確性向上
- 次のステップがより明確（PBI vs シングルスペック）
- Pre-actionの目的理解が向上
- 何をどこに書くかの混乱減少

**フレームワーク整合性**: 強化
- 証跡チェーンがより堅牢
- 関心の分離が強制
- テンプレート遵守が保証

---

## まとめ

この設計は、Pattern Router Frameworkの要件ワークフローに4つの的を絞った改善を実装します:

1. **条件付きNudging** (06_nudges.md): PBI → decompose、シングルスペック → investigate
2. **Pre-Action明確化** (04_workflows.md): 用語変換のみ、実装詳細は不可
3. **テンプレート強制** (07_requirements.md): 厳格遵守、技術コンテンツ不可
4. **Decompose検証** (pbi/decompose.md): ビジネス要件のみ転送

全ての変更は核心原則を強化します: **Requirements = WHAT（ビジネス/機能）、Investigation = WHY/HOW（技術調査）、Design = HOW（実装）**。

実装はmarkdownファイル編集のみで、コンパイル時に自動埋め込み。コード変更ゼロ、破壊的変更ゼロ、純粋なフレームワーク強化。
