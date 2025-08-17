# Branch Protection Rules 設定ガイド

このドキュメントでは、GitHub リポジトリで `just ci` と `just lint-strict` が成功しないとマージできないように設定する方法を説明します。

## 必要なステータスチェック

以下の2つのGitHub Actionsジョブが成功する必要があります：

1. **CI Checks (just ci)** - フォーマット、基本的なlint、テストを実行
2. **Strict Linting (just lint-strict)** - すべてのターゲットと機能に対する厳格なlintチェック

## 設定手順

### 1. GitHub リポジトリの Settings に移動

1. リポジトリページで `Settings` タブをクリック
2. 左側のメニューから `Branches` を選択

### 2. Branch Protection Rule を追加

1. `Add rule` ボタンをクリック
2. **Branch name pattern** に `main` を入力

### 3. 保護設定を構成

以下の設定を有効にしてください：

#### 必須設定
- ✅ **Require a pull request before merging**
  - ✅ **Require approvals** (推奨: 1以上)
  - ✅ **Dismiss stale pull request approvals when new commits are pushed**
  - ✅ **Require review from CODEOWNERS** (CODEOWNERSファイルがある場合)

- ✅ **Require status checks to pass before merging**
  - ✅ **Require branches to be up to date before merging**
  - 以下のステータスチェックを検索して追加：
    - `CI Checks (just ci)`
    - `Strict Linting (just lint-strict)`

#### 推奨設定
- ✅ **Require conversation resolution before merging**
- ✅ **Require signed commits** (セキュリティ強化のため)
- ✅ **Include administrators** (管理者も規則に従う)
- ✅ **Restrict who can push to matching branches** (必要に応じて)

### 4. 設定を保存

`Create` または `Save changes` ボタンをクリックして設定を保存します。

## 確認方法

設定が正しく機能しているか確認するには：

1. 新しいブランチを作成
2. 何か変更を加えてコミット
3. Pull Request を作成
4. 以下が表示されることを確認：
   - `CI Checks (just ci)` のステータスチェック
   - `Strict Linting (just lint-strict)` のステータスチェック
5. これらのチェックが失敗した場合、マージボタンが無効になることを確認

## トラブルシューティング

### ステータスチェックが表示されない場合

1. GitHub Actions が有効になっているか確認
2. `.github/workflows/ci.yml` ファイルが存在するか確認
3. 最低1回はワークフローが実行されている必要があります
4. ブランチ名パターンが正しいか確認

### チェックが通らない場合

以下のコマンドをローカルで実行して問題を特定：

```bash
# CI チェックを実行
just ci

# 厳格な lint チェックを実行
just lint-strict

# 個別に実行して問題を特定
just fmt-check  # フォーマットチェック
just lint       # 基本的な lint
just test       # テスト実行
```

## Justfile コマンド詳細

- `just ci`: `fmt-check`, `lint`, `test` を順番に実行
- `just lint-strict`: すべてのターゲットと機能に対して clippy を実行（警告をエラーとして扱う）

これらのコマンドがローカルで成功することを確認してからプッシュすることを推奨します。