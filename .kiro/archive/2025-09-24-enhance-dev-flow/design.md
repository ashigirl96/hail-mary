# Design - Kiroシステムプロンプト拡張

## Meta
- **Completeness**: 60%
- **Requirements**: spec-driven開発フローの反復的な性質への対応
- **Architecture Scope**: System Prompt Enhancement

## Overview

現在のKiroシステムは線形的なフロー（requirements → investigation → design）を前提としているが、実際の開発は反復的で相互依存的である。この問題を解決するため、slash commandの振る舞いをシステムプロンプトのXMLタグとして埋め込み、Claude Codeの基本認知能力として組み込む。

調査の結果、Claudeは**XMLタグ構造に特別な注意を払うよう訓練**されており、フラットとネストの**ハイブリッドアプローチ**が最適であることが判明した。

## Design Concept

### アプローチ：ハイブリッドXMLタグシステム

システムプロンプトに埋め込むXMLタグを以下の原則で設計：
- **メインフローはフラット**: 素早いアクセスと参照性を重視
- **詳細情報はネスト**: 関連情報をグループ化して認知負荷を軽減
- **命名規則で関連性を明示**: `kiro-requirements-*`のような接頭辞でグループ化

### 設計原則

1. **直接参照性**: トリガーとフローは最上位に配置
2. **論理的グループ化**: 関連する詳細はネストで整理
3. **関数的連鎖**: タグが他のタグを呼び出す明確な参照関係
4. **認知的アクセシビリティ**: 会話中に「どこを見るか」が明確

## Detailed XML Tag Structure

### 1. トリガー層（フラット・エントリーポイント）

```xml
<kiro-triggers>
- requirements: 要件, 仕様, PRD, requirement → <kiro-requirements-flow>
- investigation: 調査, 解析, 検証, investigate → <kiro-investigation-flow>
- design: 設計, アーキテクチャ, design → <kiro-design-flow>
</kiro-triggers>
```

**設計意図**: 最上位に配置することで、ユーザー発話から即座にマッチング可能

### 2. フロー層（フラット・処理フロー）

```xml
<kiro-requirements-flow>
1. Read <kiro_requirements> if exists
2. Parse user input with <kiro-requirements-template>
3. Show diff/preview
4. Execute <kiro-requirements-suggestions> → before-update
5. Update file
6. Execute <kiro-requirements-suggestions> → after-update
7. Check <kiro-impact-detection> → requirements
</kiro-requirements-flow>

<kiro-investigation-flow>
1. Read <kiro_investigation> if exists
2. Check <kiro-prerequisites> → investigation
3. Perform investigation
4. Execute <kiro-impact-detection> → investigation
5. Update file
6. Execute <kiro-investigation-suggestions> → after-update
</kiro-investigation-flow>

<kiro-design-flow>
1. Check <kiro-prerequisites> → design
2. Read <kiro_design> if exists
3. Read <kiro_investigation> for context
4. Generate design with <kiro-design-template>
5. Execute <kiro-design-suggestions> → before-update
6. Update file
7. Execute <kiro-design-suggestions> → after-update
</kiro-design-flow>
```

**設計意図**: 各フローを独立したフラットタグにすることで、処理の流れを明確化

### 3. 前提条件層（ネスト・ゲートキーパー）

```xml
<kiro-prerequisites>
  <investigation>
    - If starting new: OK to proceed
    - If updating: Check related <kiro_requirements> sections
  </investigation>

  <design>
    - MUST have fresh <kiro_investigation> (< 3 days old)
    - If missing: "先に調査が必要です。investigationを実行しますか？"
    - If stale: "調査結果が古い可能性があります。更新しますか？"
  </design>
</kiro-prerequisites>
```

**設計意図**: 前提条件を1つのタグにまとめ、内部で対象別にネスト

### 4. サジェスション層（ネスト・対話的提案）

```xml
<kiro-requirements-suggestions>
  <before-update>
    - "この内容で要件を更新しますか？ [Y/n]"
    - Show completeness score: "現在の完成度: XX%"
    - If < 70%: "調査が必要な項目: [list]"
  </before-update>

  <after-update>
    - "要件を追加で記載しますか？"
    - If has [TBD]: "技術調査を開始しますか？" → <kiro-investigation-flow>
    - If completeness = 70%: "次はdesignを作成できます" → <kiro-design-flow>
  </after-update>
</kiro-requirements-suggestions>

<kiro-investigation-suggestions>
  <after-update>
    - "追加で調査が必要ですか？"
    - If impacts design: "designの更新が推奨されます" → <kiro-design-flow>
    - If impacts requirements: "要件に反映しますか？" → <kiro-requirements-flow>
  </after-update>
</kiro-investigation-suggestions>

<kiro-design-suggestions>
  <before-update>
    - "この設計で更新しますか？ [Y/n]"
    - Show design completeness: "設計完成度: XX%"
    - List affected files: "影響ファイル: [count]個"
  </before-update>

  <after-update>
    - "実装を開始しますか？"
    - "追加の設計変更が必要ですか？"
    - If requirements changed: "要件の変更を反映しますか？"
  </after-update>
</kiro-design-suggestions>
```

**設計意図**: タイミング別（before/after）にネストして、文脈に応じた提案を整理

### 5. 影響検出層（ネスト・自動分析）

```xml
<kiro-impact-detection>
  <requirements>
    - On change: Scan for impacts on investigation, design
    - Patterns: "新機能追加", "要件変更", "スコープ拡大"
    - If significant: "関連調査の更新が必要かもしれません"
  </requirements>

  <investigation>
    - Scan for: "API変更", "DB構造", "アーキテクチャパターン"
    - Confidence scoring: 0-100%
    - If high impact (>80%): "この調査結果は<kiro_design>に影響します"
    - Auto-suggest: "designを更新しますか？" → <kiro-design-flow>
  </investigation>

  <design>
    - Monitor: "実装方針変更", "技術選択", "ファイル構造"
    - If conflicts with requirements: "要件との不整合を検出"
    - Suggest: "要件を更新しますか？" → <kiro-requirements-flow>
  </design>
</kiro-impact-detection>
```

**設計意図**: 影響検出ロジックを一箇所に集約し、ドキュメント別にネスト

### 6. テンプレート層（フラット・データ構造）

```xml
<kiro-requirements-template>
# Requirements - [Feature Name]

## Metadata
- **Completeness**: [0-100%]
- **Source**: [user-input|github-issue]
- **Last Updated**: [timestamp]
- **Dependencies**: [investigation status, design status]

## Overview
[Problem and solution]

## User Stories
- As a [user], I want [feature] so that [benefit]

## Technical Requirements
[TBD - populated by investigation]
</kiro-requirements-template>

<kiro-investigation-template>
# Investigation - [Topic]

## Metadata
- **Confidence**: [0-100%]
- **Sources**: [codebase, docs, web]
- **Last Updated**: [timestamp]
- **Impacts**: [requirements sections, design sections]

## Findings
[Key discoveries and patterns]

## Recommendations
[Actionable insights]
</kiro-investigation-template>

<kiro-design-template>
# Design - [Feature Name]

## Meta
- **Completeness**: [0-100%]
- **Based On**: [investigation timestamp]
- **Requirements Version**: [requirements timestamp]

## Overview
[As-Is/To-Be overview]

## Design
[Implementation approach]

### Target Files
[List of files to modify/create with specific changes]
</kiro-design-template>
```

**設計意図**: テンプレートは独立して参照されるため、フラットに配置

### 7. ルール層（ネスト・制約定義）

```xml
<kiro-rules>
  <requirements>
    <will>
      - Follow <kiro-requirements-template>
      - Max 70% completeness without investigation
      - Always show diff before update
      - Interactive refinement until satisfaction
    </will>
    <will-not>
      - Exceed 70% without investigation
      - Skip user confirmation
      - Auto-generate without input
    </will-not>
  </requirements>

  <investigation>
    <will>
      - Check impact on other documents
      - Maintain investigation history
      - Score confidence levels
      - Proactively suggest updates
    </will>
    <will-not>
      - Skip verification
      - Ignore dependencies
      - Overwrite without backup
    </will-not>
  </investigation>

  <design>
    <will>
      - Require fresh investigation
      - Link to investigation findings
      - Show affected components
      - Track design decisions
    </will>
    <will-not>
      - Design without investigation
      - Ignore requirement constraints
      - Skip impact analysis
    </will-not>
  </design>
</kiro-rules>
```

**設計意図**: ルールをドキュメント別に整理し、Will/Will Notで明確化

### 8. 依存関係層（フラット・相互参照）

```xml
<kiro-dependencies>
- requirements enables → investigation, design
- investigation updates → design, requirements[technical]
- investigation validates → requirements[feasibility]
- design requires → investigation[fresh], requirements[complete>50%]
- design affects → tasks, implementation
- all changes trigger → <kiro-impact-detection>
</kiro-dependencies>
```

**設計意図**: 依存関係は全体を俯瞰する必要があるため、フラットに配置

## Implementation Details

### File: `specification_section_template.md`

現在の構造に以下を追加：

````markdown
## Specification

**Current**: {spec_name} (`{spec_path}`)

- <kiro_requirements>{spec_path}/requirements.md</kiro_requirements>
- <kiro_design>{spec_path}/design.md</kiro_design>
- <kiro_tasks>{spec_path}/tasks.md</kiro_tasks>
- <kiro_investigation>{spec_path}/investigation.md</kiro_investigation>
- <kiro_memo>{spec_path}/memo.md</kiro_memo>

## Kiro Reactive Behaviors

<!-- ハイブリッド構造のXMLタグをここに挿入 -->
<kiro-triggers>
...
</kiro-triggers>

<kiro-requirements-flow>
...
</kiro-requirements-flow>

<!-- 以下、上記で定義した全タグ -->
````

### File: `SystemPrompt::new()` の拡張

```rust
impl SystemPrompt {
    pub fn new(spec_name: Option<&str>, spec_path: Option<&Path>, steerings: &Steerings) -> Self {
        let steering_content = steerings.to_string();

        // 新機能: Kiroリアクティブ動作の生成
        let reactive_behaviors = if spec_name.is_some() {
            generate_reactive_behaviors(spec_path)
        } else {
            String::new()
        };

        let specification_section = if let (Some(name), Some(path)) = (spec_name, spec_path) {
            SPECIFICATION_SECTION_TEMPLATE
                .replace("{spec_name}", name)
                .replace("{spec_path}", &path_str)
                .replace("{reactive_behaviors}", &reactive_behaviors)
        } else {
            String::new()
        };

        // ...
    }

    fn generate_reactive_behaviors(spec_path: Option<&Path>) -> String {
        // スペックの現在の状態を分析
        let has_requirements = check_file_exists(spec_path, "requirements.md");
        let has_investigation = check_file_exists(spec_path, "investigation.md");
        let has_design = check_file_exists(spec_path, "design.md");

        // 状態に応じてリアクティブ動作を調整
        // 例: investigationが古い場合、<kiro-prerequisites>に警告を追加

        REACTIVE_BEHAVIORS_TEMPLATE.to_string()
    }
}
```

## Benefits

### 調査から得られた知見の反映

1. **XMLタグへの特別な注意**: ClaudeがXMLに最適化されている特性を活用
2. **ハイブリッド構造**: フラットな素早いアクセスとネストの整理を両立
3. **明確な命名規則**: `kiro-*`接頭辞で関連タグを即座に識別
4. **関数的連鎖**: タグ間の参照関係が明確で追跡可能

### 実用的な改善

1. **プロアクティブな提案**: 影響検出による自動提案
2. **前提条件の強制**: 必要な調査なしに設計に進むことを防ぐ
3. **反復的な開発**: 要件→調査→設計の往復を自然にサポート
4. **認知負荷の軽減**: ハイブリッド構造で「どこを見るか」が明確

## Next Steps

- [x] ハイブリッドXMLタグ構造の設計
- [ ] テンプレートファイルへの実装
- [ ] Rustコードでの動的生成ロジック
- [ ] 実際の開発フローでの検証
- [ ] パフォーマンスとユーザビリティの評価

---

## Completeness Scoring Rule
- 0-30%: コンセプトとXMLタグ定義
- 30-60%: 実装方法の詳細とベストプラクティスの反映
- 60-80%: コード例とファイル変更の具体化
- 80-100%: テスト済みの完全な実装