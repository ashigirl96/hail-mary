# Brainstorming

## Pattern Router Framework拡張 - 探索的開発支援

### 課題（Issues）

- **要件が不明確な段階での支援不足**
  - 現在のPattern Router Frameworkは「要件が既に明確」という暗黙の前提
  - PM/GitHub issueで要件がまとまっている場合は `/spec:requirements` で対応可能
  - 個人開発や初期検討段階では使いづらい

- **NO Linear Workflow哲学との矛盾**
  - 哲学: "Start anywhere, go anywhere"
  - 現実: 要件が明確でないとスタート地点に立てない
  - Command/Review Pipeline両方とも「要件明確」前提

- **memo.mdの死蔵**
  - 現在"DO NOT ACCESS"で完全にFrameworkから切り離されている
  - 探索的対話の記録場所が存在しない
  - brainstorming用途でspecを作っても支援機能なし

- **MODE_Brainstorming.mdとの統合不足**
  - グローバル設定として `~/.claude/MODE_Brainstorming.md` が存在
  - Pattern Router Frameworkと連携していない
  - Spec context内での構造化されたbrainstormingが不可能

### 解決策（Solutions）

#### Option 1: Brainstorm Pipeline追加（採用案）

**概要**:
- 第3のパイプラインとして「Brainstorm Pipeline」を追加
- 探索的対話 → brainstorming.mdレポート生成 → 手動で開発移行

**設計**:
```
Brainstorm Pipeline:
Input → patterns → brainstorm → nudges
```

**特徴**:
- MODE_Brainstorming.md原則適用（Socratic Dialogue）
- brainstorming.md R/W（レポート形式）
- Hub/Gatesアクセスなし（探索段階では不要）
- Command Pipelineへの自動移行なし（ユーザー判断）

**責務の明確化**:
- ✅ 対話でbrainstorming
- ✅ brainstorming.mdレポート作成（課題/解決策/懸念点）
- ✅ 保存確認Nudge
- ✅ 次の議論トピック提案
- ❌ 自動requirements.md生成（手動移行）

**実装コスト**: 中
- System Prompt: 4ファイル追加/変更
- Slash Command: 1ファイル新規
- Repository: brainstorming.md生成メソッド追加

#### Option 2: memo.md活用（却下）

**概要**:
- 既存のmemo.mdをbrainstorming用途に転用
- "DO NOT ACCESS"を解除

**問題点**:
- memo.mdは個人メモとして使われている可能性
- 役割の混在（個人メモ vs Framework管理）
- 既存ユーザーへの影響

**実装コスト**: 低（memo.mdアクセス許可のみ）

**却下理由**: 役割分離の原則違反

#### Option 3: --brainstormフラグ統合（保留）

**概要**:
- `/spec:requirements --brainstorm` で探索モード
- 既存コマンドに探索機能を追加

**利点**:
- 新規Slash Command不要
- 既存フローとの統合

**問題点**:
- 各コマンドの責務が複雑化
- `--review`フラグとの混乱
- YAGNI原則（今は不要）

**実装コスト**: 高（全コマンド変更）

**結論**: 必要になったタイミングで検討

### 懸念点（Concerns）

- **brainstorming.mdの永続性**
  - 開発開始後、brainstorming.mdをどう扱うか？
  - Archive候補か、spec内に残すか
  - 解決案: requirements.md生成後もspec内に残す（履歴として価値あり）

- **自動移行の誘惑**
  - 「固化検出 → 自動requirements.md生成」を実装したくなる
  - NO Linear Workflow違反のリスク
  - 解決案: 手動移行を堅持、ユーザー判断を尊重

- **MODE_Brainstorming.mdとの役割重複**
  - グローバル設定とSpec contextでの機能が重複
  - 解決案: グローバル設定はトーン、Brainstorm PipelineはSpec内の構造化

- **レポート形式の柔軟性不足**
  - 課題/解決策/懸念点の固定形式が制約になる可能性
  - 解決案: 推奨形式であって強制ではない、自由記述も許容

- **--continueフラグの実装複雑性**
  - 既存brainstorming.mdを読み込んで対話再開
  - セッション状態の復元が必要
  - 解決案: MVP段階では省略、必要性確認後に実装

### 次の議論ポイント

- [ ] brainstorming.mdのArchive戦略（優先度: 中）
  - 開発完了後もspec内に保持？
  - Archiveに移動？
  - 履歴的価値の評価

- [ ] --continueフラグの必要性検証（優先度: 低）
  - 実際の使用ケースで再開機能が必要か確認
  - MVP段階では省略でよいか

- [ ] 他のSlash Commandとの整合性（優先度: 高）
  - `/spec:status`でbrainstorming.mdの状態表示が必要か
  - `/spec:timeline`との関連性

- [ ] レポート形式のカスタマイズ性（優先度: 低）
  - ユーザーが独自セクションを追加できるか
  - テンプレートの柔軟性

### 実装優先順位

**Phase 1（MVP）**:
1. Brainstorm Pipeline基本実装
2. /spec:brainstorm Slash Command
3. brainstorming.md生成（固定形式）
4. 基本Nudge（保存確認、次トピック提案）

**Phase 2（拡張）**:
1. --continueフラグ（セッション再開）
2. /spec:statusとの統合
3. レポート形式のカスタマイズ

**Phase 3（将来検討）**:
1. --brainstormフラグ（既存コマンド拡張）
2. 自動固化検出（オプトイン）
