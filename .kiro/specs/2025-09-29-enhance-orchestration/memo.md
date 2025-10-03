# Memo: enhance-orchestration

ここから、私とbrainstormingしてください

現在、tasks, requirements, investigation, designのmarkdownでそれぞれの責務に関するドキュメントを記載してますよね

しかし、場合によっては、
- 非常に大きい仕様(数ヶ月にわたるプロジェクト)
- 1つのissueに複数のバグが記載されている

みたいな問題がある時に、現在のframeworkだとそれぞれのファイルの内容がfatになりすぎたり、人間の見通しも悪かったり、リリース単位で管理す
るのが難しいという問題があると思う
ここまで、私の言っていることに相違があるかどうか考えてください
間違っている可能性もあるので、疑いながら考えてください(YOU MUST NOT HOLD BACK. GIVE IT YOUR ALL. you think in english, but return
final output in japanese --ultrathink)

1, 3の方向性を統合したものが良い気がするね

parent-specification/
  - tasks.md
  - requirements.md
    - 子仕様へのリンク
    - 子仕様の要約
  - design.md
    - 子設計へのリンク 
    - 子設計の要約
  - investigation.md
    - childに分解する際に必要だった調査 
  - 01-child-specification/
    - tasks.md
    - requirements.md
    - investigation.md
    - design.md
  - 02-child-specification/
    - tasks.md
    - requirements.md
    - investigation.md
    - design.md

次の実現可能性の問題として、我々は現在 @crates/hail-mary/src/domain/value_objects/system_prompt/orchestration/index.md を動的にsystem promptに埋め込んで、<kiro-spec-driven> がactivationされている

そして、今までの、hub, workflows, nudgesなどの責務が大きく変わるはずになる

ここで私が考えたのは、既存のmarkdownの修正は以下だけ
- requirements→investigationまでして、「設計しますか」or「specを分解しますか？」に切り替える
- specを分解するにしたら、現在のspecディレクトリの中に複数子のspecificationディレクトリを作成する
- それぞれの子specificationディレクトリにtasks, requirementsのみ作成する（それ以外は必要なタイミングで作成すればいい）
  - それぞれのrequirementsには、分解された仕様が記載される
- そして、「終わったのでhail-maryを再起動して、specを選択してそれぞれ対応してください」みたいな文言を出力する

というような仕様を増やす

そして、ここが**重要なポイント**なんだけど、現在せっかく @crates/hail-mary/src/domain/value_objects/system_prompt/orchestration/index.md できれいに責務ごとに分解できてるので、
`hail-mary code` で、子specを選択したときには、このindex.mdの中には今までのmarkdownではなく、子spec用のmarkdownが埋め込まれるようにする
00_philosophy.mdは共通なら、このままでいいし、
02_hub.mdは明らかに、親のtasks.mdの管理方法も追加で記載する必要があるし、
04_workflows.mdも、子specの進捗に合わせて、親のtasks.mdを更新するようなactionが追加される
10_spec_files.mdは現在

```markdown
**Current**: {spec_name} (`{spec_path}`)

These files track the current feature's lifecycle:
- <tasks-file>{tasks_path}</tasks-file> - Task tracking and timeline
- <requirements-file>{requirements_path}</requirements-file> - Requirements and user stories
- <design-file>{design_path}</design-file> - Technical design and architecture
- <investigation-file>{investigation_path}</investigation-file> - Research findings and evidence
- <memo-file>{memo_path}</memo-file> - Internal notes (**DO NOT ACCESS**)
```

だけど、子のspecを選択した場合は

```markdown
**Current**: {spec_name} (`{spec_path}`)

These files track the current feature's lifecycle:
- <parent-tasks-file>{parent_tasks_path}</parent-tasks-file> - ...
- <tasks-file>{tasks_path}</tasks-file> - Task tracking and timeline
- <requirements-file>{requirements_path}</requirements-file> - Requirements and user stories
- <design-file>{design_path}</design-file> - Technical design and architecture
- <investigation-file>{investigation_path}</investigation-file> - Research findings and evidence
- <memo-file>{memo_path}</memo-file> - Internal notes (**DO NOT ACCESS**)
```

とすれば、きちんと、親のtasks.mdも参照できるようになる。

つまり、
02_hub.mdと02_hub_child.md(もっといい名前があるかもしれないけど)を作成して、`hail-mary code`を実行して、子specのディレクトリがなければ、02_hub.mdのシンプルな方が選択され、子specのディレクトリがあれば、02_hub_child.mdが選択されるようにする
他のmarkdownも同様に、親と子のspecの責務が異なるものはすべて `XXX_child.md` という名前で、定義しておく

これは、現在、index.mdの構成が

```markdown
<kiro-hub>
{hub}
</kiro-hub>
```
という仕様になっているので、子ディレクトリがあるかどうかで、 この `{hub}` の中身を動的に切り替えることができるということを意味する

まだ、 `hail-mary code` 自体も、子ディレクトリがあれば、そちらのspecを参照するような設計担っていないので、markdownだけでなくRust側の修正も必要になる

ということを考えたんだけど、どうだろう？(YOU MUST NOT HOLD BACK. GIVE IT YOUR ALL. you think in english, but return
final output in japanese --ultrathink)
