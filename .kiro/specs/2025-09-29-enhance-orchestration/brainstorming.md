# Brainstorming: Kiroフレームワークのスケーラビリティ課題と階層的仕様管理

## 現状の課題 🔴

### 1. ファイルの肥大化問題
**問題**: 数ヶ月にわたる大規模プロジェクトで、各ファイルが巨大化
- `requirements.md` が数千行に
- `investigation.md` が際限なく成長（append-only protocol）
- `design.md` に全技術決定が混在
- `tasks.md` が数百のタスクで溢れる

**影響**:
- 必要な情報を見つけるのに時間がかかる
- ファイル全体の把握が困難
- エディタのパフォーマンス低下

### 2. 並列作業の困難さ
**問題**: 現在のフレームワークは単一のアクティブspecのみサポート
- 複数のバグが1つのissueに記載されている場合の対応困難
- 異なるチームが同時に異なる機能を開発できない
- すべてが単一のtasks.mdで管理される

**影響**:
- チーム間の作業がブロックされる
- 並行開発が事実上不可能
- 生産性の低下

### 3. リリース管理の不可能性
**問題**: 機能単位でのリリース切り分けができない
- v1.0とv1.1の機能が同じファイルに混在
- 特定機能だけを切り出してリリースできない
- リリースノートの作成が困難

**影響**:
- 段階的リリースができない
- ロールバック時の影響範囲が不明確
- リリース計画の可視性が低い

### 4. バージョン管理の課題
**問題**: 巨大な単一ファイルによるGit操作の困難さ
- 頻繁なマージコンフリクト
- 差分の確認が困難
- レビューの負荷が高い

**影響**:
- 複数開発者の協働が困難
- コードレビューの品質低下
- 履歴の追跡が困難

### 5. 検索性・発見性の欠如
**問題**: 横断的な情報検索ができない
- 「全specからセキュリティ関連の調査を抽出」が不可能
- インデックス機構の不在
- タグやカテゴリによる分類なし

**影響**:
- 過去の知見を活用できない
- 重複調査のリスク
- ナレッジの蓄積が活かされない

### 6. 相互依存性の管理不能
**問題**: spec間の依存関係を表現できない
- 「認証機能」が「ユーザー管理」に依存することを記述できない
- 前提条件となるspecを明示できない
- 影響範囲の把握が困難

## 提案する解決策 🚀

### 階層的仕様管理（Hierarchical Specification Management）

#### 基本構造
```
parent-specification/
├── tasks.md           # プロジェクトダッシュボード（子spec統合ビュー）
├── requirements.md    # 子仕様へのリンクと要約
├── design.md         # アーキテクチャ全体像と子設計の統合
├── investigation.md  # 分解根拠と境界設定の調査
├── 01-child-auth/    # 認証機能spec
│   ├── tasks.md
│   ├── requirements.md
│   ├── investigation.md
│   └── design.md
└── 02-child-payment/ # 決済機能spec
    ├── tasks.md
    ├── requirements.md
    ├── investigation.md
    └── design.md
```

#### ライフサイクルと役割変化

**Phase 1: PLANNING（計画フェーズ）**
- 親specで全体要件を定義
- 親investigation.mdで技術調査
- 単一のモノリシックなspec

**Phase 2: DECOMPOSITION（分解フェーズ）**
- 親investigation.mdに分解根拠を記録
- 子specを作成
- 境界と依存関係を定義

**Phase 3: EXECUTION（実行フェーズ）**
- 子specで並行開発
- 親tasks.mdがダッシュボード化
- 親requirements/design.mdが索引化

**Phase 4: INTEGRATION（統合フェーズ）**
- 親design.mdで統合設計
- 子spec間の調整
- リリース準備

### 親ファイルの新しい役割

#### 親tasks.md - プロジェクトダッシュボード
```markdown
## Child Specification Status
| Child | Phase | Progress | Dependencies | Team |
|-------|-------|----------|--------------|------|
| 01-auth | EXECUTION | 60% | - | Backend |
| 02-payment | PLANNING | 0% | 01-auth | Payment |
| 03-ui | EXECUTION | 40% | 01-auth | Frontend |

## Milestone Tracking
- [ ] v1.0: 01-auth + 03-ui (2025-02-01)
- [ ] v1.1: 02-payment (2025-03-01)
```

#### 親requirements.md - 統合要件と索引
```markdown
## Overview
統合認証・決済システムの構築

## Child Specifications
- [01-auth](./01-auth/requirements.md): OAuth2.0/SAML認証
- [02-payment](./02-payment/requirements.md): Stripe統合
- [03-ui](./03-ui/requirements.md): 認証UI

## Cross-Cutting Concerns
- セキュリティ要件（全子specに適用）
- パフォーマンス目標
```

#### 親investigation.md - 分解根拠
```markdown
## Decomposition Rationale
**分解の必要性を認識**: 2025-09-29
**根拠**:
- 異なるチームが担当（Backend, Payment, Frontend）
- リリースタイミングが異なる
- 技術的に独立したコンポーネント

## Boundary Analysis
- 01-auth ←→ 02-payment: JWT tokenで連携
- 01-auth ←→ 03-ui: REST APIで連携
```

#### 親design.md - アーキテクチャ統合
```markdown
## System Architecture
[全体アーキテクチャ図]

## Integration Points
- API Gateway設計
- 共通エラーハンドリング
- データフロー

## Child Design Integration
- [01-auth/design.md]: 認証サービス詳細
- [02-payment/design.md]: 決済サービス詳細
```

### 実装上の工夫

#### 1. 段階的導入（Progressive Enhancement）
- 既存の単一specから開始
- 必要に応じて子specに分解
- 強制的な階層化は避ける

#### 2. Cross-Reference Pattern
```markdown
<!-- 親から子への参照 -->
詳細は [01-auth/requirements.md#user-stories]

<!-- 子から親への参照 -->
全体像は [../requirements.md#overview]

<!-- 兄弟間の参照 -->
関連: [../02-payment/investigation.md#api-design]
```

#### 3. Lazy Creation
- すべてのファイルを最初から作らない
- 必要になったタイミングで作成
- 空ファイルの乱立を防ぐ

### 期待される効果 ✅

1. **スケーラビリティ**: 大規模プロジェクトでも管理可能
2. **並行作業**: 複数チームが独立して作業
3. **リリース管理**: 機能単位でのリリース制御
4. **検索性向上**: 階層による自然な分類
5. **依存管理**: 明示的な依存関係の記述
6. **認知負荷軽減**: 必要な部分だけフォーカス

## 懸念事項と対策 ⚠️

### 懸念: 構造が複雑になりすぎる
**対策**:
- 2階層までに制限（親・子のみ、孫は作らない）
- 明確な分解基準を設定
- 小規模プロジェクトは単一specのまま

### 懸念: ファイル間の整合性維持
**対策**:
- 親tasks.mdで一元管理
- 自動検証ツールの開発
- Reactive Pattern-Based Orchestrationの拡張

### 懸念: 学習コストの増加
**対策**:
- 段階的な移行パス
- 豊富なサンプルとドキュメント
- 既存ワークフローとの互換性維持

## 実装アプローチ：Scoped Orchestration Architecture 🎯

### 概念モデルの転換：階層から「スコープ」へ

#### Parent-Child から Scoped-Global へ
当初は「親(管理者)・子(実行者)」という階層モデルで考えていたが、議論を通じて**スコープモデル**への転換が必要と判明。

**Before（階層モデル）:**
```
親spec（管理者）
  ↓ 指示・報告
子spec（実行者）
```

**After（スコープモデル）:**
```
Global Context ←→ Scoped Execution
   （文脈）        （焦点・主人公）
```

**重要な洞察**: 子specで作業中は、その子specが**主人公**。親は上司ではなく**コンテキスト提供者**。

### Dynamic Orchestration System の設計

#### コア実装戦略
現在の`orchestration/index.md`のテンプレート構造を活用し、コンテキストに応じて異なるmarkdownを動的に埋め込む。

```markdown
<!-- index.md の構造は不変 -->
<kiro-hub>
{hub}  <!-- ここに context に応じた内容が入る -->
</kiro-hub>
```

```rust
// Rust側の実装
fn detect_spec_scope(spec_path: &Path) -> SpecScope {
    if has_parent_spec(spec_path) {
        SpecScope::Scoped {
            global_context: parent_path(spec_path)
        }
    } else {
        SpecScope::Global
    }
}

fn select_orchestration_file(name: &str, scope: SpecScope) -> &'static str {
    match (name, scope) {
        ("hub", SpecScope::Scoped) => include_str!("02_hub_scoped.md"),
        ("hub", SpecScope::Global) => include_str!("02_hub.md"),
        // 他のファイルも同様
    }
}
```

### Scoped版が必要なOrchestrationファイル

詳細な分析により、当初の想定より多くのファイルにscoped版が必要と判明：

| ファイル | Scoped版 | 理由 |
|---------|---------|------|
| 00_philosophy.md | ❌ | 普遍的原則は不変 |
| **01_principles.md** | ✅ **01_principles_scoped.md** | Dual hub access protocol |
| **02_hub.md** | ✅ **02_hub_scoped.md** | Local + Global coordination |
| **03_patterns.md** | ✅ **03_patterns_scoped.md** | Scope-aware routing patterns |
| **04_workflows.md** | ✅ **04_workflows_scoped.md** | Dual update protocols |
| **05_gates.md** | ✅ **05_gates_scoped.md** | Cross-scope validation |
| **06_nudges.md** | ✅ **06_nudges_scoped.md** | Context-aware suggestions |
| 07-09_*.md | ❌ | Document structure unchanged |
| **10_spec_files.md** | ✅ **10_spec_files_scoped.md** | Global context references |

### Scoped Orchestration の具体的な振る舞い

#### 02_hub_scoped.md - Two Scopes of Truth
```markdown
**Local Scope** (`./tasks.md`): あなたの直接的な責任領域
**Global Context** (`../tasks.md`): 全体の調整状態

| Your Action | Local Scope | Global Context |
|------------|-------------|----------------|
| タスク追加 | ✅ Full control | ❌ Never add |
| 進捗更新 | ✅ Primary authority | 🔄 Propagate completion |
| 依存確認 | 📖 Read own deps | 📖 Read sibling status |
```

#### 05_gates_scoped.md - Cross-Scope Validation
```markdown
**Sibling Dependency Check**:
- Check: Sibling spec `../01-auth/tasks.md` status
- Action: ⚠️ WARNING if blocked

**Global Prerequisite Check**:
- Check: Global `../requirements.md` decomposition status
- Action: ❌ BLOCK if incomplete
```

#### 10_spec_files_scoped.md - Extended File References
```xml
<global-tasks-file>{global_tasks_path}</global-tasks-file>
<global-requirements-file>{global_requirements_path}</global-requirements-file>
<tasks-file>{tasks_path}</tasks-file>
<requirements-file>{requirements_path}</requirements-file>
<!-- 他のファイル参照 -->
```

### ワークフローの変更点

#### 分解判断ポイント
Requirements → Investigation 完了後に新しい選択肢：
- 「設計しますか？」（従来通り）
- **「仕様を分解しますか？」**（新規）

#### 分解時の動作
1. 現在のspecディレクトリ内に子specディレクトリを作成
2. 各子specに最小限のファイル（tasks.md, requirements.md）を作成
3. 親のinvestigation.mdに分解根拠を記録
4. ユーザーに再起動を促す：「hail-maryを再起動し、子specを選択して作業を続けてください」

### 言語設計：LLMへの適切な指示表現

#### Mental Model を変える表現
**❌ 避けるべき表現:**
- 「親に報告する」
- 「上位への更新」
- 「親タスクを更新」

**✅ 推奨される表現:**
- 「自分の成果をグローバルコンテキストに反映」
- 「完了情報をコンテキストに伝播」
- 「ローカルスコープでの成果を共有」

#### Key Phrase
```
"You are the protagonist of this scope.
Global context is your stage, not your manager."
```

### 実装フェーズ

**Phase 1: 基礎実装**
- 分解提案のnudge追加
- 子spec作成機能
- 単一階層のみサポート

**Phase 2: Dynamic Orchestration**
- _scoped.md ファイル作成
- Rust側の動的選択実装
- スコープ間の連携

**Phase 3: 高度な機能**
- 多階層サポート（必要に応じて）
- 横断的な検索/集計
- 依存関係グラフ可視化

### 効果と価値

1. **既存アーキテクチャの保持**: index.mdテンプレート構造をそのまま活用
2. **段階的移行**: 既存の単一specも動作し続ける
3. **認知的整合性**: 作業中のspecが主人公という自然な mental model
4. **スケーラビリティ**: 大規模プロジェクトでも各スコープは管理可能なサイズ
5. **並行開発**: 複数チームが独立したスコープで作業可能

---

*このドキュメントは、2025-09-29のブレインストーミングセッションで作成され、*
*Scoped Orchestration Architectureの詳細設計を含むよう拡張されました。*
*Kiroフレームワークの次期バージョンに向けた実装仕様です。*