# Tasks

## Required Investigations
*Topics will be defined after requirements completion*

## State Tracking

| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | pending | - | Define requirements |
| investigation.md | pending | - | Start investigation after requirements |
| design.md | complete | - | - |
| tasks.md#Timeline | in-progress | phase2: 3/3 (100%) | テスト実行 |

## Timeline

- [x] Spec created → enhance-requirements
- [ ] Requirements definition
- [ ] Investigation topics identification
- [x] Design documentation → design.md#overview
- [x] Implementation planning → design.md#実装影響分析
- [x] phase1: Pattern Router Markdown修正 → design.md#設計詳細
  - [x] 06_nudges.md修正（PBIタイプ条件分岐追加）
  - [x] 04_workflows.md修正（Pre-action簡潔化 + `<reasoning>`タグ）
  - [x] 07_requirements.md修正（Boundaries強化、具体例追加）
  - [x] pbi/decompose.md修正（コンテンツ検証追加）
- [x] phase2: ビルドと検証 → design.md#コンパイルとデプロイ
  - [x] `just build` でリビルド
  - [x] コンパイルエラーチェック
  - [x] バイナリ動作確認
- [ ] phase3: 動作確認テスト → design.md#検証戦略
  - [ ] テストケース1: PBI nudging検証
  - [ ] テストケース2: Pre-action範囲検証
  - [ ] テストケース3: テンプレート強制検証
  - [ ] テストケース4: PBI decompose検証
