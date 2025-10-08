# Memo: requirements


新しいslash commandの作成をしたい
名前は `/spec:requirements`

このslash commandの目的は、２つある
- prd: 要望からPRDの生成
- issue: github issueを整理したdocumentの作成
この生成されたドキュメントを元に、この後、調査や、設計、タスク分解をしていく。
なので、このドキュメントの責務は、要望の整理と、要望を実現するために必要な条件の整理

コマンドには、`/spec:requirements --type prd` と `/spec:requirements --type bug` の２つのサブコマンドがある
オプショナルで `--issue [github issueのURL]` もつけられる

> typeが`prd`のとき

- overview
- purpose
- user stories
- acceptance criteria
- technical requirements
- priority
- risk and migrations

> typeが`bug`のとき

- overview
- as is
- to be

かな？他にも必要そうな情報があればほしいけど

---

Behavioral Flowとしては、
1. コマンドが実行されたら、その後、ユーザーに対して、要望の詳細を聞いて、[STOP]する
  - typeが`prd`のとき、どういう新機能開発がしたいか質問をする
  - typeが`bug`のとき、現状どういう状態で、どういう状態になっているべきか質問する
  - [STOP]する
2. ユーザが要望を入力したら、その内容を元に、requirements.mdを更新して、再度正しいかユーザーに確認する
  - ユーザがOKしたら、完了
  - ユーザがNGと何がわからないかを入力して、その情報から再度requirements.mdを更新して、再度正しいかユーザーに確認する

Key Behaviors
- requirements.md の完成度を毎回出力する
- github issueのURLがあれば、github mcpを使って、issueの内容を要約

Boundaries

- Will
 - requirements.mdの更新 
- Will Not
 - 調査、設計、タスク分解をする
 - requirements.md以外を更新する

---

reference:
slash commands: @reference/slash-commands.md
slash command we made: @.claude/commands/hm/steering-remember.md

---

最初に、 <kiro_design_path> に、slash commandのrequirements.mdをどのように設計するか書いてほしい(今回はslash commandなのでRustは関係ない)

<kiro_design_path>には、

```markdown
[requirements.md]
```

と、その解説

を記載しなさい


---

1. PRDとbugのテンプレートを記載するようにする
2. `Bash(date:*): メタデータ用のタイムスタンプ生成` 要らん
3. Will: `参考にしたドキュメントを必ず記載` を追加
4. Will Not: `勝手に Iterative Refinementを終わらせる` を追加
5. `Check if <kiro_requirements_path> already exists` 
    1. CheckじゃなくてReadにする
    2. 現時点でどのような内容が書かれているかを把握する
    的な書き方にする。１つの文章にしても良い
6. Behavioral Flowの(4.)と(5.)の内容が被ってる気がする。Finalizationは、保存じゃなくてSummarizeとかだけでいいかも

---


# Requirements - [Project Name]

## Metadata
- **Completeness**: [0-100%]
- **Source**: [user-input|github-issue: URL]
- **References**: 
  - [List of consulted documents]
  - [Will be populated by /spec:investigate]

## 1. Overview
- Problem statement
- Proposed solution

## 2. Purpose
- [Why this feature is needed]

## 3. User Stories
- As a [user], I want [feature] so that [benefit]
- Priority: [P0/P1/P2]

## 4. Acceptance Criteria
- Given [context], When [action], Then [outcome]
- Edge cases and error conditions

## 5. Technical Requirements
[Will be populated by /spec:investigate]


# Requirements - [Bug Title]

## Metadata
- **Completeness**: [0-100%]
- **Source**: [user-input|github-issue: URL]
- **References**: 
  - [List of consulted documents]
  - [Will be populated by /spec:investigate]

## 1. Overview
- Bug summary
- Severity: [Critical/High/Medium/Low]
- Affected components: [TO BE DETERMINED - requires `/spec:investigate`]

## 2. Current State (As-Is)
- **Steps to Reproduce**:
  1. [Step by step - user provided]
- **Actual Behavior**: What happens
- **Error Messages**: Logs, stack traces
- **Root Cause**: [TO BE INVESTIGATED - requires codebase analysis]
- **Code Location**: [TO BE IDENTIFIED via `/spec:investigate`]

## 3. Expected State (To-Be)
- **Expected Behavior**: What should happen
- **Success Criteria**: How to verify fix
- **Validation Steps**: Testing approach
- **Implementation Approach**: [TO BE DETERMINED after investigation]