# Memo: project

crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/00_philosophy.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/01_principles.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/02_hub.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/03_patterns.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/04_workflows.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/05_gates.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/06_nudges.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/07_requirements.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/08_investigation.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/09_design.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/10_spec_files.md
crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/index.md
crates/hail-mary/src/domain/value_objects/system_prompt/mod.rs
を全て読んだ上で、 Pattern Router Framework(@crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/README.md) がなにか理解してください

@crates/hail-mary/src/cli/commands/code.rs, @crates/hail-mary/src/application/use_cases/launch_claude_with_spec.rs を使うと、Pattern Router Frameworkをsystem promptとして埋め込んで、
@.claude/commands/hm/design.md
@.claude/commands/hm/requirements.md
@.claude/commands/hm/investigate.md
@.claude/commands/hm/timeline.md
などを使って、spec drivenな開発ができるところまで理解しなさい

現在、Pattern Router Frameworkは期待通りの挙動をしている。
このspec drivenで開発することで1つの明確なissue/PRDに対してをrequirements/designとして表現することで、開発に進むことができる。
なので、現在この Pattern Router Frameworkを使ってspec driven developmentは1つのPull Requestを作成するまでをusecaseとしている。
ここわかりますか？

しかし、場合によっては大きなプロジェクトなどで、複数のPull Requestに分割する必要があるものもある。例えば、長期的に運用する必要がある場合や、backendとfrontendで分けて開発する必要がある場合など
そして現在のPattern Router Frameworkはこの複数のPull Requestに分割することをサポートしていない。
なので、これをサポートするには、大きく変える必要があると考えているので、私が考えている案に対して、brainstormしてほしい。

改めておさらいだけど、
`hail-mary code`をした時点で
```
.kiro/
  specs/[feature-name]/
    tasks.md
    memo.md
```
が生成され、`/spec:requirements --type prd` をすると

```
.kiro/
  specs/[feature-name]/
    requirements.md
    tasks.md
    memo.md
```

とrequirements.mdが生成され、 さらに`/spec:investigate` をすると

```
.kiro/
  specs/[feature-name]/
    requirements.md
    investigation.md
    tasks.md
    memo.md
```

のように、ファイルが作成されていく。design.mdが最終ゴールみたいなところ

ここまで一旦理解できてる？

----

今回考えているのが、やっぱり最初は同じで
`hail-mary code`をしたら、以下のようなファイルが生成される。
```
.kiro/
  specs/[feature-name]/
    tasks.md
    memo.md
```

そして、userとclaude codeが議論していってある程度固まったり、github issue自体に全容が把握できるような仕様が書いてあって、`/spec:requirements --type pbi` をしたら、
```
.kiro/
  specs/[feature-name]/
    requirements.md
    tasks.md
    memo.md
```
とrequirements.mdが生成される。ただ、このrequirements.mdには、 @crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/07_requirements.md で書かれているようなtemplateではなく、

```markdown
## Product Backlog Itemの概要
[PBIの概要を記載]
## sbi-1-[sbi-title]
requirements type: [prd/bug/tech]
[sbi-1 description]
## sbi-2-[sbi-title]
requirements type: [prd/bug/tech]
[sbi-2 description]
## sbi-3-[sbi-title]
requirements type: [prd/bug/tech]
[sbi-3 description]
...
```

といったように、複数のsbi(Sprint Backlog Item)のtitleとdescriptionが書かれるようになる。ここで例によってadaptiveにuserと議論しながら、sbiを切る単位を決めていく感じにする

そして、分割する単位を決めたら、 `/hm:sbi --decompose` (もっといい案があるかもしれない) を実行すると、

```
.kiro/
  specs/[feature-name]/
    requirements.md
    tasks.md
    memo.md
    sbi-1-[sbi-title]/
      requirements.md
      tasks.md
      memo.md
    sbi-2-[sbi-title]/
      requirements.md
      tasks.md
      memo.md
```

が生成されるようにする。各、requirements.mdには、sbi-1の詳細な仕様(PRD/bug/techのformatはPBIのrequirementsを参照)が書かれるようにする。
この時点で、分割するというusecaseが完了する。

ここまでの私のアイデアに対する評価をして、続けてbrainstormしてほしい。

---

そして、各sbiのrequirements.mdをclaudeと議論しながら、investigation, design, tasksを進めていく感じにする。
ただ、現在の `hail-mary code` では、specをネストしているものは選択できないので、

```markdown
🚀 Launch without specification
📝 Create new specification
   2025-10-07-project
>    sbi-1-[sbi-title]
     sbi-2-[sbi-title]
     sbi-3-[sbi-title]
     📝 Create new SBI specification

```

みたいな感じに、sbiを選択できるようにする必要がある。ここで `Create new SBI specification` を選べるようにしたいのは、最初に `/hm:sbi --decompose` をした時点で、見つけられなかったsbiを後から追加するedge caseをサポートするため

そして、SBIを選択したら、通常通りのspec driven developmentができるようになる
現在、私が考えている留意点として、 @crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/10_spec_files.md で書かれている、
- <tasks-file>{tasks_path}</tasks-file> - Task tracking and timeline
- <requirements-file>{requirements_path}</requirements-file> - Requirements and user stories
- <design-file>{design_path}</design-file> - Technical design and architecture
- <investigation-file>{investigation_path}</investigation-file> - Research findings and evidence
とは別に、 `<pbi-file>{pbi_path}</pbi-file> - Product Backlog Item ...` が追加した新しいファイルを追加する。 
どういうことかというと、

@crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/index.md で

```markdown
<kiro-spec-files>
{spec_files}
</kiro-spec-files>
```

と言ったように、 @crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/10_spec_files.md を組み込むようになっているので、SBIを選択したときだけ、 10_spec_files_sbi.md という名前にしたやつを組み込むように `hail-mary code`でSBIを選択したときに選ばれるようにしようかなと考えている

あなたの提案してくれた `tasks.md の2層構造` はとても魅力的なんだけど、そのためには 02_hub.md で新たに管理するものが増えそうだし、 04_workflows.md でもpbi側を考慮したworkflowの内容が増えそうでちょっと懸念している。
他にも提案してくれた、 `/hm:sbi add sbi-4-monitoring        # 新SBI追加` はとても良さそう。自身で明示的に追加させてから、PBI側の requirements.md にも追加させる方法と、interactiveにSBIを増やせる方法の2つをサポートするのは良さそう

私が考えているのは、通常のcaseは現行の挙動のままにして、SBIを複数作る場合には、拡張したファイル(10_spec_files_sbi.md)を生成するという方針にしようかなと思っている

ここまでであなたの理解と、さらにbrainstormしてほしい 

---
My Feedback:
1. `<pbi-tasks-file>{pbi_tasks_path}</pbi-tasks-file> - SBI progress checklist` は要らないと思う。それをどうやって管理するかっていう情報は誰が面倒を見るの？
2. だから `PBI Tasks.md - Simple Checklist` のチェックをつけるとかは誰がやるの？
3. `07_requirements.md` と、 slash commandの requirements.md のargument hintにも追加する必要ある
4. " - Copy `tasks.md`, `memo.md` (initial state)"は要らないかも
5. `**Update PBI tasks.md**` も要らない。これをチェックしたりするためのpromptはないから
6. `application/use_cases/decompose_pbi.rs` は要らないかも
7. `hail-mary archive --sbi sbi-1-backend-api  # Advanced`も一旦要らないかも