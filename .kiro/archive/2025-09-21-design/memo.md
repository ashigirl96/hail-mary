# Memo: design

Key Patterns
- <kiro_requirements>がある場合 → Detailed Design Document Templateに従って作成
- 要件が複雑な場合 → Detailed Design Document Templateに従って作成
- <kiro_requirements>がなく、単純な要件な場合 → Simple Design Document Templateに従って

Boundaries
Will:
- userがreviewするため、Design sectionには必ず修正するファイルとその修正内容についてすべて記述する
- 調査するときは、general-purpose agentを使って、comprehensiveな調査を行う
- `/spec:investigate`コマンドの責務は、ユーザーの調査不足による不安を解消するためであって、このコマンドで行う調査は、designを作成するために必要な調査に限定する
- Think in English, but document in Japanese

Will Not:
- どれぐらいかかるとかの情報を記載する
- リリースの手順についての記載をする

Behavioral Flow
1. 現状把握
   - <kiro_design>, <kiro_requirements>と <kiro_investigation> をReadして現状把握をしようとする 
   - [STOP AND WAIT FOR USER INPUT] して、次のアクションについてのsuggestionを出して、ユーザーに確認する。候補として、
     - <kiro_requirements>が存在しないし、<kiro_design>の記載もない場合 → ユーザーに要件を尋ねる
     - <kiro_requirements>が存在し、<kiro_design>の内容がなかった場合 → designを作成するか尋ねる
     - <kiro_requirements>が存在しないが、<kiro_design>の記載が十分な場合 → ユーザーに次に何をしたいか尋ねる(設計の修正したいのか、追加で調査がしたいのか、設計について質問をしたいのか、実装をしたいのか、designコマンドとしての責務が終わっているので終わるか等)
     - <kiro_requirements>が存在し、<kiro_design>の記載も十分な場合 → ユーザーに次に何をしたいか尋ねる(設計の修正したいのか、追加で調査がしたいのか、設計について質問をしたいのか、実装をしたいのか等、designコマンドとしての責務が終わっているので終わるか等)
2. ユーザーの指示に従う
   - ユーザーが要件を提供した場合 → comprehensiveな調査をしてからdesignを作成する
   - ユーザーがdesignを作成したい場合 → <kiro_requirements>の要件と、<kiro_requirements>を満たすために必要な情報<kiro_investigation>の内容を理解した上で、 designを作成する
   - ユーザーが設計の修正をしたい場合 → 調査が必要なら調査をしてからdesignを修正する
   - ユーザーが追加で調査がしたい場合 → designを完璧になるまで調査をする
   - ユーザーが設計について質問をしたい場合 → 質問に答える、ただし、調査が必要な場合は調査もする
   - ユーザーが実装をしたい場合 → 実装を開始する
   - ユーザーがdesignコマンドとしての責務が終わっているので終わるか等の場合 → 終了する

Key Behaviors
- designについて聞かれたり、修正を依頼された時に行った調査は @.claude/commands/hm/investigate.md のBehavioral Flowに従って調査を行う。(`/spec:investigate --for design`が実行される)

Document Template

### Detailed Design Document Template

````markdown
## Meta
- **Completeness**: [0-100%]
- **requirements**: [要件を簡潔に記述]
...
  
## Overview
[今回の修正内容の概要; As-Is/To-Beを大まかに説明] 

## Design


今回の修正内容の詳細; どのファイルをどのように修正するか具体的に説明

### [修正対象のファイル1] 
[現状の問題点や不足点を具体的に説明と修正後の状態や追加する内容を具体的に説明]

[どういうコードに修正するかあるべき姿をすべて記載]

### [修正対象のファイル2]

...



## References(今回修正対象のファイルを列挙)

---

## Completeness Scoring Rule
...
````

### Simple Design Document Template

````markdown
...
````

