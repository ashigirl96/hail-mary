# Investigation: intereactive-summarize

## Claude Code Hooks Investigation

### UserPromptSubmit フックによる自動コンテキスト注入

#### 概要
`UserPromptSubmit`フックを使用することで、ユーザーの入力を毎回インターセプトして、自動的にsteeringファイルなどの追加コンテキストをClaudeに注入できる。

#### 仕組み
1. ユーザーがプロンプトを送信
2. `UserPromptSubmit`フックが実行される
3. フックのstdout出力が**自動的にClaudeのコンテキストに追加される**（特別な動作）
4. Claudeが元の入力＋追加コンテキストを処理

#### 設定例
```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/add_steering.sh"
          }
        ]
      }
    ]
  }
}
```

#### スクリプト例
```bash
#!/bin/bash
# add_steering.sh
if [ -f "$CLAUDE_PROJECT_DIR/.kiro/steering/product.md" ]; then
  echo "===== STEERING CONTEXT ====="
  cat "$CLAUDE_PROJECT_DIR/.kiro/steering/product.md"
  echo "============================"
fi
exit 0
```

### Stop フックによる自動継続と品質ゲート

#### 主な用途
1. **自動継続**: タスクが未完了の場合、Claudeに追加作業を強制
2. **品質ゲート**: テストやリントが通るまで続行
3. **自動レポート生成**: 作業完了後に追加タスクを実行

#### Stop フックの動作
- `decision: "block"`: Claudeの停止をブロックして続行を強制
- `reason`: Claudeに次に何をすべきか具体的に指示

#### 実装例
```python
#!/usr/bin/env python3
import json
import sys

input_data = json.load(sys.stdin)

# 無限ループ防止
if input_data.get('stop_hook_active'):
    sys.exit(0)

# タスクが未完了なら続行を強制
output = {
    "decision": "block",
    "reason": "まだテストが残っています。全てのテストファイルを実行してください。"
}
print(json.dumps(output))
```

### Transcript解析によるClaude出力の取得

#### transcript_pathの活用
Stopフックで受け取る`transcript_path`から、Claude Codeの全ての入出力履歴を取得可能。

#### transcriptファイル形式
- JSONL形式（1行1JSON）
- 各エントリーには以下が含まれる：
  - `type`: "assistant"/"user"など
  - `content`: テキストやツール使用情報
  - `timestamp`: タイムスタンプ

#### 解析例
```python
#!/usr/bin/env python3
import json
import sys

input_data = json.load(sys.stdin)
transcript_path = input_data['transcript_path']

# 会話履歴から情報を抽出
files_modified = []
commands_run = []

with open(transcript_path, 'r') as f:
    for line in f:
        entry = json.loads(line)
        if entry.get('type') == 'assistant':
            for content in entry.get('content', []):
                if content.get('type') == 'tool_use':
                    tool = content.get('name')
                    inputs = content.get('input', {})

                    if tool in ['Edit', 'Write', 'MultiEdit']:
                        files_modified.append(inputs.get('file_path'))
                    elif tool == 'Bash':
                        commands_run.append(inputs.get('command'))
```

### 重要な発見事項

1. **UserPromptSubmitの特殊性**: 通常のフックと異なり、stdoutが自動的にClaudeのコンテキストに追加される唯一のフック（SessionStartも同様）

2. **無限ループ防止の重要性**: Stopフックでは必ず`stop_hook_active`をチェックして無限ループを防ぐ必要がある

3. **Transcript活用の可能性**:
   - セッションサマリーの自動生成
   - 作業内容の追跡とレポーティング
   - 品質メトリクスの収集

### 活用アイデア

1. **動的コンテキスト管理**: UserPromptSubmitで、現在の作業内容に応じて異なるsteeringファイルを自動選択して注入

2. **自動品質保証フロー**: Stopフックで、コード作成→テスト→リント→ドキュメント更新を自動で続行

3. **セッション分析**: transcriptを解析して、作業効率や頻繁に使用するコマンドを分析

