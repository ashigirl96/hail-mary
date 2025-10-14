# Design

## Meta
- **Completeness**: 100%
- **Requirements**: Brainstorm Pipeline追加によるPattern Router Framework拡張
- **Architecture Scope**: Full-stack（Domain/Application/Infrastructure/CLI）

## Overview
**As-Is**: 2つのパイプライン（Command, Review）、要件明確前提、memo.md死蔵

**To-Be**: 3つのパイプライン（Command, Review, Brainstorm）、要件不明確でもスタート可能、brainstorming.md活用、MODE_Brainstorming.md廃止

## Design

MODE_Brainstorming.mdの機能をPattern Router Frameworkに完全統合し、Brainstorm Pipelineとして実装。investigation.md#brainstorm-pipeline-designで確立した設計を踏襲。

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/10_brainstorming.md (New File)

brainstorming.md構造定義（requirements.md#technical-objectives、investigation.md#brainstorm-pipeline-design）。

````markdown
## Brainstorming Document Structure

### Boundaries

**Will**:
- **Exploration capture** - MODE_Brainstorming.mdのSocratic Dialogue記録
- **Brief generation** - 課題/解決策/懸念点の構造化要約
- **Cross-session continuity** - 探索の継続性維持
- **Use spec language** - tasks.mdのLanguageフィールドに従う

**Will Not**:
- **Personal notes** - 個人メモはmemo.md（DO NOT ACCESS維持）
- **Replace requirements.md** - 固化後はrequirements.mdへ手動移行
- **Permanent storage** - 固化後はarchive候補

### Format

investigation.mdと同じトピックベース構造（Append-Only）：

```markdown
# Brainstorming

## [topic-1]

### 課題（Issues）
- [課題1]
- [課題2]

### 解決策（Solutions）
#### Option 1: [名前]
- [説明]
- 実装コスト: [低/中/高]

#### Option 2: [名前]
- [説明]
- 実装コスト: [低/中/高]

### 懸念点（Concerns）
- [懸念1]
- [懸念2]

### 次の議論ポイント
- [ ] [ポイント1]（優先度: [高/中/低]）

---

## [topic-2]

### 課題（Issues）
...
```

### Key Behaviors
- investigation.mdと同じAppend-Only protocol
- トピックベース管理（セクション単位）
- 新規/再開の自動判定
- MODE_Brainstorming.mdのBrief Generation実装
````

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/03_patterns.md

BRAINSTORM pattern追加（investigation.md#pattern-recognition-extension）。MODE_Brainstorming.mdのActivation Triggers統合。

**追加箇所**: 既存EXPLICIT_REVIEW Patterns後

````markdown
**BRAINSTORM Patterns**:

| User Pattern | Action | Strategy Output |
|-------------|--------|--------------------|
| "/spec:brainstorm", "brainstormしたい" | Explore | `{class: "BRAINSTORM", strategy: "brainstorm", components: ["patterns", "brainstorm", "nudges"]}` |
| "何か作りたい", "作成を考えている" | Explore (Vague requests) | 同上 |
| "brainstorm", "explore", "discuss", "figure out" | Explore (Keywords) | 同上 |
| "maybe", "possibly", "thinking about", "could we" | Explore (Uncertainty) | 同上 |
| "UXを考えたい", "user journeyを議論したい" | Explore (Interactive) | 同上 |

**Routing Decision Process** 追加:
```
Input: "/spec:brainstorm UX"
→ Class: BRAINSTORM
→ Confidence: 1.0（明示的コマンド）
→ Strategy: brainstorm
→ Components: ["patterns", "brainstorm", "nudges"]
→ Route to: Brainstorm Pipeline

Input: "UXを一緒に考えたい"
→ Class: BRAINSTORM
→ Confidence: 0.7（暗黙的、MODE_Brainstorming.md由来）
→ Strategy: brainstorm
→ Components: ["patterns", "brainstorm", "nudges"]
→ Route to: Brainstorm Pipeline
```

**Key Principles**:
- Pattern class determines entire routing flow
- No default flow - every input gets classified and routed
- Components are invoked only as specified by strategy
````

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/04_workflows.md

Brainstorm Pipeline定義（investigation.md#brainstorm-pipeline-design）。MODE_Brainstorming.mdのBehavioral Changes統合。

**追加箇所**: Review Pipeline定義後、Document-Specific Pre-Actions前

````markdown
### Brainstorm Pipeline (BRAINSTORM class)
```
Input → patterns → brainstorm → nudges
```

**Characteristics**:
- MODE_Brainstorming.md原則適用
- brainstorming.md R/W（レポート形式）
- Hub/Gatesアクセスなし（探索段階では不要）
- Command Pipelineへの自動移行なし（ユーザー判断）
- 軽量級（Review Pipelineと同等）

**Brainstorm Protocol**:
1. **Socratic Dialogue**: 問いかけで隠れた要件を引き出す（MODE_Brainstorming.md由来）
2. **Non-Presumptive**: 仮定を避け、ユーザーに発見の方向を委ねる
3. **Collaborative Exploration**: 指示的コンサルではなく発見のパートナー
4. 課題/解決策/懸念点を対話で整理
5. "brainstorming.mdに保存しますか？"
6. brainstorming.md生成（Brief Generation）
7. 次の議論トピック提案
8. **終了**（Command Pipeline移行なし）

**Key Behaviors**:
- Stateless until saved: brainstorming.md保存前は一時的
- Manual migration: 開発開始はユーザーが `/spec:requirements` 実行
- Natural dialogue: MODE_Brainstorming.mdの会話スタイル維持

**After Brainstorming Complete**:
<event id="brainstorm:post-action">
1. brainstorming.md保存
2. 次の議論トピック提示
3. Nudge: "開発開始は `/spec:requirements` を実行してください"
</event>
````

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/06_nudges.md

Brainstorm templates追加（investigation.md#slash-command-structure）。MODE_Brainstorming.mdのExamples統合。

**追加箇所**: Review Pipeline Templates後

````markdown
## Brainstorm Pipeline Templates

**探索中**（Socratic Dialogue from MODE_Brainstorming.md）:
- "この機能はユーザーにとってどんな問題を解決しますか？"
- "主要ユーザーとそのメインワークフローは何ですか？"
- "想定ユーザー数とパフォーマンス要件は？"
- "既存システムとの統合は必要ですか？"
- "類似の既存サービスで参考になるものは？"

**Collaborative Exploration**:
- "🔍 一緒に探索しましょう："
- "具体的な課題をお聞かせください"
- "現状 vs 理想の状態を教えてください"
- "セキュリティ要件やコンプライアンス上の制約は？"
- "タイムラインやリソース上の制約は？"

**保存確認**（Brief Generation）:
```
📝 Brainstorming整理完了

課題:
• [抽出された課題1]
• [抽出された課題2]

解決策:
• Option 1: [アプローチ1]
• Option 2: [アプローチ2]

懸念点:
• [懸念1]

brainstorming.mdに保存しますか？
```

**次の議論トピック**:
- "次の議論トピック: [トピック名]（優先度: 高）"
- "他に議論したいトピックはありますか？"

**開発移行案内**:
- "brainstorming完了。開発開始は `/spec:requirements` を実行してください"
- "議論を続ける場合は `/spec:brainstorm --topic [トピック名]` で再開できます"
````

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/11_spec_files.md

brainstorming-file追加。

**追加箇所**: investigation-file定義後

````markdown
- <investigation-file>{investigation_path}</investigation-file> - Research findings and evidence
- <brainstorming-file>{brainstorming_path}</brainstorming-file> - Exploratory dialogue report
- <memo-file>{memo_path}</memo-file> - Internal notes (**DO NOT ACCESS**)
````

### crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/index.md

brainstorming変数追加。

**追加箇所**: investigation変数後

````markdown
<kiro-investigation>
{investigation}
</kiro-investigation>

<kiro-brainstorming>
{brainstorming}
</kiro-brainstorming>

<kiro-design>
{design}
</kiro-design>
````

### crates/hail-mary/src/domain/value_objects/system_prompt/mod.rs

10_brainstorming.md組込、brainstorming.md path追加（investigation.md#repository-layer-implementation）。

```rust
// Pattern Router templates追加
const PATTERN_ROUTER_BRAINSTORMING: &str = include_str!("pattern_router/10_brainstorming.md");

// index.md変数置換に追加
let specification_section = PATTERN_ROUTER_INDEX
    .replace("{philosophy}", PATTERN_ROUTER_PHILOSOPHY)
    // ... 既存変数
    .replace("{investigation}", PATTERN_ROUTER_INVESTIGATION)
    .replace("{brainstorming}", PATTERN_ROUTER_BRAINSTORMING)  // 追加
    .replace("{design}", PATTERN_ROUTER_DESIGN)
    .replace("{spec_files}", &spec_files_section);

// build_pbi_spec_files関数にbrainstorming_path追加
fn build_pbi_spec_files(spec_name: &str, spec_path: &Path) -> String {
    let path_str = spec_path.display().to_string();

    let requirements_path = format!("{}/requirements.md", path_str);
    let design_path = format!("{}/design.md", path_str);
    let tasks_path = format!("{}/tasks.md", path_str);
    let investigation_path = format!("{}/investigation.md", path_str);
    let brainstorming_path = format!("{}/brainstorming.md", path_str);  // 追加
    let memo_path = format!("{}/memo.md", path_str);

    PATTERN_ROUTER_SPEC_FILES
        .replace("{spec_name}", spec_name)
        .replace("{spec_path}", &path_str)
        .replace("{requirements_path}", &requirements_path)
        .replace("{design_path}", &design_path)
        .replace("{tasks_path}", &tasks_path)
        .replace("{investigation_path}", &investigation_path)
        .replace("{brainstorming_path}", &brainstorming_path)  // 追加
        .replace("{memo_path}", &memo_path)
}
```

### .claude/commands/spec/brainstorm.md (New File)

/spec:brainstorm Slash Command（investigation.md#slash-command-structure）。

```yaml
---
name: brainstorm
description: "Collaborative requirement exploration with report generation - triggered by: brainstorm, explore, discuss, UX, user journey, 考えたい, 議論"
argument-hint: "[--topic <name>]"
---

# /spec:brainstorm

MODE_Brainstorming.mdの機能をPattern Router Framework内で実現。探索的対話でbrainstorming.mdレポート作成。

Follow <kiro-workflows> Brainstorm Pipeline:
- During exploration: record to brainstorming.md
- After complete: execute event id="brainstorm:post-action"
- Next action: execute event id="brainstorm:nudge-next" from <kiro-nudges>

Additional context:
- <kiro-philosophy> for reactive pattern routing
- <kiro-patterns> for BRAINSTORM pattern recognition
- <kiro-brainstorming> for brainstorming.md structure
- <kiro-nudges> for brainstorm templates
- <kiro-investigation> for append-only protocol reference

## Usage

```
/spec:brainstorm --topic UX    # UX議論（新規or再開を自動判定）
/spec:brainstorm              # オープンエンド議論
```

## Key Patterns

**Topic Detection** (investigateパターン踏襲):
- `--topic <name>` → 特定トピックに焦点
  - brainstorming.md読取 → セクション検索
  - セクション存在 → Append（議論再開）
  - セクション不在 → Create（新規議論）
- No args → オープンエンド議論

## Boundaries

**Will**:
- Socratic Dialogueで課題/解決策/懸念点整理
- brainstorming.mdレポート生成（Append-Only）
- 保存確認Nudge
- 次の議論トピック提案

**Will Not**:
- 自動requirements.md生成
- Command Pipelineへの自動移行

## Behavioral Flow

1. **Topic判定**:
   - `--topic`あり → brainstorming.md読取、セクション検索
   - セクション存在 → Append mode（再開）
   - セクション不在 → Create mode（新規）
2. **対話実行**: MODE_Brainstorming.md原則でSocratic Dialogue
3. **整理**: 課題/解決策/懸念点を構造化
4. **保存確認**: "brainstorming.mdに保存しますか？"
5. **Append/Create**: トピックセクション更新または新規作成
6. **Nudge**: 次の議論トピック提示

Key behaviors:
- investigation.mdと同じAppend-Only protocol
- トピックベース管理（自動判定）
- 手動移行原則（ユーザー判断尊重）
- MODE_Brainstorming.md廃止（機能統合済）
```

### crates/hail-mary/src/application/repositories/spec_repository.rs

RepositoryInterface拡張（investigation.md#repository-layer-implementation）。

**追加箇所**: 既存メソッド定義後

```rust
pub trait SpecRepositoryInterface {
    // ... 既存メソッド

    /// Generate brainstorming.md in spec directory
    fn create_brainstorming(&self, spec_name: &str, lang: &str) -> Result<(), ApplicationError>;
}
```

### crates/hail-mary/src/infrastructure/repositories/spec.rs

brainstorming.md生成実装（investigation.md#repository-layer-implementation）。

**追加箇所**: impl SpecRepositoryInterface内

```rust
impl SpecRepositoryInterface for SpecRepository {
    // ... 既存メソッド

    fn create_brainstorming(&self, spec_name: &str, lang: &str) -> Result<(), ApplicationError> {
        let spec_path = self.get_spec_path(spec_name)?;
        let brainstorming_path = spec_path.join("brainstorming.md");

        // Template取得
        let template = match lang {
            "ja" => crate::infrastructure::embedded_resources::BRAINSTORMING_TEMPLATE_JA,
            _ => crate::infrastructure::embedded_resources::BRAINSTORMING_TEMPLATE_EN,
        };

        // ファイル生成
        std::fs::write(&brainstorming_path, template)
            .map_err(|e| ApplicationError::FileSystemError(
                format!("Failed to create brainstorming.md: {}", e)
            ))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::TestDirectory;

    #[test]
    fn test_create_brainstorming_ja() {
        let test_dir = TestDirectory::new();
        let path_manager = test_dir.path_manager();
        let spec_repo = SpecRepository::new(path_manager);

        // Spec作成
        spec_repo.create_spec("test-brainstorm", "ja").unwrap();

        // brainstorming.md生成
        let full_name = format!("{}-test-brainstorm", chrono::Utc::now().format("%Y-%m-%d"));
        spec_repo.create_brainstorming(&full_name, "ja").unwrap();

        // 検証
        let brainstorming_path = test_dir.path()
            .join(".kiro/specs")
            .join(&full_name)
            .join("brainstorming.md");
        assert!(brainstorming_path.exists());

        let content = std::fs::read_to_string(&brainstorming_path).unwrap();
        assert!(content.contains("## 課題（Issues）"));
        assert!(content.contains("## 解決策（Solutions）"));
        assert!(content.contains("## 懸念点（Concerns）"));
    }
}
```

### crates/hail-mary/src/infrastructure/embedded_resources.rs

brainstorming.md templates追加（investigation.md#repository-layer-implementation）。

**追加箇所**: 既存テンプレート定義後

```rust
pub const BRAINSTORMING_TEMPLATE_JA: &str = r#"# Brainstorming

## [topic]

### 課題（Issues）
- [課題1]

### 解決策（Solutions）
#### Option 1: [名前]
- [説明]
- 実装コスト: [低/中/高]

### 懸念点（Concerns）
- [懸念1]

### 次の議論ポイント
- [ ] [ポイント1]（優先度: [高/中/低]）
"#;

pub const BRAINSTORMING_TEMPLATE_EN: &str = r#"# Brainstorming

## [topic]

### Issues
- [Issue 1]

### Solutions
#### Option 1: [Name]
- [Description]
- Implementation cost: [Low/Medium/High]

### Concerns
- [Concern 1]

### Next Discussion Points
- [ ] [Point 1] (Priority: [High/Medium/Low])
"#;
```

### crates/hail-mary/src/domain/value_objects/system_prompt/mod.rs (Tests)

Unit tests追加（investigation.md#testing-strategy）。

**追加箇所**: 既存tests mod内

```rust
#[test]
fn test_system_prompt_with_brainstorming() {
    let spec_name = "test-brainstorm";
    let spec_path = PathBuf::from(".kiro/specs/test-brainstorm");
    let steerings = Steerings(vec![]);

    let prompt = SystemPrompt::new(Some(spec_name), Some(&spec_path), &steerings);
    let content = prompt.as_str();

    // brainstorming.mdパス含有確認
    assert!(content.contains("<brainstorming-file>"));
    assert!(content.contains("test-brainstorm/brainstorming.md"));

    // BRAINSTORM pattern定義確認
    assert!(content.contains("<kiro-patterns>"));
    assert!(content.contains("BRAINSTORM Patterns"));

    // Brainstorm Pipeline定義確認
    assert!(content.contains("<kiro-workflows>"));
    assert!(content.contains("Brainstorm Pipeline"));

    // brainstorming.md構造定義確認
    assert!(content.contains("<kiro-brainstorming>"));
    assert!(content.contains("Brainstorming Document Structure"));
}

#[test]
fn test_brainstorm_pipeline_no_hub_access() {
    let spec_name = "test-brainstorm";
    let spec_path = PathBuf::from(".kiro/specs/test-brainstorm");
    let steerings = Steerings(vec![]);

    let prompt = SystemPrompt::new(Some(spec_name), Some(&spec_path), &steerings);
    let content = prompt.as_str();

    // Brainstorm PipelineにHub/Gatesアクセスなし確認
    let workflows_section = content.split("<kiro-workflows>").nth(1).unwrap();
    let brainstorm_section = workflows_section.split("Brainstorm Pipeline").nth(1).unwrap()
        .split("##").next().unwrap();

    assert!(brainstorm_section.contains("Hub/Gatesアクセスなし"));
    assert!(brainstorm_section.contains("Command Pipelineへの自動移行なし"));
}
```

## Implementation Order

1. **System Prompt拡張**（Domain層）
   - 10_brainstorming.md新規作成
   - 03_patterns.md: BRAINSTORM pattern追加
   - 04_workflows.md: Brainstorm Pipeline定義
   - 06_nudges.md: Brainstorm templates追加
   - 11_spec_files.md: brainstorming-file追加
   - index.md: brainstorming変数追加

2. **mod.rs更新**（Domain層）
   - include_str!追加
   - build_pbi_spec_files修正
   - Unit tests追加

3. **Slash Command**（CLI層）
   - .claude/commands/spec/brainstorm.md新規作成

4. **Repository層**（Application/Infrastructure）
   - spec_repository.rs: Interface拡張
   - spec.rs: 実装＋テスト
   - embedded_resources.rs: Templates追加

5. **検証**
   - `just test` 実行（154 tests → 156+ tests）
   - 手動統合テスト（/spec:brainstorm実行）
