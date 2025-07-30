# Hail Mary - CLI with TUI Support

Go製のCLIツールで、Cobra、slog、Bubbleteaを統合した例です。

## 特徴

- **Cobra**: コマンドライン解析とサブコマンド管理
- **slog**: 構造化ログ（ログレベル設定可能）
- **Bubbletea**: 特定のサブコマンドでTUI（Terminal UI）を起動

## プロジェクト構造

```
hail-mary/
├── cmd/
│   ├── root.go     # ルートコマンドとslog設定
│   ├── ui.go       # TUIサブコマンド
│   └── list.go     # 通常のサブコマンドの例
├── internal/
│   └── ui/
│       └── model.go # Bubbletea TUIモデル
├── go.mod
├── go.sum
├── main.go
└── README.md
```

## インストール

```bash
go install github.com/ashigirl96/hail-mary@latest
```

または、ソースからビルド：

```bash
git clone https://github.com/ashigirl96/hail-mary.git
cd hail-mary
go build -o hail-mary
```

## 使い方

### 基本的な使い方

```bash
# ヘルプを表示
./hail-mary --help

# ログレベルを設定（debug, info, warn, error）
./hail-mary --log-level=debug list
```

### listコマンド（通常のCLI）

```bash
# 基本的なリスト表示
./hail-mary list

# すべてのアイテムを表示
./hail-mary list --all

# JSON形式で出力
./hail-mary list --format=json

# CSV形式で出力
./hail-mary list --format=csv

# デバッグログ付き
./hail-mary list --all --log-level=debug
```

### uiコマンド（TUI起動）

```bash
# TUIを起動
./hail-mary ui

# 初期テキスト付きでTUIを起動
./hail-mary ui --text="Hello, TUI!"

# デバッグログ付き（TUI終了後にログが表示されます）
./hail-mary ui --log-level=debug
```

#### TUIの操作方法

- **Enter**: 入力を確定して終了
- **Esc/Ctrl+C**: キャンセルして終了
- **←/→**: カーソル移動
- **Home/End**: 行頭/行末へジャンプ
- **Backspace/Delete**: 文字削除

## 開発

### 依存関係のインストール

```bash
go mod tidy
```

### ビルド

```bash
go build -o hail-mary
```

### テスト実行

```bash
go test ./...
```

## ライセンス

MIT License