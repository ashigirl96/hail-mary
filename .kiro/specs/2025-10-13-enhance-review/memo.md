# Memo: enhance-review

`--review` の挙動を改善したい
- `/spec:design --review` を実行した時に、 <requirements-file>と<investigation-file>を読んだ上で、<investigation-file>の情報だけで、<requirements-file>を満たせるかの設計の方針と、どういう情報が足りないかを表示したい
- `/spec:requirements --review` を実行すると、`/spec:requirements` の時と同様に、得た要件に対して、先に調査をしてから、整理した要件を表示されたい。それを保存するか改善するかという流れにしたい
- `/spec:investigate --review` は要らない