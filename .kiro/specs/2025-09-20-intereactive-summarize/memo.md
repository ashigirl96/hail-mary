# Memo: intereactive-summarize


slash command investigateの課題
- 質問→調査→まとめる→…の、まとまったものに対して批評しても大きくドキュメントが変わることができない
- 毎回保存されてしまうため、質問→調査→質問→調査→…ほどのリズミカルな会話ができない。

objective
- ある程度interactiveに議論して、確度高いとユーザーが納得したものを保存できるようにしたい
- `/hm:summarize start` を開始してから、常に「（最初の質問→次の質問→次の次の質問）保存する？」って聞かれるようにする

まだ決めてないこと

保存する形式を
- ユーザーに出力した時系列の情報を出力する(user question → claude code answer → question → claude code answer → ... が保存される)
- 構造化されて質問と回答が構造化されて保存されるようにする(上記のuser question → claude code answerのやり取りを１つにまとめたレポートにする)

`/hm:summarize start` と `/hm:summarize record` 的な感じにするか？