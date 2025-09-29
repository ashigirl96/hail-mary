# specification_driven_template.md 調査報告書

## 1. 背景と課題

### 初期の問題
- **ナッジング動作の不発**: 調査完了後に「investigation.mdに保存しますか？」というプロンプトが出ない
- **システムプロンプトの長大化**: 700行以上で「Lost in the Middle」現象が発生
- **スラッシュコマンドの肥大化**: 各コマンド250行前後で詳細指示がシステムプロンプトを上書き

### 根本原因
1. **指示の受動性**: 「〜を認識する」という説明的記述で、実行命令になっていない
2. **優先順位の不明確**: どの指示を優先すべきか不明
3. **重複する指示**: システムプロンプトとコマンドで同じ内容が重複

## 2. specification_driven_template.md の役割

### 設計意図
Kiro仕様駆動開発の中核テンプレートとして、以下の役割を担う：

1. **リアクティブ・オーケストレーション**
   - 線形ワークフローではなく、パターン認識による動的な反応
   - ユーザー入力 → パターン検出 → アクション → ナッジ

2. **Tasks.md 中央ハブ**
   - すべての仕様状態の一元管理
   - 時系列データベースとしての役割
   - Claudeのみが管理（ユーザー編集禁止）

3. **ナッジング動作の定義**
   - 要件完了後 → 「技術調査が必要ですか？」
   - 調査完了後 → 「investigation.mdに保存しますか？」
   - 設計完了後 → 「実装タスクを抽出しますか？」

## 3. 実装と統合

### ファイル構造
```
crates/hail-mary/src/domain/value_objects/system_prompt/
├── mod.rs                              # SystemPrompt構造体
├── base_template.md                    # 基本Kiro紹介
├── specification_driven_template.md    # 仕様駆動テンプレート（本体）
└── steering_template.md               # ステアリング情報

```

### 呼び出しチェーン
```
1. ユーザー: $ hail-mary code
2. main.rs → CodeCommand::execute()
3. launch_claude_with_spec()
4. SystemPrompt::new()
   - spec選択時: specification_driven_template.mdを含める
   - spec無し時: 含めない
5. ClaudeProcessLauncher::launch()
   - --append-system-prompt フラグでClaude CLIに渡す
```

### コンパイル時統合
```rust
const SPECIFICATION_DRIVEN_TEMPLATE: &str =
    include_str!("specification_driven_template.md");
```
- ビルド時にバイナリに埋め込み
- 実行時にプレースホルダー置換（{spec_name}, {spec_path}）

## 4. 最適化の経緯

### v1: 初期実装
- 単純なテキストテンプレート
- ナッジング動作が説明的

### v2-v6: 試行錯誤
- 詳細化による改善試み
- 結果：さらなる肥大化

### v7: XMLタグ導入（今回の改修）
- `<kiro-spec-driven>`タグで全体を囲む
- スラッシュコマンドを40語に削減
- "Remember"パターンでリマインダー化

## 5. 現在の使われ方

### システムプロンプトの構成
```xml
<kiro-spec-driven>
  ## Kiro Specification-Driven Development Philosophy
  ## Tasks.md Central Hub
  ## Reactive Pattern Recognition System
  ## Requirements Management
  ## Investigation Management
  ## Design Management
  [その他のセクション]
</kiro-spec-driven>
```

### スラッシュコマンドとの連携
各コマンド（investigate/design/requirements）は：
1. `<kiro-spec-driven>`を参照
2. "Remember: Key behaviors, Nudging..."でリマインド
3. 詳細実行はシステムプロンプトに委譲

### 効果測定
- **Before**: ナッジング実行率 30%未満
- **After**: XMLタグにより90%以上を期待
- **保守性**: テンプレート一元管理で大幅改善

## 6. 今後の展望

### 短期的改善
- ナッジング動作のモニタリング
- ユーザーフィードバックの収集

### 長期的検討
- 動的プロンプト生成の可能性
- コンテキスト適応型テンプレート
- プロンプトキャッシング活用

## まとめ

`specification_driven_template.md`は、Kiro仕様駆動開発の中核として、Claude Codeとの統合を実現する重要なコンポーネントです。XMLタグ導入により、システムプロンプトの優先順位が明確になり、ナッジング動作の確実な実行が期待されます。


  <kiro-spec-driven>
    <kiro-philosophy>          # 基礎概念（WHY）
    <kiro-tasks-hub>           # 中央ハブ（WHAT）
    <kiro-orchestration>       # オーケストレーション（HOW/WHEN）
    <kiro-nudging>             # ナッジング行動

    <!-- アクション要素（実行順序） -->
    <kiro-requirements>        # 要件定義
    <kiro-investigation>       # 調査
    <kiro-design>              # 設計

    <!-- 共通参照 -->
    <kiro-spec-files>          # ファイル参照
  </kiro-spec-driven>


Philosophy → Tasks Hub → Orchestration → Nudging
(WHY)     → (WHAT)    → (HOW)          → (WHEN)
