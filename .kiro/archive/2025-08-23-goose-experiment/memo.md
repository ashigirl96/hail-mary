現在、`new` サブコマンドで、新しいプロジェクトが追加されるが、`complete` サブコマンドが存在しない。 `done` サブコマンドを追加すると、TUIで起動して、現在ある、`.kiro/specs` にあるディレクトリ一覧と選択肢が表示される ratatuiで、複数の選択肢を選択(space key)し、enterキーで、実行するようなことできる？

## **実現可能性**

ratatuiには複数選択をサポートするウィジェットが組み込まれています。 rendering.rs:1163-1166 **`List`**ウィジェットと**`ListState`**を使用することで、スペースキーで複数選択し、エンターキーで実行するような機能を実装できます。

## **基本的な実装アプローチ**

### **1. TUIアプリケーションの初期化**

ratatuiの便利な初期化関数を使用してTUIを起動できます： init.rs:34-36

```
ratatui::run(|terminal| {
// TUIロジック
})
```

### **2. 複数選択リストの実装**

**`List`**ウィジェットを使用して選択可能なリストを作成します： table.rs:39-42 **`StatefulWidget`**として実装されているため、状態管理が可能です。

### **3. イベント処理**

crosstermを使用してキーボードイベントを処理します： lib.rs:230-244

```
match event::read()? {
    Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
        KeyCode::Char(' ') => {
// スペースキーで選択/選択解除
        },
        KeyCode::Enter => {
// エンターキーで実行
        },
// その他のキー処理
    }
}
```

## **`.kiro/specs`ディレクトリの読み込み**

ファイルシステムからディレクトリ一覧を取得し、**`List`**ウィジェットのアイテムとして表示できます。

## **Notes**

ratatuiは即座モードレンダリングを採用しており、 lib.rs:116-122 毎フレーム完全にUIを再描画しますが、変更された部分のみをターミナルに送信することで効率的に動作します。複数選択機能は**`ListState`**を拡張してカスタム状態管理を実装することで実現できます。

Wiki pages you might want to explore:

- [**Overview (ratatui/ratatui)**](https://deepwiki.com/ratatui/ratatui/1-overview)
- [**Core Concepts (ratatui/ratatui)**](https://deepwiki.com/ratatui/ratatui/2-core-concepts)

**Copy Response**Share

[ratatui/ratatui](https://github.com/ratatui/ratatui)

[ratatui-widgets/src/list/rendering.rs](https://github.com/ratatui/ratatui/blob/5fa342cc/ratatui-widgets/src/list/rendering.rs)

`1160        *state.offset_mut() = offset;
1161        state.select(selected);
1162`

`1163        let list = List::new(["Item 0", "Item 1", "Item 2", "Item 3", "Item 4", "Item 5"])
1164            .scroll_padding(padding)
1165            .highlight_symbol(">> ");
1166        StatefulWidget::render(list, buffer.area, &mut buffer, &mut state);`

`1167        assert_eq!(buffer, Buffer::with_lines(expected));
1168    }
1169`

[ratatui/ratatui](https://github.com/ratatui/ratatui)

[ratatui/src/init.rs](https://github.com/ratatui/ratatui/blob/5fa342cc/ratatui/src/init.rs)

`31//!
32//! The module provides several related functions that handle different initialization scenarios:
33//!`

`34//! - [`run`] - Initializes a terminal, runs a closure, and automatically restores the terminal
35//!   state. This is the simplest way to run a Ratatui application and handles all setup and cleanup
36//!   automatically.`

`37//! - [`init`] - Creates a terminal with reasonable defaults including alternate screen and raw
38//!   mode. Panics on failure.
39//! - [`try_init`] - Same as [`init`] but returns a `Result` instead of panicking.`

[ratatui/ratatui](https://github.com/ratatui/ratatui)

[ratatui-widgets/src/table.rs](https://github.com/ratatui/ratatui/blob/5fa342cc/ratatui-widgets/src/table.rs)

`36///
37/// [`Table`] implements [`Widget`] and so it can be drawn using `Frame::render_widget`.
38///`

`39/// [`Table`] is also a [`StatefulWidget`], which means you can use it with [`TableState`] to allow
40/// the user to scroll through the rows and select one of them. When rendering a [`Table`] with a
41/// [`TableState`], the selected row, column and cell will be highlighted. If the selected row is
42/// not visible (based on the offset), the table will be scrolled to make the selected row visible.`

`43///
44/// Note: if the `widths` field is empty, the table will be rendered with equal widths.
45/// Note: Highlight styles are applied in the following order: Row, Column, Cell.`

[ratatui/ratatui](https://github.com/ratatui/ratatui)

[ratatui/src/lib.rs](https://github.com/ratatui/ratatui/blob/5fa342cc/ratatui/src/lib.rs)

`113//! section of the [Ratatui Website] for more details on how to use other backends ([Termion] /
114//! [Termwiz]).
115//!`

`116//! Every application built with `ratatui` needs to implement the following steps:
117//!
118//! - Initialize the terminal (see the [`init` module] for convenient initialization functions)
119//! - A main loop that:
120//!   - Draws the UI
121//!   - Handles input events
122//! - Restore the terminal state`

`123//!
124//! ## Initialize and restore the terminal
125//!`

`227//! Website] for conceptual information. For example, if you are using [Crossterm], you can use the
228//! [`crossterm::event`] module to handle events.
229//!`

`230//! ```rust,no_run
231//! use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
232//!
233//! fn handle_events() -> std::io::Result<bool> {
234//!     match event::read()? {
235//!         Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
236//!             KeyCode::Char('q') => return Ok(true),
237//!             // handle other key events
238//!             _ => {}
239//!         },
240//!         // handle other events
241//!         _ => {}
242//!     }
243//!     Ok(false)
244//! }`

`245//! ```
246//!
247//! ## Layout`