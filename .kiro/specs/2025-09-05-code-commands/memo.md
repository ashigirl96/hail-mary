# Memo: code-commands

`hail-mary code` というサブコマンドを追加したい。このサブコマンドを実行すると、claude codeを実行する。つまり、`claude` というコマンドをこのhail-mary codeを実行することで実行される
claude には `--append-system-prompt <prompt>                   Append a system prompt to the default system prompt`  というオプションがあり、システムプロンプトを追加することができる
私は、`hai-mary code` を実行したときに、明示的に現在どのタスクをフォーカスしてclaude codeがタスクを遂行するべきかシステムプロンプトに追加したいと考えている。
例えば、 `2025-09-05-code-commands` というタスクをやるときに、必要なコンテキスト情報は `.kiro/specs/2025-09-05-code-commands/` にある。

❯ ls .kiro/specs/2025-09-05-code-commands
design.md // comprehensive technical design for a specification
memo.md  // user memo
requirements.md  // comprehensive requirements for a specification
tasks.md // implementation tasks for a specification

があり、それぞれの役割がある。なので、システムプロンプトには、どのファイルパスにはどういう役割があるか明示されててほしい

そして、`hail-mary code` を実行したときのワークフローは、

1. `hail-mary complete` のように、`.kiro/specs` 配下のディレクトリ一覧を検索できるようにして、「あなたはどのspecをやりますか？」選択できる。
  - `complete` との違いは、新しいspecから始めたい場合もあるので、 `hail-mary new` のように、新しいspecを作る選択肢も追加してほしい。これを選択したら、名前を入力して、`hail-mary new` と同様に`.kiro/specs/` ディレクトリ配下に新しいディレクトリやファイルを作成する
2. 選択されたspecのファイルパスの情報をシステムプロンプトをに追加した状態で、`claude code`を立ち上げるようにする。つまり、`claude --append-system-prompt ...` の ...にファイルパス情報が記載される

---

この後、claude codeのコマンドも実装したい。例えば、

`/requirements 〇〇を実装したい` をしたら、claude codeのコマンド内では、


```
対象ファイル: <kiro_requirements_path />
```

と汎用的なXMLタグにしたいため、kiro_requirements_pathをシステムプロンプト内に書かれてほしい

e.g. 


<kiro_requirements_path>.kiro/specs/2025-09-05-code-commands/requirements.md</kiro_requirements_path>

みたいな

---

私は特に、システムプロンプトの書き方がよく分かっておらず、context7などで良いシステムプロンプトの書き方を調べてほしい



---

- `Create new specification` で作ったものからclaude codeが実行されない。❌ Error: Specification XXX not found と表示されてしまう
- claude codeで `ctrl+z`すると、バックグラウンドになるはずなんだけど、hail-mary codeを実行する親プロセスのせいで、backgroundにならない