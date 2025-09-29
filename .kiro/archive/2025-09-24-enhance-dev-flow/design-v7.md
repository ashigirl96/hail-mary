# Kiro コマンド最適化設計 v7

## Meta
- **Completeness**: 90%
- **Requirements**: Kiroコマンドのナッジング動作改善とシステムプロンプト最適化
- **Architecture Scope**: System Prompt + Slash Commands

## Overview

### 問題の背景
- 調査実行後に`investigation.md`への保存ナッジが出ない
- システムプロンプト700行以上、各コマンド250行前後（推奨の10倍以上）
- "Lost in the Middle"現象でナッジング指示が無視される
- コマンドの詳細指示がシステムプロンプトを上書きしている可能性

### 解決アプローチ
XMLタグによる構造化とコマンドファイルの極限簡潔化

## Design

### 詳細XML構造案（最終的に不採用だが参考として記録）

#### specification_driven_template.md の深いネスト構造
```markdown
<kiro-spec-driven priority="critical" override="all">

  <!-- 必須実行ディレクティブを最上位に配置 -->
  <kiro-mandatory-execution>
    <investigation-nudge mandatory="true" skip-allowed="false">
      When ANY investigation completes → MUST ask "Save to investigation.md? [Y/n]"
    </investigation-nudge>
    <requirements-nudge mandatory="true">
      When requirements complete → MUST ask "Technical investigation needed?"
    </requirements-nudge>
    <design-nudge mandatory="true">
      When design complete → MUST ask "Extract implementation tasks?"
    </design-nudge>
  </kiro-mandatory-execution>

  <!-- Requirements フェーズ定義 -->
  <kiro-requirements>
    <templates>
      <prd-template>
        # Requirements - [Feature Name]
        ## 1. Overview
        [What problem are we solving?]
        ## 2. Purpose
        [Why this feature is needed]
        ## 3. User Stories
        - As a [user], I want [feature] so that [benefit]
        ## 4. Acceptance Criteria
        - [ ] Given [context], When [action], Then [outcome]
      </prd-template>

      <bug-template>
        # Bug Report - [Title]
        ## Metadata
        - **Severity**: [Critical/High/Medium/Low]
        ## 1. Problem
        - **What's broken**: [description]
        - **How to reproduce**: [steps]
        ## 2. Expected
        - **Should do**: [expected behavior]
      </bug-template>
    </templates>

    <flow>
      <step1>Check existing requirements</step1>
      <step2>Interactive gathering with type-specific questions</step2>
      <step3>Generate document using appropriate template</step3>
      <step4>Iterative refinement until satisfaction</step4>
      <step5>Save and suggest next steps</step5>
    </flow>

    <stop-points>
      <stop id="1">After initial questions - wait for user input</stop>
      <stop id="2">After document generation - wait for confirmation</stop>
      <stop id="3">After each refinement - wait for approval</stop>
    </stop-points>
  </kiro-requirements>

  <!-- Investigation フェーズ定義 -->
  <kiro-investigation>
    <nudging mandatory="true" priority="critical">
      <trigger>Investigation keyword detection</trigger>
      <action>Complete investigation</action>
      <must-ask>Save to investigation.md? [Y/n]</must-ask>
      <blocking>Never proceed without offering save</blocking>
    </nudging>

    <confidence-tracking>
      <levels>
        <low>0-30%</low>
        <medium>30-70%</medium>
        <high>70-100%</high>
      </levels>
      <rule>Design blocked if confidence <70%</rule>
    </confidence-tracking>

    <topic-management>
      <new-topic>Auto-generate kebab-case title</new-topic>
      <existing-topic>Append to existing section</existing-topic>
      <section-preservation>Never overwrite, always append</section-preservation>
    </topic-management>
  </kiro-investigation>

  <!-- Design フェーズ定義 -->
  <kiro-design>
    <prerequisites>
      <investigation-check>
        <threshold>70%</threshold>
        <blocking-message>Investigation required (current: X%, required: >70%)</blocking-message>
      </investigation-check>
    </prerequisites>

    <templates>
      <detailed-template>
        ## Meta
        - **Completeness**: [0-100%]
        - **Requirements**: [Brief summary]
        - **Architecture Scope**: [Backend/Frontend/Full-stack]
        ## Overview
        [As-Is/To-Be overview]
        ## Design
        [Comprehensive description]
        ### [file-path]
        [Current issues]
        [Post-modification state]
        ```language
        // Complete final state code
        ```
      </detailed-template>

      <simple-template>
        ## Meta
        - **Completeness**: [0-100%]
        ## Overview
        [Change overview]
        ## Design
        [Key decisions]
        ### [file]
        ```language
        // Key changes
        ```
      </simple-template>
    </templates>

    <architect-agents>
      <backend-architect>API design, database, security</backend-architect>
      <frontend-architect>UI, accessibility, performance</frontend-architect>
      <system-architect>Scalability, technology selection</system-architect>
    </architect-agents>
  </kiro-design>

</kiro-spec-driven>
```

### 問題点
この深いネスト構造には以下の問題がある：
1. **可読性低下**: XMLのネストが深すぎて人間が読みづらい
2. **Claude の処理**: 深いネストは注意の分散を招く可能性
3. **メンテナンス困難**: 修正時に該当箇所を探しにくい

## 採用する簡潔な設計

### シンプルな実装方針

#### 1. specification_driven_template.md の修正
```markdown
<kiro-spec-driven>
[既存の内容をそのまま保持]
## Kiro Specification-Driven Development Philosophy
...
## Tasks.md Central Hub
...
## Reactive Pattern Recognition System
...
[以下既存内容]
</kiro-spec-driven>
```

単純に全体を`<kiro-spec-driven>`で囲むだけ。内部構造は現状維持。

#### 2. コマンドファイルの簡潔化

**investigate.md**:
```markdown
---
name: investigate
description: "Kiro specification-driven investigation"
allowed-tools: Read, Write, MultiEdit, Grep, Glob
---

# /hm:investigate

Follow <kiro-spec-driven> in system prompt for investigation phase.
Focus on investigation nudging behaviors and confidence tracking.
```

**design.md**:
```markdown
---
name: design
description: "Technical design from requirements"
allowed-tools: Read, Write, MultiEdit, Task
---

# /hm:design

Follow <kiro-spec-driven> in system prompt for design phase.
Check investigation confidence >70% before proceeding.
```

**requirements.md**:
```markdown
---
name: requirements
description: "Structured requirement documents"
allowed-tools: Read, Write, MultiEdit
---

# /hm:requirements

Follow <kiro-spec-driven> in system prompt for requirements phase.
Use appropriate template (PRD/Bug) based on context.
```

### メリット
1. **XMLタグの効果**: Claudeは`<kiro-spec-driven>`を特別に認識
2. **可読性維持**: フラットな構造で人間にも読みやすい
3. **衝突回避**: コマンドが最小限なのでシステムプロンプトと競合しない
4. **保守性**: テンプレートやフローはシステムプロンプト内の自然言語で管理

### 実装順序
1. specification_driven_template.md を`<kiro-spec-driven>`で囲む
2. 3つのコマンドファイルを簡潔版に更新
3. 動作確認（調査後のナッジが出るか）

## 技術的根拠

### Anthropicの公式ベストプラクティスより
- **最適長**: 100-500語（現在の1/10）
- **XMLタグ使用**: セクションの明確な区別に有効
- **データファースト**: コンテキスト先、指示後で30%改善
- **Lost in the Middle対策**: 重要指示は冒頭50行以内

### 期待される改善
- ナッジング実行率: 30% → 90%以上
- システムプロンプト認識率: 向上
- 保守性: 大幅改善

## Completeness Scoring
- 0-30%: 問題分析と基本設計
- 30-60%: XML構造設計とコマンド簡潔化
- 60-80%: 実装詳細と技術根拠
- 80-100%: 完全な実装準備完了（90%達成）