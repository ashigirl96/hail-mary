# Memo: talk

現在の、`/spec:requirements` と `/spec:design` は、実行したら、確認することなくに、`requirements.md` と `design.md` に書き込んでしまう。
しかし、必ずしも正しい仕様や設計にならない場合もあるので、あらかじめユーザーにどのような仕様にするか、設計にするかを表示して、確認してから書き込むようにしたい。
正確には、requirementsとdesignのslash commandにそういうオプションをつけたら、書き込む前に、ユーザーに確認するようにしたい

gatesにフラグの定義をして、そこで管理するべき？
slash commandにオプションをつけるべき？
それとも他にもいい案がある？brainstormしてほしい