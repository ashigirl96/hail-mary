# Investigation: interactive-investigate

## user-agent-spoofing

**Date**: 2025-09-21
**Original Question**: @crates/anthropic-client/src/lib.rs で headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")); と clientの偽装をしているのはなんで？

### Round 1: 初期調査

#### User-Agent設定の理由

コード内の `user-agent` ヘッダー設定について調査しました：

```rust
headers.insert("user-agent", HeaderValue::from_static(
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
));
```

**なぜブラウザのUser-Agentを偽装しているのか：**

1. **OAuth認証フローの互換性**
   - このクライアントはAnthropicのOAuth認証を使用しています（`console.anthropic.com/v1/oauth/token`）
   - Anthropic ConsoleのOAuthエンドポイントは、ブラウザベースの認証フローを前提としている可能性が高い
   - 一部のAPIサーバーは、プログラマティックなクライアント（`reqwest/0.12`など）からのリクエストをブロックする場合がある

2. **Claude Codeとの統合**
   - このクライアントは`~/.local/share/opencode/auth.json`から認証情報を読み込んでいる
   - これはClaude Code（ブラウザベースのツール）と同じ認証メカニズムを共有している
   - Claude Codeのブラウザセッションと同じように振る舞うことで、認証の一貫性を保つ

3. **APIレート制限の回避**
   - 一般的なHTTPクライアントライブラリのデフォルトUser-Agent（例：`reqwest/0.12`）は、APIサーバー側で厳しいレート制限を受ける可能性がある
   - ブラウザのUser-Agentを使用することで、通常のウェブトラフィックとして扱われる

4. **セキュリティ対策の回避**
   - AnthropicのAPIは、ボットやスクレイピングツールからのアクセスを防ぐため、User-Agentをチェックしている可能性
   - 正規のブラウザからのリクエストに見せかけることで、これらの制限を回避

### Round 2: OpenCodeの実装調査

**User Question**: opencodeではどのように偽装しているか調査してほしい

#### OpenCodeソースコードのUser-Agent設定調査

OpenCodeのソースコード（`/Users/nishimura/.ghq/src/github.com/sst/opencode`）でUser-Agent設定を調査しました：

**1. WebFetchツールでのUser-Agent偽装**
`packages/opencode/src/tool/webfetch.ts`:
```typescript
headers: {
  "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
  Accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
  "Accept-Language": "en-US,en;q=0.9",
}
```

**2. Anthropic OAuth認証フロー**
`packages/opencode/src/provider/anthropic-auth.ts`では、OAuth認証時にUser-Agentを設定していない。代わりに：
- `authorization: Bearer ${auth.access}` でOAuthトークンを使用
- `x-api-key` ヘッダーを削除（OAuth使用時はAPIキー不要）
- `anthropic-beta` ヘッダーでClaude Code機能を有効化

**3. GitHub API呼び出し**
`cloud/function/src/auth.ts`:
```typescript
headers: {
  Authorization: `Bearer ${response.tokenset.access}`,
  "User-Agent": "opencode",  // シンプルな識別子
  Accept: "application/vnd.github+json",
}
```

#### 比較分析：hail-mary vs OpenCode

| 項目 | hail-mary (anthropic-client) | OpenCode |
|------|------------------------------|----------|
| **User-Agent設定** | Chrome/120.0.0.0 (Mac) | Windows版Chrome/120.0.0.0（WebFetchのみ）|
| **APIアクセス** | Anthropic API直接 | OAuth経由でAnthropic API |
| **認証方法** | OpenCodeのauth.jsonを流用 | 独自のOAuth実装 |
| **GitHub API** | なし | "opencode"として正直に識別 |

### Round 3: 認証フローの詳細確認

**User Question**: ん？hail-maryもopencode auth loginでoauth認証した情報をauth.jsonとして使ってるよね？

#### reqwestのデフォルトUser-Agent動作の真実

テスト結果から判明したこと：
- reqwestは**User-Agentヘッダーを自動で付けない**
- デフォルトで送信されるのは `Accept: */*` のみ
- つまり、User-Agentを明示的に設定しない限り、そのヘッダー自体が存在しない

実際のテスト結果：
```json
{
  "headers": {
    "Accept": "*/*",
    "Host": "httpbin.org",
    "X-Amzn-Trace-Id": "Root=1-68cf9c4a-7f17ca540c2981e3029d5fb3"
  }
}
```

#### 各環境のデフォルト動作

| 環境 | デフォルトUser-Agent |
|------|---------------------|
| **reqwest (Rust)** | なし（ヘッダー自体が送信されない） |
| **Node.js fetch** | なし（ヘッダー自体が送信されない） |
| **curl** | `curl/X.X.X` |
| **Python requests** | `python-requests/X.X.X` |

### Round 4: 実験による検証

**User Request**: 2, 3番にして、実際に試してみて(anthropic-clientのexampleあるからそれを実行してみて)

#### 実験結果

| User-Agent設定 | 結果 | エラー内容 |
|---------------|------|-----------||
| **空文字列** `""` | ❌ 失敗 | 403 Forbidden (Cloudflare CAPTCHA) |
| **シンプル** `"hail-mary"` | ✅ 成功 | 正常に動作 |
| **なし** (ヘッダー自体なし) | ❌ 失敗 | 403 Forbidden (Cloudflare CAPTCHA) |
| **ブラウザ偽装** (元の設定) | ✅ 成功 | 正常に動作 |

#### 判明した事実

1. **Cloudflareの保護**
   - Anthropic APIは`console.anthropic.com`でCloudflareによって保護されている
   - User-Agentヘッダーが**存在しない**または**空**の場合、CAPTCHAページにリダイレクト

2. **User-Agent要件**
   - 何らかの**非空のUser-Agent**が必要
   - `"hail-mary"`のようなシンプルな文字列でも動作する
   - ブラウザを偽装する必要は**実際にはない**

3. **reqwestのデフォルト動作**
   - reqwestはデフォルトでUser-Agentを送信しない
   - これがCloudflareのボット検出に引っかかる原因

4. **OpenCodeとの違い**
   - OpenCode (Node.js/Bun)も同じくデフォルトでUser-Agentを送信しない
   - でもOpenCodeのコードにUser-Agent設定がない理由は不明
   - 可能性：Node.jsランタイムやブラウザ環境では別の仕組みがある？

### 結論

**hail-maryがUser-Agent偽装する理由**：
- Cloudflareのボット検出を回避するため
- 空でない任意のUser-Agentで十分（ブラウザ偽装は過剰対応）
- より誠実な実装は`"hail-mary/0.1.0"`のようなカスタムUser-Agent

推奨される修正：
```rust
headers.insert("user-agent", HeaderValue::from_static("hail-mary/0.1.0"));
```

これで、セキュリティを回避しつつ、正直にクライアントを識別できます。

