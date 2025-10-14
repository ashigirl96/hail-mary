# Technical Requirements

## Overview
Pattern Router Frameworkに第3のパイプライン「Brainstorm Pipeline」を追加し、要件が不明確な段階での探索的開発を支援する。MODE_Brainstorming.mdの原則をSpec context内で構造化し、brainstorming.mdレポート生成を通じて要件固化プロセスを実現する。

## Motivation
- **Current State**: Pattern Router Frameworkは「要件が既に明確」という暗黙の前提を持ち、Command/Review Pipelineの両方が構造化されたドキュメント生成を想定
- **Problem**: 個人開発や初期検討段階での探索的対話が支援されず、NO Linear Workflow哲学との矛盾が存在
- **Impact**: 開発者がbrainstorming用途でspecを作成しても、Framework側の支援機能がなく、MODE_Brainstorming.mdとの統合も不十分

## Technical Objectives
1. Brainstorm Pipeline実装（第3のパイプライン）
2. /spec:brainstorm Slash Command追加
3. brainstorming.md生成機能（レポート形式）
4. MODE_Brainstorming.md原則の統合

## Acceptance Criteria
- [ ] Pattern Recognition（03_patterns.md）にBRAINSTORM pattern追加
- [ ] Workflow定義（04_workflows.md）にBrainstorm Pipeline追加
- [ ] Nudge templates（06_nudges.md）にBrainstorm用テンプレート追加
- [ ] brainstorming.md構造定義（11_brainstorming.md）新規作成
- [ ] /spec:brainstorm Slash Command実装（.claude/commands/spec/brainstorm.md）
- [ ] Repository層でbrainstorming.md生成メソッド追加
- [ ] 既存テストが全て通過（154 tests passing維持）

## Risk Assessment
- **Breaking Changes**: なし（既存Pipeline（Command/Review）に影響なし）
- **Migration Effort**: 低（新機能追加のみ、既存機能変更なし）
- **Rollback Plan**: 追加ファイル削除で即座にロールバック可能
