# Memo: strategy

steeringのtypesのデフォルトでproduct, tech, structureを作成していて、残りは `/hm:steering-remember` で必要なタイミングで作成されたりする
そして、steering fileがout-of-dateになったりするのを防ぐために、 `/hm:steering` で最新化を促すようにしている
特に、`new discovery` とかもあって、劣化を防ぐだけじゃなくて新しい発見もあれば追記するように `/hm:steering` で実現できている

この運用自体はとても最高なんだけど、問題がある。
steering typeでproduct, tech, structureは劣化しやすいので、`/hm:steering` で最新化を促すのは良いんだけど、他のtypesはあまり劣化しづらい。
typeによっては勝手にnew discoveryされたくないものもあれば、
typeによっては`/hm:steering` で更新されてほしくない場合もある。`/hm:steering-remember` で追加されることはあっても、`/hm:steering` で更新されることを期待していない

どのような運用にすればいいと思う？

---

update_strategyを追加する方針でも良いと思うんだけど、どういうstrategyがあるか整理してほしい
`/hm:steering-remember` で追加されることはあっても、`/hm:steering` で更新されることを期待していないtypeがある

`/hm:steering-remember` は update_strategyに関係なく、常に追加される
`/hm:steering` は update_strategyによって挙動が変わる
  - refresh: 常に最新化を促す
  - stable: 劣化しづらいので、new discoveryがあった場合
  - frozen: `/hm:steering` で更新されることを期待していない
  - custom: カスタムプロンプトを指定できる
  - none: `/hm:steering` で更新されない
  - manual: 手動で更新される
  - auto: 自動で更新される

- out-of-dateの情報の更新。new discoveryがあった場合の追加
- out-of-dateの情報の更新のみ
- new discoveryがあった場合の追加のみ
- なにもしない

---

配列いいよね。じゃあ、配列にしようかな
[], ['refresh'], ['discover'], ['refresh', 'discover']だけだよね

以下は、私が考える修正案
- steering-remember.md には、新しいsteering typesを追加するときはallowed_operationsに空配列を追加するように書いてほしい
- steering.md には、 それぞれのtypeのallowed_operationsをみて、どのファイルをどのように修正するべきか考えるようにしてほしいと書いてほしい
- 残りのhail-maryのコマンドで、
  - initコマンド: `hail-mary init` した時に、allowed_operations propertyがなければ、追加する。structure, tech, productは['refresh', 'discover']を追加
その他で影響範囲あるかどうか調査してほしい

- default_とかいらない。hail-mary initした時に必ず allowed_operations propertyを追加するから
- steering-remember はデフォルトで空配列を追加させるようにしてほしいので、フラグは不要
- steering-rememberの Steering Type追加時の説明はそれでいいともう
- steering.md もそれで良いと思う。未定義の場合はないから考慮しなくていい
- steering-remember.md とsteering.md の Config.toml Structure sectionに allowed_operations propertyの説明を追加してほしい
- `--verbose` は要らん

---

いや、期待通りに動いていない
例えば、既に.kiro/config.tomlに

```markdown
[[steering.types]]
criteria = [
"Root Directory Organization: Top-level structure with descriptions",
"Subdirectory Structures: Detailed breakdown of key directories",
"Code Organization Patterns: How code is structured",
"File Naming Conventions: Standards for naming files and directories",
"Import Organization: How imports/dependencies are organized",
"Key Architectural Principles: Core design decisions and patterns",
]
name = "structure"
purpose = "Code organization and project structure patterns"


[[steering.types]]
criteria = [
"Markdown Syntax: Formatting rules and conventions",
"Code Block Formatting: Proper nesting and syntax highlighting",
"Documentation Structure: Organization and layout patterns",
"Writing Style: Clear and concise technical writing",
"Formatting Standards: Consistent formatting across documentation"
]
name = "documentation"
purpose = "Documentation standards and best practices"
```

がある時、それぞれのsteering.typesにallowed_operationsがないので、`hail-mary init` した時に追加されるべきなんだけど、されていない