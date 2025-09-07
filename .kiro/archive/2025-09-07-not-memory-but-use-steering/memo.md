# Memo: not-memory-but-use-steering

現在のMemory MCPの形態を変えたい。
AsIs:
現在、Memory MCPはsqliteに保存するようにして、generateで .kiro/config.toml にある memory > types に合わせて書き出すようになっている。
問題は、sqliteにどのような内容が書かれているかがわからない
ToBe:
claude-code-specのように、 .kiro/steering で永続化させるようにしたい
しかし、懸念として、このファイルが意図しない状態で上書きされてしまうことだ。

## usecase1

`hail-mary init` をした時に、 .kiro/config.toml に書いている memory > types に従って、.kiro/steering/ 以下にファイルを作成する。
もし、すでにファイルがあれば、上書きしない。それが `--force` オプションがついていても、上書きしない。存在しないファイルがあれば作成する
あと、今回に関係ないが `--force` オプションが付いているとき、 .kiro/config.tomlも上書きしない

usecase3でどのsteeringファイルに入れるべきか判定するために、ファイル名は memory > types の name にcriterion(配列) を追加してほしい

例えば、memory > types に structureがあったとして、 criterionには、

- Root Directory Organization: Top-level structure with descriptions
- Subdirectory Structures: Detailed breakdown of key directories
- Code Organization Patterns: How code is structured
- File Naming Conventions: Standards for naming files and directories
- Import Organization: How imports/dependencies are organized
- Key Architectural Principles: Core design decisions and patterns

のような、criterionを追加する。

## usecase2

新しいMemory MCPでは、.kiro/steering/draft/<記憶のタイトル>.md に逐次保存されるようにしてほしい
なので、Memory MCPではなく、Steeringのslash commandで十分なので、Memory MCPは廃止する

<example1>
ユーザー：constで関数を定義するんじゃなくて、functionを使って定義してほしい
claude code: わかりました。
... 
修正しました。
ユーザー：/hm:steering-remember 
claude code: 了解しました。では、.kiro/steering/draft/const-function-definition.md に保存しますね。
...
</example1>

このようにして、ユーザーがrememberしてくださいと言ったときに、.kiro/steering/draft/に保存するようにしなさい。

## usecase3

新しい、slash commandを作ってほしい。slash commandの作り方は @reference/slash-commands.md や @claude-code-spec/.claude/commands/kiro/steering.md
 を参考にしてほしい
あたらしい /hm:steering は、`draft` ディレクトリにあるものを１つずつ読んで、 すでにある memory > types のどれに該当するかを判定して、追記するようにしてほしい
判定する基準は criterionに箇条書きになっているものを参考にして、振り分けるものを考える

---

わかりやすく上では、 memory > types と書いているが、これからはmemoryじゃなくて、 steering > types にする


--------

> draft配下のファイルは誰の責務？

⏺ 良い質問ですね。draft配下のファイルの責務について、設計を明確にする必要があります。

現在の設計では責務が曖昧なので、以下のように整理すべきです：

責務の分担案

1. Claude Code側（Slash Commands）の責務

- 作成: /hm:steering-rememberコマンドでdraftファイルを作成
- 読み取り: /hm:steeringコマンドでdraftファイルを読み取って分類
- 削除: 分類成功後にdraftファイルを削除

2. hail-mary CLI側の責務

- ディレクトリ作成: hail-mary initで.kiro/steering/draft/ディレクトリを作成
- クリーンアップ: hail-mary steering cleanのようなコマンドで古いdraftを削除
- マイグレーション: 既存のMemory MCPデータをdraftとして出力

推奨する設計

## Draft管理の責務分担

### Claude Code (Slash Commands)
**主要責務**: draftファイルのライフサイクル全体を管理

1. **作成** (`/hm:steering-remember`)
- タイムスタンプ付きファイル名の生成
- コンテンツの保存

2. **処理** (`/hm:steering`)
- draftファイルの読み取り
- 分類アルゴリズムの実行
- ステアリングファイルへの追記

3. **削除**
- 分類成功後の自動削除
- エラー時はdraftに残す

### hail-mary CLI
**主要責務**: インフラストラクチャとメンテナンス

1. **初期化** (`hail-mary init`)
- `.kiro/steering/draft/`ディレクトリ作成

2. **メンテナンス** (`hail-mary steering`)
- `--clean-drafts`: 30日以上古いdraftを削除
- `--list-drafts`: draft一覧表示
- `--migrate`: Memory MCPからの移行

3. **バックアップ**
- draftのアーカイブ機能

理由

Claude Code側にdraft管理の主要責務を置く理由：
1. リアルタイム性: 会話中に即座にdraftを作成・処理する必要がある
2. コンテキスト理解: AIが会話内容を理解して適切に分類できる
3. ユーザー体験: slash commandで完結する方が使いやすい

hail-mary CLI側は補助的な役割：
1. 初期セットアップ: ディレクトリ構造の準備
2. メンテナンス: 定期的なクリーンアップ
3. 移行サポート: 既存システムからの移行

この分担により、日常的な使用はClaude Code側で完結し、hail-mary CLIは管理・保守タスクに専念できます。


-------

designの修正ポイント

1. slash command全体的に

書き方は @/Users/nishimura/.ghq/src/github.com/SuperClaude-Org/SuperClaude_Framework/SuperClaude/Commands/load.md などSuperClaudeのCommandsにあるmarkdown を参考にしなさい
特に、usageやexample、Behavioral Flowかな。実装詳細じゃなくて `Behavioral Flow` にしなさい

あと、すべて英語で書きなさい

2. /hm:steering-remember
- @reference/slash-commands.mdを参考に、 $ARGUMENTSを使わないでください
- argument-hintのoptionの解説を追加
- steering-rememberは会話コンテキストより、既存のsteeringファイルに存在しない学ぶべき内容を保存するためのコマンドであることを明確にする

3. /hm:steering
- argument-hintの書き方合ってる？
- config.toml とか雑に書かないで、明確にファイルを指定する。config.tomlの場合は @.kiro/config.toml とか
- 既存の steeringファイルを .kiro/steering/backup/ にバックアップしてから、WriteToolで追記するようにする。あと、MultiEditを使うようにしなさい

4. 初期の .kiro/config.toml の types

product.md 

Product Overview: Brief description of what the product is
Core Features: Bulleted list of main capabilities
Target Use Case: Specific scenarios the product addresses
Key Value Proposition: Unique benefits and differentiators

tech.md

Architecture: High-level system design
Frontend: Frameworks, libraries, build tools (if applicable)
Backend: Language, framework, server technology (if applicable)
Development Environment: Required tools and setup
Common Commands: Frequently used development commands
Environment Variables: Key configuration variables
Port Configuration: Standard ports used by services

structure.md

Root Directory Organization: Top-level structure with descriptions
Subdirectory Structures: Detailed breakdown of key directories
Code Organization Patterns: How code is structured
File Naming Conventions: Standards for naming files and directories
Import Organization: How imports/dependencies are organized
Key Architectural Principles: Core design decisions and patterns

の３つをデフォルトにしなさい

5. steering.instructions

これはslash command側がやってるから要らない