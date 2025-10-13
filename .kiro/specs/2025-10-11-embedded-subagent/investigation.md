# Investigation

## zellij-command-sender

### Overview

Zellij terminal multiplexerを外部プロセスから制御し、実行中のClaude Codeペインに標準入力を送信する方法を調査。

### Findings

**Zellij CLI Actions (推奨アプローチ)**

- **Finding**: Zellij公式CLIで`write-chars`および`write`アクションが提供されている
- **Source**: https://zellij.dev/documentation/cli-actions.html, Bash実行結果
- **Confidence**: 95%
- **Impact**: 外部プロセスから安全かつ安定的にClaude Codeペインへ入力送信が可能

```bash
# 基本構文
zellij -s <session-name> action write-chars "<text>"
zellij -s <session-name> action write <ascii-code>

# 実用例
zellij -s hail-mary20251010134733 ac write-chars "/spec:status"
zellij -s hail-mary20251010134733 ac write 13  # Enter (CR)
```

**Key Behaviors**:
- セッション指定: `-s`オプションで複数セッション環境に対応
- Enterキー: `write 13` (CR) でコマンド実行、`write 10` (LF) は改行のみ
- フォーカス不要: ペインがフォーカスされていなくても送信可能
- 省略形: `action` → `ac`

**Zellij Rust API (zellij-client)**

- **Finding**: `zellij-client`および`zellij-utils`クレートが存在し、プログラマティックなアクセスが可能
- **Source**: https://crates.io/crates/zellij-client, GitHub zellij-org/zellij
- **Confidence**: 85%
- **Impact**: Rust APIで直接制御可能だが、private APIのため非推奨

```rust
// 内部API（非推奨）
use zellij_utils::ipc::ClientToServerMsg;
use zellij_utils::input::actions::Action;

// ClientToServerMsg::Action with Action::WriteChars
Action::WriteChars { chars: String }
Action::Write { bytes: Vec<u8>, ... }
```

**Limitations**:
- `zellij-client`は公式ドキュメント化されていないprivate API
- Zellijバージョン更新で破壊的変更のリスク
- 公式は`zellij action` CLI使用を推奨

**Zellij Plugin API (zellij-tile)**

- **Finding**: プラグインからのwrite制御は可能だが、外部プロセス制御には不適
- **Source**: https://docs.rs/zellij-tile, 先行調査結果
- **Confidence**: 90%
- **Impact**: プラグイン内部からの制御には最適だが、hail-maryのユースケースには合わない

### Alternative Approaches

**PTY (Pseudo-Terminal)**

- **Finding**: `pty-process`クレートでPTY経由の入力送信が可能
- **Confidence**: 80%
- **Impact**: exec()の代わりにspawn_pty()を使用する必要あり、hail-maryプロセスが親として残る

```rust
use pty_process::Command;

let mut child = Command::new("claude").spawn_pty().await?;
let mut pty = child.pty();
pty.write_all(b"/spec:status\n").await?;
```

**Named Pipes / Unix Sockets**

- **Finding**: FIFOやUnix domain socketsでIPC可能
- **Confidence**: 70%
- **Impact**: 複雑性が高く、今回のユースケースにはオーバーエンジニアリング

### Recommendation

**アプローチ比較**

| アプローチ | 安定性 | 実装コスト | 保守性 | 推奨度 |
|----------|--------|----------|--------|--------|
| Zellij CLI | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ✅ **最推奨** |
| zellij-client API | ⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⚠️ 非推奨 |
| PTY | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | △ 特定用途のみ |
| Named Pipes | ⭐⭐⭐ | ⭐ | ⭐⭐ | ❌ 不要 |

**結論**: Zellij CLI (`zellij -s <session> action write-chars`)を`std::process::Command`でラップする実装が最適

### Implementation Proposal

```rust
// crates/hail-mary/src/infrastructure/process/zellij_command_sender.rs
pub struct ZellijCommandSender {
    session_name: String,
}

impl ZellijCommandSender {
    pub fn new(session_name: impl Into<String>) -> Self {
        Self { session_name: session_name.into() }
    }

    pub fn from_env() -> Result<Self> {
        let session_name = std::env::var("ZELLIJ_SESSION_NAME")?;
        Ok(Self::new(session_name))
    }

    pub fn send_command(&self, command: &str) -> Result<()> {
        Command::new("zellij")
            .args(["-s", &self.session_name, "ac", "write-chars", command])
            .output()?;
        Command::new("zellij")
            .args(["-s", &self.session_name, "ac", "write", "13"])
            .output()?;
        Ok(())
    }
}
```

### Evidence Sources

1. Zellij公式ドキュメント: https://zellij.dev/documentation/cli-actions.html
2. zellij-client crate: https://crates.io/crates/zellij-client (v0.43.1)
3. GitHub source (ipc.rs): https://github.com/zellij-org/zellij/blob/main/zellij-utils/src/ipc.rs
4. GitHub source (actions.rs): https://github.com/zellij-org/zellij/blob/main/zellij-utils/src/input/actions.rs
5. 実機検証: `zellij -s hail-mary20251010134733 ac write-chars` 成功確認
