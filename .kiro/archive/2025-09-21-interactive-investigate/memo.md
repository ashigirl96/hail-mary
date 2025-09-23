# Memo: interactive-investigate

新たなslash command `/hm:interactive-investigate` を追加したい。

Behavioral Flow:
1. `/hm:interactive-investigate start

   [調査依頼内容]
   ` で、新しいtopicとして、調査を開始する
2. 通常の調査依頼通りに、調査をしたら、結果を出す。 
   ```
   [調査結果]
   
   ---
   [Y: Save, N: Discard, A: 追加質問+調査]
   ```
 3. `Y`を押すと、結果を<kiro_investigation>に保存して、topicを閉じる
    -　必ず、`/hm:kiro_investigation` を実行した時に、した質問から、回答までで、ユーザーに表示してた内容を保存する(なぜなら、ユーザーはresponseで返してた表示結果しか見ていないから。なので、保存された内容が表示してたときの内容よりも少ないと、ユーザーは混乱する)
    - 可能な限り、どのような調査をして、どういう勘違いをして、ユーザーにどういうFeedbackをもらって、その後にどういう調査をして、最終的にどういう結論に至ったかを、時系列でわかるように保存する
 4. `N`を押すと、保存せずに調査を終わる
 5. `A`を押すと、追加質問を受け付けて、再度調査を行う `A, [追加質問内容]` → 2に戻る

Reference:
slash command: @reference/slash-commands.sh
