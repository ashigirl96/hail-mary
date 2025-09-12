現在の @.claude/commands/hm/steering.md の課題に対して
<kiro_investigation_path/>で調査して、 <kiro_design_path/>で設計した。@.claude/commands/hm/steering.md を実装して、　一回 `/hm:steering` を使ってみた。

@.kiro/specs/2025-09-12-enhance-steering/log.md に `/hm:steering` を実行した時のログを記載してる

以下が私のレビューです

- steeringのtypesは4つあるのに、Taskが１つしか実行されていない。本当は
  Task(Investigate comprehensively according to product criteria)
  Task(Investigate comprehensively according to tech criteria)
  Task(Investigate comprehensively according to structure criteria)
  ...

的な感じになってほしい
---

reference:
- slash command: @reference/slash-commands.md
- prompt engineering steering: @.kiro/steering/prompt-engineering.md




----

- 逐次更新うざい
- 内部でultrathinkにしてほしい