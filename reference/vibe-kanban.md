# Vibe KanbanにおけるClaude実行とセッション管理

## 概要

Vibe Kanbanは、Claude CLIを効率的に管理するために、**プロセス単位の実行**と**セッションIDによる会話継続**を組み合わせたアーキテクチャを採用しています。

## Claude実行の仕組み

### 1. 初回実行（新規タスク）

`ClaudeExecutor::spawn()`メソッドで新しいClaudeプロセスを起動：

```rust
// backend/src/executors/claude.rs
command: "npx -y @anthropic-ai/claude-code@latest -p --dangerously-skip-permissions --verbose --output-format=stream-json"
```

- タスクのタイトルと説明をプロンプトとして標準入力に渡す
- `--output-format=stream-json`により、出力をJSON形式で取得
- プロセスは対話完了後に正常終了

### 2. フォローアップ実行（会話継続）

`ClaudeExecutor::spawn_followup()`メソッドで既存セッションを継続：

```rust
// フォローアップ時のコマンド
format!("{} --resume={}", self.command, session_id)
```

- `--resume`オプションで前回のsession_idを指定
- 同じ会話コンテキストを維持しながら新しいプロセスを起動
- ユーザーの追加入力を標準入力として渡す

## セッションID管理

### session_idの永続化

1. **データベーステーブル構造**
   ```rust
   // backend/src/models/executor_session.rs
   pub struct ExecutorSession {
       pub id: Uuid,
       pub task_attempt_id: Uuid,
       pub execution_process_id: Uuid,
       pub session_id: Option<String>,  // Claudeが生成したsession_id
       pub prompt: Option<String>,
       pub summary: Option<String>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   ```

2. **session_idの保存タイミング**
   - Claudeプロセスの出力をストリーミングで読み取り中に抽出
   - `ExecutorSession::update_session_id()`メソッドでDBに保存

### session_idの取得と保存フロー

1. **Claudeの出力から自動抽出**
   ```rust
   // normalize_logsメソッド内
   if session_id.is_none() {
       if let Some(sess_id) = json.get("session_id").and_then(|v| v.as_str()) {
           session_id = Some(sess_id.to_string());
       }
   }
   ```

2. **実際のClaude出力例**
   ```json
   {
     "type": "system",
     "subtype": "init",
     "session_id": "e988eeea-3712-46a1-82d4-84fbfaa69114",
     "model": "claude-sonnet-4-20250514"
   }
   ```

3. **データベースへの保存**
   ```rust
   // backend/src/executor.rs または各executorの実装内
   ExecutorSession::update_session_id(
       pool,
       execution_process_id,
       &session_id
   ).await
   ```

### セッション継続フロー

```
1. タスク開始
   ↓
2. spawn() → 新規Claudeプロセス起動
   ↓
3. Claudeが session_id を生成・出力
   ↓
4. Vibe Kanbanが session_id を抽出
   ↓
5. ExecutorSession::update_session_id() でDBに保存
   ↓
6. プロセス終了
   ↓
7. ユーザーが追加入力
   ↓
8. フロントエンドが保存済みsession_idを送信
   ↓
9. spawn_followup() → --resume=<session_id> でプロセス起動
   ↓
10. 前回の会話を引き継いで継続
```

### session_idの永続性

- **保存場所**: SQLiteデータベースの`executor_sessions`テーブル
- **紐付け**: `execution_process_id`を通じてプロセスと関連付け
- **取得方法**: タスクやプロセスIDから関連するsession_idを照会可能
- **永続期間**: タスクが削除されるまで保持

## 特徴とメリット

1. **プロセス分離**
   - 各実行が独立したプロセスとして動作
   - エラーや異常終了が他のタスクに影響しない

2. **セッション永続性**
   - session_idにより会話履歴が保持される
   - プロセスが終了しても会話コンテキストは失われない

3. **スケーラビリティ**
   - 複数のタスクを並行実行可能
   - 各タスクが独立したセッションを持つ

4. **ストリーミング対応**
   - JSON形式の出力をリアルタイムで処理
   - WebSocketでフロントエンドにストリーミング配信

## プランモードの特殊処理

プランモード（`ClaudePlan`）では、特別なwatchkillスクリプトを使用：

```bash
# "Exit plan mode?" という文字列を検出したら自動終了
word="Exit plan mode?"
# ... 検出ロジック ...
```

これにより、プランモードの自然な終了を実現しています。