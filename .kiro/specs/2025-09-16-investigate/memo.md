# Memo: investigate

**新しいslash command(`/hm:investigate` )を作成したいので、相談に乗ってください** 。
objective:
- `/hm:requirements` (; @.claude/commands/hm/requirements.md ) でPRDやbug reportの追加調査するため
- この後作る `/hm:design` (まだ作ってない) で設計をするための調査するため
- requirementsやdesignとは関係なく、codebaseやWebSearchやcontext7の情報から調査をするため

盛り込みたい機能:
- @.claude/commands/hm/steering-remember.md のようにformatは自動で選択されるようにしたい
- コードベースからの調査と、context7やWebから検索したドキュメントの情報はフォーマットをタイプで選べるし、
- `/hm:requirements` のように、interactiveに調査を進める
- Yなら終わるし、nの代わりに追加調査をさせる
- Data FlowはMermaidで書く
- 調査の優先度 system promptにある<steering>tagの中身 > codebase > context7 > web
- どこのリソースからデータを取ってきたか
- <kiro_investigation_path>に調査結果を保存する
- `/hm:investigate --topic` したら、
  - そのtopicに関する調査をする。
  - そこから、インタラクティブに会話をしながら、その `topic` が１つのsectionになるようにしたい
  - また `/hm:investigate --topic "YYY"` をしたら、以前の分客とは切り出して、新たなtopicとして、調査をして、それがまた１つのsectionとなるようにする
- @.claude/commands/hm/steering.md のように、Task tool(subagent)を使って並列で調査をする。調査するときに、それぞれのTask Toolがなにを調査するかのプランも表示されていると嬉しい

reference:
- @reference/slash-command-structure.md の構造を参考にしたいので、 @.claude/commands/hm/requirements.md が良いと思う
- @reference/slash-commands.md
- @.claude/commands/hm/steering-remember.md

また決めきれていないこと:
- 調査において、どういうフォーマットがありそうか; conciseで、必要な情報が揃っている。行の長さは調査の複雑さによって変わるようにしたい
- `/hm:requirements` と `/hm:design` との関係性。可能な限り、 `/hm:requirements` と `/hm:design` はそれぞれ独立して、documentを作ることに集中して、 `/hm:investigate` がそれらの情報を補完するような役割をする？
  -  `/hm:investigate --requirements --topic` にしたら、<kiro_investigation_path> にも書き込むし、その後に、 <kiro_requirements_path> にも書き込むようにする?
  - `/hm:investigate --design --topic` にしたら、<kiro_investigation_path> にも書き込むし、その後に、 <kiro_design_path> にも書き込むようにする?
  - ↑はもっとgood ideaがあれば、提案がほしい
- `--topic` と書いてきたけど、調査してほしい内容を1行でまとめるのって難しいから、
  ```
  /hm:investigate --topic
  > 新たに調査してほしい内容を教えてください:
  [user input]
  ```
  とかのほうが良いのかな？そして、claude codeがconciseなタイトルを考えて、それをsection titleにする？
- どうすれば、 <steering> tagの中身を優先的に見てくれるか


---

My Feedbacks:
- sourcesも自動で steering > codebase > context7 > web と調査の優先度を決めてほしいので、明示的に指定はしたくない
- Problem Analysis Formatをベースにしてもいいかも。Evidenceとして、Technical PatternやKnowledge SummaryやArchitecture Flowが入っててもいいし、Root CauseにArchitecture FlowやData Flowや、code snippetsが入っててもいいし、という感じで臨機応変にしてもいいかもね？
- この <kiro_investigation_path> に保存されるんだけど、どんどん蓄積(append)していくイメージね。 interactiveに同じtopicを話しているときは、同じsectionにupdateしていく感じかな。「前の調査はこうだったけど、実はこうだったかも」みたいなmemoもあってもいいけど
- Behavioral Flowで、会話しながら <kiro_investigation_path> に保存して、「まだ同じtopicについて話続けるかどうか」聞く感じがいいね。最後に Documentationで `--for` optionで requirements.md や design.md が保存されているのはいいね！

My Feedbacks:
- Key Patternsに、「調査依頼内容からdepthのstandardかdeepを決める」的な文言ほしい
- investigation.md → <kiro_investigation_path>
- requirements.md → <kiro_requirements_path>
- design.md → <kiro_design_path>
- Document Templateに `Metadata` は不要
- Topicの以下も不要
  **Investigated**: [timestamp]
  **Last Updated**: [timestamp]
- markdownのnestedの書き方をちゃんとして


My Feedbacks:
- `**Progressive Save**: Write after each round, not just at end` は明示的に、 <kiro_investigation_path> に保存するって書いてほしい。`--for` がある場合は、終わった時に、 <kiro_requirements_path> や <kiro_design_path> にも保存するって書いてほしい
- `[hint]`をなくして `--topic [topic name]` optionをつけてほしい。`--topic [topic name]` のtopic nameが付いているときは、topic nameに該当するtopic を更新してほしいし、topic nameがない場合は必ず新しいsectionを作るって書いてほしい。Will/Will Notと、Key Behaviorsに書く(他にも修正が必要なところはあるかもしれないので、MultiEditして)


**新しいslash command(`/hm:investigate` )を作成したいので、相談に乗ってください** 。

現在、 <kiro_design_path> に設計書を書いているので、理解してください

reference:
- @reference/slash-command-structure.md の構造を参考にしたいので、 @.claude/commands/hm/requirements.md が良いと思う
- @reference/slash-commands.md
- @.claude/commands/hm/steering-remember.md


---

- Task Toolの話もしたい
- `Example 3: Multi-Topic Session` がおかしい。`/hm:investigate --topic` で会話して、保存が終わった後に、また `/hm:investigate --topic` が実行されたら、「前のtopic？」と聞くんじゃなくて、新しいtopicを調査したい



`問題: これらは「実行の詳細」であり、Boundaries の責務（何をする/しない）を超えている。Behavioral Flow に属する内容` をより詳細に書いてほしい
Task agentsを使うべきとか、調査の後に保存とかは確かに実行的な話でもあるけど、責務としてやってほしいことを書いておかないと、忘れそうじゃない？

じゃあ、以下は修正して
- Key Patternsの重複箇所
- Document Templateの位置


---

- /hm:investigate しただけなのに、requirements.md とかdesign.mdを見に行ってた。 `--for` をつけてない場合は探さないでほしい（Will Notに書く内容？)
- 以下のように表示されているのに、
```
Launching parallel investigators:
• [Code Explorer] Search implementation in codebase
• [Docs Researcher] Query Context7 for best practices
• [Web Searcher] Find recent solutions and updates
`⏺ steering-investigator(Search Anthropic client headers)` 
```
  と１つだけしかsteering-investigatorしか呼び出されていない。
  複数subagentを呼び出すなら、なぜそのsubagentを呼び出すのか理由の記載と、呼び出すって言ってるならちゃんと呼び出すようにしてほしい
- subagentを作る、切り分ける
- `--for` flagが付いているときは、勝手に調査を開始するんじゃなくて、プランを提示してから
- 現在、フォーマットに合わせた結果、欲しい情報に対して、フォーマットが邪魔をしている感じがする。
  - なので、フォーマットに従うという項目は消して良い(sectionごと消して良い)
  - 例えば、Data FlowをMermaidで書くのは良いけど、Evidenceに入れたいし、Root Causeにも入れたいし、Recommendationsにも入れたい。Code Snippetsも同様
  - code snippetsがある場合は、意味について説明を書く
  - root causesについて書くときは、必ず Recommendations も記載する
