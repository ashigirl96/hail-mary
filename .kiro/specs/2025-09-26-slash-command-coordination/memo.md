# Memo: slash-command-coordination

現在、 
- @.claude/commands/hm/requirements.md がPRDや要件をまとめるためのslash commandの責務
- @.claude/commands/hm/design.md が、設計をまとめるためのslash commandの責務
- @.claude/commands/hm/investigate.md が学習したことをまとめるためのslash commandの責務

となっている。今まで私が開発してたslash commandの課題点は、その一つのコマンドで様々なバリエーションを生み出そうとしていたところにある。
@.claude/commands/hm/design.md をみたらわかるが、What's next? (update/investigate/questions/implement/done) と記載しているように、slash commandを呼び出してから、なにを行動させるか促すようなOrchestrator的な責務も入り込んでしまっている

そこで、私は新たに、ReactiveかつNudge的なsystem promptを考えようとしていた
v2: @.kiro/specs/2025-09-24-enhance-dev-flow/design-v2.md (厳密)
v3: @.kiro/specs/2025-09-24-enhance-dev-flow/design-v3.md (nudge)

つまり、slash commandを使わないで、system promptだけで、claude codeにOrchestrator的な役割と、それぞれの行動の細かなbehavioral flowまでを決めさせることを考えていた

しかし、 @reference/slash-commands.md で新たに `SlashCommand` tool が使えるようになって、明示的に人間がslash commandを呼ばなくても 例えば `run design`というだけで、claude codeが自動で `SlashCommand(/hm:design)` を呼び出してくれるようになった。
これは、つまり、機械的に呼び出すところから、自然言語で呼び出せるようになったことを意味している

ここで、考えたのは、system promptのorchestrator的な責務と、slash commandのを分離することにある。

今、v1, v2を考えた時に、しっかりlayerを分けることで、system prompt内で責務を分けようとしていたが、Slash commandも考慮することで、それぞれの責務が何であるべきかが不明瞭になってきた。

もう少し分解してほしい

現状のslash commandの責務
- trigger
- key patterns
- boundaries(will/ will not)
- tool coordination
- document templates
- behavioral flow(明示的なaction & next stepの提示; orchestrator的な役割)
  - key behaviors 
- examples

まだ実装はしてないが新たに設計しようとしていたv2のsystem promptの責務(厳密にしようとしたversion)
- principles layer
- recognition layer
- pattern layer
- flow layer
- state layer
- suggestion layer
- impact detection layer
- example layer
- template layer
- rules layer(slash commandのboundaries的な感じ)
- prerequisites layer
- dependencies layer

まだ実装はしてないが新たに設計しようとしていたv3のsystem promptの責務(nudge的にしようとしたversion)
- principles layer
- recognition layer
- flow layer
- suggestions layer
- template layer
- dependencies layer

なので、私と話し合いながら、どのような責務分担が良いかをbrainstormingしてほしい

あと、templateを使う事でわかってきた課題として、一回 `/hm:requirements` `/hm:design` でPRDや設計書を作成した後に、追加で仕様を増やそうとすると、全体を考慮して追加というより、
先にできたテンプレートのドキュメントに対して、追加情報的な感じで appendされてしまうという、`/hm:requirements` `/hm:design` 側のtemplateの限界も感じている(`/hm:investigate` は逐次調査した結果を追加していくだけなので現状のまｍで良いんだけどね)
他にも、designしたものが evidenceとして、 investigation.mdのどこに対応しているかを明示的に紐づけることもできていない


My Reviews:

Templateの方について
source: investigation.md#<section name>というのはガチで良さそう
versioningはやっぱりいいんかね？何が追加されたかは把握しやすいよね
ただ、個人的に、requirements.mdやdesign.md自体にversionという概念がなくて、常に完成されたものになっててほしいという感覚もある

どうやってsyncを考えるかについて、
sync slash commandの提案もあるかなと思った
- requirements←design
- investigation→design
ですよね？だから、自動でsyncが発火したら、「requirementsをXXXに更新して」って言ったら、`/hm:requirements`が呼ばれてから、`/hm:sync` が実行されて、designも更新される感じを想像した
ただ、やっぱり、designを更新するかどうかは、userが決めたいので、優先的にsuggestionを出す方がいいよね。
イメージとしては、「requirementsをXXXに更新して」って言ったら、`/hm:requirements`が呼ばれてドキュメントが更新されたあと、「designも更新する必要があるので、先に調査しましょうか」的な

Templateとsync slash commandとかの話を考えてて、
新たにtasks.mdというものを提案したいなという気持ちが出てきた。
versioningを気にしないといけないのは先にrequirements.mdを作って、そこからdesign.mdを作ったけど、その後にrequirements.mdに新たな仕様を追加した時に、design.mdから観た時に、どこが更新されているかわかりづらいところにあると思うんだけど、
tasks.mdがすべての時系列を追える様になっていれば、「
- [ ] XXの仕様を追加されたので、requirements.mdを更新
- [ ] requirements.mdのXXXが更新されたので、それに伴い、調査をしてinvestigation.mdを更新した後　design.mdを更新
」
みたいになっていれば、良いんじゃないかという気持ちになってきた。
これがあれば、すべての作業が時系列で追えるし、どこが更新されたかもわかりやすいし、syncがなくても、依存関係に伴って、何をしなきゃいけないかが明瞭になってくるんじゃないかという気持ちになっている

それで、tasks.mdが実装の順番とかもわかるようになっていれば、実装の優先順位もわかりやすくなるんじゃないかと思う
design.mdとの責務の分離しやすくなりそう
state trackingもさ、tasks.mdの冒頭とかに書いていれば、それぞれのドキュメントに状態を持たせなくても良さそう

---

```markdown
### Priority: P0
- [ ] Setup authentication middleware
  - source: design.md#auth-middleware
  - evidence: investigation.md#jwt-vs-session

### Priority: P1
- [ ] Implement user registration
  - source: design.md#user-registration
  - requirements: requirements.md#user-stories

  System Prompt（超軽量）

<kiro-awareness>
  <!-- 話題認識と自然なルーティングのみ -->
  「要件を更新」→ SlashCommand(/hm:requirements)
  「調査結果」→ SlashCommand(/hm:investigate)

  <!-- tasks.mdから状態を読んでsuggestion -->
After any update → Check tasks.md → Suggest next action
</kiro-awareness>

Slash Commands（単一責任）

/hm:requirements:
- Create/Update requirements.md
- Add entry to tasks.md
- NO orchestration

/hm:investigate:
- Add research to investigation.md
- Add entry to tasks.md
- NO next step suggestion

/hm:design:
- Create/Update design.md
- Link to investigation evidence
- Add entry to tasks.md

/hm:tasks:  # NEW!
- Update state tracking
- Generate change log
- Create implementation tasks
- Show what needs sync
```

これめっちゃよい


```rust
4. tasks.md - Timeline & State Tracker (NEW!)
                                                            
# Tasks - Project Timeline & State
                                                            
## 📊 Document State
| Document | Status | Last Updated | Completeness |
|----------|--------|--------------|--------------|
| requirements.md | complete | 2025-01-26 14:00 | 100% |
| investigation.md | in-progress | 2025-01-26 15:30 | 70% |
| design.md | outdated | 2025-01-25 16:00 | 60% |
                                                            
## 🔄 Pending Impacts
- ⚠ requirements更新(セキュリティ) → design要更新
- ⚠ investigation#performance完了 → design#caching要更新
                                                            
## 📅 Timeline
### 2025-01-26
- [x] 14:00: セキュリティ要件追加 → requirements.md
  - Impact: design.mdの認証部分要更新
- [ ] 15:30: セキュリティ調査中 → investigation.md
- [ ] Pending: design.md更新（セキュリティ反映）
                                                            
## 🎯 Implementation Priority
1. [ ] P0: Authentication基盤
2. [ ] P1: User management
3. [ ] P2: Performance tuning
                                                            
🔧 Slash Command Simplification
                                                            
/hm:requirements:
  責務: requirements.md更新 + tasks.mdに記録
  削除: Orchestration, next step提案
                                                            
/hm:investigate:
  責務: investigation.md追記 + tasks.mdに記録
  削除: Impact analysis
                                                            
/hm:design:
  責務: design.md更新 + tasks.mdに記録
  削除: What's next判断
                                                            
/hm:tasks (NEW):
  責務: tasks.md管理、状態更新、依存関係追跡
                                                            
🤖 System Prompt (Lightweight Orchestrator)
                                                            
<kiro-orchestrator>
  <!-- 軽量な認識とNudging -->
  <behavior>
    - tasks.mdから状態を読み取り
    - 依存関係に基づいてnudge
    - 「設計への影響をtasks.mdに記録しました」
    - 「次のタスクが追加されました」
  </behavior>
</kiro-orchestrator>
```

----

- system promptにドキュメントの責務を記載置きたい
  - tasksは実装順序だけがわかればいい

- 
- Implementation sectionはP0, P1, P2とかも必要ない。あくまで、design.mdに書かれているどのファイルを実装しているか淡々とflatに書いてほしい
- tasksのImplementation sectionは実装順序だけがわかればいい

- tasks.mdに以下必要？
```
## 🔗 Dependency Graph
```
requirements.md#security ─┐
├→ design.md#auth-security → Implementation
investigation.md#security ┘
```

## 📝 Notes
- Design blocked until security investigation complete
- Consider parallel implementation of P1 items 4-6
- Performance optimization deferred to P2
```

イメージなんだけど、↓だとどう？
```markdown
- [x] Initial requirements defined → requirements.md created
- [x] Technology stack researched → investigation.md#tech-stack
- [x] Basic architecture designed → design.md created
- [ ] Setup authentication middleware
  - source: design.md#auth-middleware
  - requirements: requirements.md#security-requirements
- [ ] Implement user registration
  - source: design.md#user-registration
  - requirements: requirements.md#user-stories
- [ ] Review design decisions against investigation findings
```


- system promptにドキュメントの責務を記載置きたい
  - tasksは実装順序だけがわかればいい
  - 副作用として更新されるもの
