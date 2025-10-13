# Memo: documentation-lang

`hail-mary code` で、.kiro配下のファイルが自動で作成されたり、新しく入力したspecの名前からスペックドキュメントが生成されるようになっているが

.kiro/config.tomlも生成されるが、ここに新たに「specドキュメントをどの言語で記載するか」を定義できるようにしたい
例えば、config.tomlに
```toml
[spec]
lang = "ja"

```
と書いておく(もっといい名前があればそれを採用)と、specドキュメントは日本語で生成されるようにしたい

そのためには、`hail-mary code`をした時に、生成されるtasks.mdにどの言語で記載するかというmeta情報が含まれるようにしなければならない
なので、tasks.mdを生成するときに、config.tomlに含まれるspec.langの情報を参照して、埋め込むようにしたい

現状

```markdown
# Tasks

## State Tracking

| Document | Status | Coverage | Next Action |
|----------|--------|----------|-------------|
| requirements.md | pending | - | Define requirements |
| investigation.md | pending | - | Start investigation after requirements |
| design.md | pending | - | Awaiting 100% coverage |
| tasks.md#Timeline | pending | 0% | Plan implementation order |

## Timeline

- [x] Spec created → documentation-lang
- [ ] Requirements definition
- [ ] Investigation topics identification
- [ ] Design documentation
- [ ] Implementation planning
```

を生成しているが、どこかに言語について記載されたい。

そして、 @crates/hail-mary/src/domain/value_objects/system_prompt/pattern_router/07_requirements.md とかに、「tasks.mdに記載されている言語で書き込むように」みたいなことがBoundariesにかいてあると夜朝王
investigation, designも同様に
