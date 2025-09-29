# Memo: enhance-dev-flow


自然言語での呼び出しが可能になったことを受け、新たにtasks.mdを中心とした時系列管理アーキテクチャを導入する。

@crates/hail-mary/src/domain/value_objects/system_prompt/specification_driven_template.md の現在の私が考える課題点

philosophyの記述が正しいかわからない。このspecification drivenの開発において、明示的なworkflowは存在しない。
Reactive(ユーザーの命令に対して、key pattern/recognitionに基づいて行動を決定する)かつNudge(行動後に、次の行動をsuggestionする)的にしたい。

全体的に、`<kiro_tasks>` を中心とした、要件・調査・設計の行動をorchestrationするための仕組みが `kiro specification driven` である、という認識が薄いように感じる。

ドキュメント更新方法についても曖昧だと思っている。
そもそも、このkiro specification drivenにおいて、`<kiro_tasks>` が一番重要なドキュメントとなる。どんな行動を実行する前も、行動した後も、更新される時系列のデーターベース的な立ち位置にないといけない。
しかし、現状だと、そのような意図が伝わりづらい。 `**CRITICAL**: ` 的な文言で、ちゃんと重要性を伝えられる方が良いと思う

あと、PRD, investigation, designのドキュメントそれぞれの更新をする前後で、先に tasks.mdを更新するというような旨も書いてある？めちゃくちゃ大事なんだけど
あと、designは必ずinvestigationを参照するようにしないといけない、という旨も書いてある？めちゃくちゃ大事なんだけど


- それぞれのsectionのレイヤーが分かりづらい

---

My Feedbacks:
- 提案されたnudgedに対して、「いや、追加で調査してほしい」的なことをいえば、tasks.mdの状態に基づいてアクションを決定するわけじゃなくて、先にtasks.mdを更新してから、次の行動を決定する、という流れにしたい
- tasks.mdはユーザーは一切触れないという旨も理解しておく
- `Kiro Workflow Rules` と書くのと、requirements, investigate, designそれぞれが @.claude/commands/hm/interactive-investigate.md のように、`Boundaries` とか `Key Behaviors` を分けて書くのとどちらが良いと思う？
  - `Domain-Specific Documentation Rules` とかも、`Kiro Workflow Rules` とsectionが分かれているのは違和感があるが、あなたはどう思う？
 
My Feedbacks:
- exampleは @experimental/claude-code-opus-system.md のように、 <example>...</example> のように囲うのはどう？
  - 会話のexample
  - 出来あたがったそれぞれのdocument templateのexample



-----------



My Feedbacks:
- １つのtopicに対して、続けて調査を行った場合、同じtopicのsectionに対して追記されるようなpromptになっている？ 逐次同じtopic-nameに対して、質問をされた場合、ユーザーの質問を総合したものを追加する
- ユーザーとの会話で、ユーザーに出力したものを保存するかどうか聞く感じ
- remindで、<steering-XXX>のように思い出させたいから、どのsectionをUserInputのタイミングで検知できるように刺せると良いと思う？ MUSTで従え的な
  - 現状、slash commandから呼び出すようにしたから、要らなさそう！ 

My Feedback:
- 逐次同じtopic-nameに対して、質問をされた場合、ユーザーの質問を総合したものを追加する
- designの説明をもう少し詳しく
- `[Y/n]`じゃなくて、Y or follow-up questionの方がいい
- slash commandに先に、TodoWriteを使って一連の行動を確定させる。的なことを書いたほうがいいか
- ナッジ的反応がない（今、成功したけど、たまたまかもしれないからまだ残しておく）
- requirementsがない状態で、 `design` をされたら、 何を実装したいのか促す

- 対応済み
  - tasks.mdをデフォルトに作成する
  - designに影響するからいるっぽいinvestigation.mdのState Tracking要らない。
  - Last Updated要らない
  - 失敗した時に、諦めない(claude code のissueの問題でダメっぽい)
  - 時間の概念は要らなさそう



---


いきなりdesignをしようとしたら、requirementsを先に定義した方がいいんじゃない？ってnudgingしてほしいし、
requirementsを更新したら、designもcompleteになってるから、更新しますか？


- tasks-orchestration
  - document boundaries 
- nudging before action
  - 提案
- nudging after action
  - 提案 
