# Memo: task-orchestration

正直、私は現在のsection構成に疑問を持っている。なので、specification_driven_template.mdを元にした、新たな `kiro.md` を作成したい。
しかし、一気に修正したくないので、sectionごとに段階的に追加していく方針にしたい
なので、まずは私とbrainstormingをしながら、相談しましょう。私が実装してくださいと言うまでは実装を促さないでください
わかりましたか？

philosophyは基本的に良い。しかし、Reactive Orchestration Patternの箇所が疑問である

いや、別に80% / 20%にこだわらなくても良い気がする
現在は、
```
User Input → Pattern Recognition → Consult tasks.md state → Determine action →
Update <kiro_tasks> (BEFORE) → Execute action → Update <kiro_tasks> (AFTER) → Nudge suggestion
```
だが、
1. User Input
2. Pattern Recognition
3. Consult tasks.md state
4. validation
5. Determine action
6. Update <kiro_tasks> (BEFORE)
7. Execute action
8. Update <kiro_tasks> (AFTER)
9. suggestion? recommendation?
の順番になるべきだと思う

あと、nudgeを「gentle Guidance」みたいな書き方をしているが、要は、行動を促すぐらいのニュアンスにしたい。
つまり、userのinputに対して、tasks.md stateから期待した行動に一致していなければ、正しい行動に誘導する、みたいなニュアンスにしたいんだよね
これが、非線形で、reactiveで、nudge suggestionで、tasks.md orchestrationというこのframeworkの本質だと思う

---

私も凄く悩んでる
基本、このframeworkで管理しているのは、tasks.md, requirements.md, investigation.md, design.mdの4つだよね
requirements, investigation, designには、boundaries, key behaviors, templatesがあるよね
それに対して、tasks.mdは、orchestrationの役割も持つから、boundaries(責務), key behaviors(pattern recognitionからどういう行動をさせるか; BEFORE/AFTERそれぞれでactionも異なる), templates, validationのrule, recommendationのruleというか今わかっている促したいnext action(e.g. requirementsが確定してないのに、designを始めようとしているなら、まずrequirementsを確定させるべきだよね、みたいな)も持つべきだと思う

どういうsection構成が良いか、brainstormingしましょう

---

principlesに、orchestrationに全体ルール(静的なもの？)、patternsが(動的？; ドキュメント間の連携振る舞い)

明日は、
1. kiro-requirements, kiro-investigation, kiro-designの3つを作成する
2. kiro-tasksも作る
3. 
それから、


1. principles: 普遍的な運用ルール（WHAT must always be true）
2. hub: tasks.mdの具体的な構造と役割（HOW tasks.md works）
3. patterns: ユーザー入力の認識（WHAT triggers what）
4. workflows: 操作のタイミングと実行（WHEN and WHAT to do）



- `/spec:requirements` をいきなり開始したら、どうなる？
- tasks.mdに既にrequirementsがcompleteになっている状態で、続けて、仕様を追加したい場合は？
- designをいきなり開始したら、どうなる？ 
- requirementsやdesignに関係ない話題で `/spec:investigate` を開始したら、どうなる？
- investigationをいきなり開始したら、どうなる？
- claude codeとuserが対話してる中で、requirementsに関する議論なので、requirementsを更新する？とsuggestionをclaude codeが出すか
- claude codeとuserが対話してる中で、調査結果に関する議論なので、investigationを更新する？とsuggestionをclaude codeが出すか
 
のように、このreactive-pattern based orchestrationの柔軟性を加味した上で、どれぐらいあらゆる想定をカバーできているのか、考えたい
まず、上記のようなケース以外にも、開発者がspec drivenな開発をするうえで、どのようなcaseがあるか50個ぐらい挙げてほしい

---
現在のフローは単一的で、explicitなケースしか考慮できてない