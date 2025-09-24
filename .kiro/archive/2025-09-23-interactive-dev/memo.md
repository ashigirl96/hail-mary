# Memo: interactive-dev


## issue 
@.claude/commands/hm/design.md はバックエンド系の開発に役立つんだけど、フロントエンド開発の場合は、人間が設計を作るほうがいい
理由は、仕様書(requirements.md)に詳細なUI情報が載っている場合は、@.claude/commands/hm/design.md で実装しやすいんだけど、Figmaとかにusecaseとかが載っている場合は、逐次に開発者がFigmaや開発者が持つ情報を与えつつ、step-by-stepで設計書(design.md)を作成して、実装をしていきたいんだよね

----

# /hm:interactive-dev

allowed-tools: Read, Write, MultiEdit, Grep, Glob, Task, WebSearch, mcp__context7__*, mcp__sequential-thinking__*, mcp__figma__*

Key Patterns:
interactiveに 質問に答える、調査、設計、設計の更新、実装 の何をしたいかを常にユーザーに委ねる
基本的なFlowは、質問→調査→設計保存→質問→...→終わったら実装を促すなんだけど、完全に全部の設計が終わっていなくても部分的に実装をしたい場合もあるし、「設計してくれたけどXXはどうなってるの？」という質問に対して必要なら調査をして、設計を修正をしたり、既に設計がユーザーの要望を満たしていたら理由をつけて解説をしたり、強力な柔軟性を持つ

前提として、@.claude/commands/hm/design.md は一気にdesign.mdを作成するが、このslash commandでは、Phase by Phaseで設計を進めていく

document template

```markdown

## Phase1: <section-name>

[design content]

### tasks

- [ ] <task>
- [ ] <task>
...

## Phase2: <section-name>

[design content]

### tasks

- [ ] <task>
- [ ] <task>
...

...
```

Behavioral Flow:
 @.claude/commands/hm/design.md 同様に、<kiro_requirements>, <kiro_investigation>と<kiro_design>を最初に読み込む
<kiro_requirements>がなかったり、仕様の情報が空だった場合は、ユーザーに、何を開発をしたいか質問をする
 <kiro_design>が空なら、<kiro_requirements>からどういう設計が良さそうかFollow up Questionを出す。<kiro_design>が空でなければ、<kiro_requirements>を満たすためになにを設計するべきかFollow up Questionを出す
Follow up questionとともに、ユーザーの質問に答える、調査、設計、設計の更新、実装の候補を出す

設計の更新を促すには、 ユーザーの質問・指摘からcodebaseを調査をしたり、現在保持している情報からthinkして更新した方がいいと考えた後にする 。「Phase N: <section-name>の内容AsIsがXXX(詳細)で、ToBeをYYY(詳細)にしますか？」みたいな感じに出す
実装を選択肢の候補煮出すのは、そのPhaseの設計ができて「Phase N: <section-name> の実装をしますか？」みたいな感じに出す

Key Behaviors:
起動後に<kiro_design>を読み込んだ後や、<kiro_design>を更新した後に、<kiro_design>の情報を元にFollow up Questionを出すが、常に、優先順位の高い順に出す。backendだったら、domain→application→infrastructure→presentationの順に出すみたいな。frontendだったら、hook→componentの順に出すみたいな

Boundaries:

Will:
- **Interactive Loop**: ユーザーの期待する機能が全て完成するまでずっとループし続ける

Will Not:
- <kiro_tasks>を考慮しない。この<Kiro_design>にtasksを含める

------

reference: @.claude/commands/hm/investigate.md

私の言っていることを理解してください