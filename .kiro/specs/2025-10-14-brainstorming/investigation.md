# Investigation

## brainstorm-pipeline-design

**Finding**: Pattern Router FrameworkのREADME.mdと既存パイプライン設計から、Brainstorm Pipeline統合の明確な方針を確立。

**Source**:
- `crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/README.md`
- `04_workflows.md:16-163`（Command/Review Pipeline定義）

**設計方針**:

### Pipeline Flow
```
Input → patterns → brainstorm → nudges
```

**Characteristics**（既存パイプラインとの比較）:

| 特性 | Command | Review | Brainstorm |
|-----|---------|--------|------------|
| 重量 | 重量級 | 軽量級 | 軽量級 |
| Hub | R/W | なし | なし |
| Gates | 全検証 | なし | なし |
| 永続化 | tasks.md | エフェメラル | brainstorming.md |
| 移行先 | - | Command | **なし**（手動） |

**Protocol定義**:
```markdown
**Brainstorm Protocol**:
1. MODE_Brainstorming.md原則適用（Socratic Dialogue）
2. 課題/解決策/懸念点を対話で整理
3. "brainstorming.mdに保存しますか？"
4. brainstorming.md生成（レポート形式）
5. 次の議論トピック提案
6. 終了（Command Pipeline移行なし）
```

**Post-Action定義**:
```markdown
<event id="brainstorm:post-action">
1. brainstorming.md保存
2. 次の議論トピック提示
3. Nudge: "開発開始は `/spec:requirements` を実行してください"
</event>
```

**Confidence**: 95%
**Impact**: Brainstorm Pipelineの実装仕様が確定。04_workflows.mdへの追加箇所明確化。

---

## slash-command-structure

**Finding**: 既存Slash Command（requirements.md, design.md）のYAML frontmatter構造とBehavioral Flow pattern確認。

**Source**:
- `.claude/commands/spec/requirements.md:1-35`
- `.claude/commands/spec/design.md:1-21`
- `pattern_router/README.md:549-600`（Slash Command統合セクション）

**/spec:brainstorm 構造**:

```yaml
---
name: brainstorm
description: "Collaborative requirement exploration with report generation"
argument-hint: "[topic] [--continue]"
---

# /spec:brainstorm

MODE_Brainstorming.mdに基づく探索的対話でbrainstorming.mdレポート作成。

Follow <kiro-workflows> Brainstorm Pipeline:
- During exploration: record to brainstorming.md
- After complete: execute event id="brainstorm:post-action"
- Next action: execute event id="brainstorm:nudge-next"

Additional context:
- <kiro-philosophy> for reactive pattern routing
- <kiro-patterns> for BRAINSTORM pattern recognition
- <kiro-brainstorming> for brainstorming.md structure
- <kiro-nudges> for brainstorm templates

## Boundaries

**Will**:
- Socratic Dialogueで課題/解決策/懸念点整理
- brainstorming.mdレポート生成
- 保存確認Nudge
- 次の議論トピック提案

**Will Not**:
- 自動requirements.md生成
- Command Pipelineへの自動移行

## Behavioral Flow

1. 対話開始: MODE_Brainstorming.md原則適用
2. 整理: 課題/解決策/懸念点を構造化
3. 保存確認: "brainstorming.mdに保存しますか？"
4. レポート生成: brainstorming.md作成
5. 次の提案: 次の議論トピック提示
6. 終了: ユーザーが手動で開発移行

Key behaviors:
- 複数回対話可能（`--continue`で再開）
- 手動移行原則（ユーザー判断尊重）
- 独立レポートドキュメント
```

**配置場所**: `.claude/commands/spec/brainstorm.md`

**Confidence**: 98%
**Impact**: Slash Command実装仕様確定。System Prompt Centrism原則に従い、ロジックは04_workflows.md/11_brainstorming.mdに記述、commandは参照のみ。

---

## pattern-recognition-extension

**Finding**: 03_patterns.mdの既存パターンクラス構造分析。BRAINSTORM pattern追加方針確定。

**Source**:
- `03_patterns.md:1-87`（EXPLICIT/EXPLICIT_REVIEW定義）
- `pattern_router/README.md:148-176`（パターン分類システム）

**BRAINSTORM Pattern追加**:

```markdown
**BRAINSTORM Patterns**:

| User Pattern | Action | Strategy Output |
|-------------|--------|-----------------|
| "/spec:brainstorm", "brainstormしたい" | Explore | `{class: "BRAINSTORM", strategy: "brainstorm", components: ["patterns", "brainstorm", "nudges"]}` |
| "UXを一緒に考えたい", "議論したい" | Explore | 同上 |
| "何を作るべきか相談したい" | Explore | 同上 |

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
→ Confidence: 0.7（暗黙的、MODE_Brainstorming.md検出）
→ Strategy: brainstorm
→ Components: ["patterns", "brainstorm", "nudges"]
→ Route to: Brainstorm Pipeline
```
```

**Key Principles確認**:
- Pattern class determines entire routing flow ✅
- No default flow ✅
- Components invoked only as specified by strategy ✅

**Confidence**: 95%
**Impact**: 03_patterns.mdへの追加箇所明確化。既存パターン（EXPLICIT/EXPLICIT_REVIEW）と同等の信頼度設定可能。

---

## repository-layer-implementation

**Finding**: Repository層の実装パターン確認。brainstorming.md生成メソッド追加箇所特定。

**Source**:
- `crates/hail-mary/src/infrastructure/repositories/spec.rs:1-353`
- `crates/hail-mary/src/application/repositories/spec_repository.rs:1-50`

**実装方針**:

### Trait定義追加（application/repositories/spec_repository.rs）
```rust
pub trait SpecRepositoryInterface {
    // 既存メソッド...

    /// Generate brainstorming.md in spec directory
    fn create_brainstorming(&self, spec_name: &str, lang: &str) -> Result<(), ApplicationError>;
}
```

### 実装追加（infrastructure/repositories/spec.rs）
```rust
impl SpecRepositoryInterface for SpecRepository {
    fn create_brainstorming(&self, spec_name: &str, lang: &str) -> Result<(), ApplicationError> {
        let spec_path = self.get_spec_path(spec_name)?;
        let brainstorming_path = spec_path.join("brainstorming.md");

        // Template読込（embedded_resources.rsから）
        let template = self.get_brainstorming_template(lang)?;

        // ファイル生成
        fs::write(&brainstorming_path, template)
            .map_err(|e| ApplicationError::FileSystemError(
                format!("Failed to create brainstorming.md: {}", e)
            ))?;

        Ok(())
    }
}
```

### Template追加（infrastructure/embedded_resources.rs）
```rust
pub const BRAINSTORMING_TEMPLATE_JA: &str = r#"# Brainstorming Report: [Topic]

## 課題（Issues）
- [課題1]

## 解決策（Solutions）
### Option 1: [名前]
- [説明]
- 実装コスト: [低/中/高]

## 懸念点（Concerns）
- [懸念1]

## 次の議論トピック
- [ ] [トピック1]（優先度: [高/中/低]）
"#;

pub const BRAINSTORMING_TEMPLATE_EN: &str = r#"# Brainstorming Report: [Topic]

## Issues
- [Issue 1]

## Solutions
### Option 1: [Name]
- [Description]
- Implementation cost: [Low/Medium/High]

## Concerns
- [Concern 1]

## Next Discussion Topics
- [ ] [Topic 1] (Priority: [High/Medium/Low])
"#;
```

**Clean Architecture適用**:
- Domain: brainstorming構造定義（11_brainstorming.md）
- Application: RepositoryInterface拡張
- Infrastructure: 具体的実装＋Template管理

**Confidence**: 90%
**Impact**: Repository層への変更最小限。既存パターン（requirements.md生成）踏襲で安全。

---

## testing-strategy

**Finding**: 既存テスト構造分析。Brainstorm Pipeline追加時のテスト戦略確立。

**Source**:
- `crates/hail-mary/src/domain/value_objects/system_prompt/mod.rs:141-313`（SystemPrompt tests）
- `crates/hail-mary/tests/` （統合テスト）
- `justfile:28-33`（テストコマンド）

**テスト戦略**:

### 1. Unit Tests（mod.rs）

```rust
#[test]
fn test_system_prompt_with_brainstorming() {
    let spec_name = "test-brainstorm";
    let spec_path = PathBuf::from(".kiro/specs/test-brainstorm");
    let steerings = Steerings(vec![]);

    let prompt = SystemPrompt::new(Some(spec_name), Some(&spec_path), &steerings);
    let content = prompt.as_str();

    // brainstorming.mdパスが含まれることを確認
    assert!(content.contains("<brainstorming-file>"));
    assert!(content.contains("test-brainstorm/brainstorming.md"));

    // BRAINSTORM pattern定義が含まれることを確認
    assert!(content.contains("<kiro-patterns>"));
    assert!(content.contains("BRAINSTORM Patterns"));
}
```

### 2. Repository Tests（spec.rs）

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_create_brainstorming() {
        let temp_dir = TestDirectory::new();
        let spec_repo = SpecRepository::new(/* ... */);

        // brainstorming.md生成
        spec_repo.create_brainstorming("test-spec", "ja").unwrap();

        // ファイル存在確認
        let brainstorming_path = temp_dir.path()
            .join(".kiro/specs/test-spec/brainstorming.md");
        assert!(brainstorming_path.exists());

        // Template内容確認
        let content = fs::read_to_string(&brainstorming_path).unwrap();
        assert!(content.contains("## 課題（Issues）"));
        assert!(content.contains("## 解決策（Solutions）"));
    }
}
```

### 3. Integration Tests（tests/）

```bash
# Slash Command統合テスト（手動）
$ hail-mary code
→ Select: 2025-10-14-brainstorming
→ /spec:brainstorm UX
→ 対話実行
→ brainstorming.md生成確認
```

### 4. CI維持

```bash
# 既存テスト維持確認
just test  # 154 tests passing維持必須
```

**テスト優先順位**:
1. SystemPrompt unit tests（必須）
2. Repository unit tests（必須）
3. Integration tests（推奨）

**Confidence**: 85%
**Impact**: 既存154 testsへの影響なし。新規テスト追加で品質保証。
