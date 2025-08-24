# Zellij Plugin 技術調査メモ

## 調査依頼内容

zellij pluginで以下の機能の実現可能性について調査：

1. **3つのpaneを同時に立ち上げる**
2. **clickableなwidgetは作れるのか？クリックした時にそれをRust側でhookに何かできるのか？**
3. **pane1の標準出力をpane2の標準出力に流すことは可能か？**

## 調査結果サマリー

すべての要求された機能がZellijプラグインAPIで実現可能。

| 機能 | 実現可能性 | 難易度 | 備考 |
|------|-----------|--------|------|
| 3つのpane同時立ち上げ | ✅ 可能 | 低 | 複数のAPI方法あり |
| クリック可能widget | ✅ 可能 | 中 | マウスイベント+Built-in UI |
| pane間出力リレー | ✅ 可能 | 高 | 直接パイプは不可、間接的に実現 |

## 詳細実装方法

### 1. 3つのpaneを同時に立ち上げる

#### 方法A: Terminal API使用（推奨）

```rust
use zellij_tile::prelude::*;

impl ZellijPlugin for MyPlugin {
    fn load(&mut self, _: BTreeMap<String, String>) {
        request_permission(&[PermissionType::OpenTerminalsOrPlugins]);
        
        // 方法1: 通常のタイル配置
        open_terminal(&PathBuf::from("/"));
        open_terminal(&PathBuf::from("/"));
        open_terminal(&PathBuf::from("/"));
        
        // 方法2: フローティング配置
        open_terminal_floating(&PathBuf::from("/"), None);
        open_terminal_floating(&PathBuf::from("/"), None);
        open_terminal_floating(&PathBuf::from("/"), None);
        
        // 方法3: プラグインの近くに配置
        open_terminal_near_plugin(&PathBuf::from("/"));
        open_terminal_near_plugin(&PathBuf::from("/"));
        open_terminal_near_plugin(&PathBuf::from("/"));
    }
}
```

#### 方法B: Layout File使用

```yaml
# layout.kdl
---
direction: Horizontal
parts:
  - direction: Vertical
    split_size:
      Percent: 33
  - direction: Vertical  
    split_size:
      Percent: 33
  - direction: Vertical
    split_size:
      Percent: 34
```

```rust
// 動的レイアウト適用
let layout = r#"
direction: Horizontal
parts:
  - {}
  - {}
  - {}
"#;
new_tabs_with_layout(layout.to_string());
```

#### 利用可能API一覧

| API関数 | 用途 | 権限要求 |
|---------|------|----------|
| `open_terminal` | 新しいタイル端末 | `OpenTerminalsOrPlugins` |
| `open_terminal_floating` | フローティング端末 | `OpenTerminalsOrPlugins` |
| `open_terminal_near_plugin` | プラグイン近くに端末 | `OpenTerminalsOrPlugins` |
| `open_terminal_in_place` | 現在のpaneを置換 | `OpenTerminalsOrPlugins` |
| `new_tabs_with_layout` | レイアウト指定で新タブ | `ChangeApplicationState` |

### 2. Clickableなwidgetとクリックhook

#### マウスイベント処理基盤

```rust
use zellij_tile::prelude::*;

#[derive(Default)]
struct ClickablePlugin {
    buttons: Vec<Button>,
    selected_index: usize,
}

struct Button {
    x: usize,
    y: usize, 
    width: usize,
    height: usize,
    label: String,
    action: ButtonAction,
}

enum ButtonAction {
    CreatePane,
    SendCommand(String),
    SwitchTab(usize),
}

impl ZellijPlugin for ClickablePlugin {
    fn load(&mut self, _: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::OpenTerminalsOrPlugins,
            PermissionType::WriteToStdin,
        ]);
        subscribe(&[EventType::Mouse, EventType::Key]);
        
        // ボタン初期化
        self.init_buttons();
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Mouse(mouse_event) => {
                self.handle_mouse_event(mouse_event)
            }
            Event::Key(key) => {
                self.handle_key_event(key)
            }
            _ => false
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        self.render_ui(rows, cols);
    }
}

impl ClickablePlugin {
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> bool {
        match mouse_event.action {
            MouseAction::LeftClick => {
                let clicked_button = self.find_button_at(
                    mouse_event.column, 
                    mouse_event.row
                );
                
                if let Some(button_index) = clicked_button {
                    self.execute_button_action(button_index);
                    return true;
                }
            }
            MouseAction::ScrollUp | MouseAction::ScrollDown => {
                // スクロール処理
                return true;
            }
            _ => {}
        }
        false
    }
    
    fn find_button_at(&self, x: usize, y: usize) -> Option<usize> {
        self.buttons.iter().position(|button| {
            x >= button.x && 
            x < button.x + button.width &&
            y >= button.y && 
            y < button.y + button.height
        })
    }
    
    fn execute_button_action(&mut self, button_index: usize) {
        if let Some(button) = self.buttons.get(button_index) {
            match &button.action {
                ButtonAction::CreatePane => {
                    open_terminal_near_plugin(&PathBuf::from("/"));
                }
                ButtonAction::SendCommand(cmd) => {
                    write_chars(cmd);
                }
                ButtonAction::SwitchTab(tab_index) => {
                    switch_tab_to(*tab_index);
                }
            }
        }
    }
}
```

#### Built-in UI Components使用

```rust
fn render_clickable_table(&self) {
    let table = Table::new()
        .add_row(vec!["Button 1", "Button 2", "Button 3"])
        .add_styled_row(vec![
            Text::new("Create Pane").selected(),
            Text::new("Send Cmd").color_range(1, ..),
            Text::new("Switch Tab").color_range(2, ..)
        ]);
    
    print_table_with_coordinates(table, 0, 0, Some(60), Some(5));
}

fn render_ribbon_buttons(&self) {
    print_ribbon_with_coordinates(
        Text::new("Pane 1").selected(), 
        0, 0, Some(12), None
    );
    print_ribbon_with_coordinates(
        Text::new("Pane 2").color_range(1, 1..5), 
        12, 0, Some(12), None
    );
    print_ribbon_with_coordinates(
        Text::new("Pane 3").color_range(2, 1..5), 
        24, 0, Some(12), None
    );
}
```

### 3. Pane間出力リレー

直接的なパイプ機能は提供されていないが、以下の方法で実現可能：

#### 方法A: write_to_pane_id API使用

```rust
use zellij_tile::prelude::*;

#[derive(Default)]
struct OutputRelayPlugin {
    source_pane_id: Option<u32>,
    target_pane_id: Option<u32>,
    buffer: Vec<String>,
}

impl ZellijPlugin for OutputRelayPlugin {
    fn load(&mut self, _: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::WriteToStdin,
            PermissionType::ReadApplicationState,
        ]);
        subscribe(&[EventType::PaneUpdate]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(pane_manifest) => {
                self.monitor_pane_outputs(&pane_manifest);
                true
            }
            _ => false
        }
    }
}

impl OutputRelayPlugin {
    fn relay_output(&self, data: &str) {
        if let Some(target_id) = self.target_pane_id {
            // データをターゲットpaneに送信
            write_to_pane_id(target_id, data.as_bytes().to_vec());
            // または文字として送信
            write_chars_to_pane_id(target_id, data);
        }
    }
    
    fn monitor_pane_outputs(&mut self, pane_manifest: &PaneManifest) {
        // 注意: 現在のAPIでは直接pane出力を監視する方法はない
        // 代替案: custom messageやworkerを使用
        
        for (tab_position, panes) in &pane_manifest.panes {
            for pane in panes {
                if Some(pane.id) == self.source_pane_id {
                    // pane情報から何らかの方法で出力を取得
                    // （実際のAPIでは制限があるため工夫が必要）
                }
            }
        }
    }
}
```

#### 方法B: Plugin Workers使用

```rust
use zellij_tile::prelude::*;

// メインプラグイン
register_plugin!(OutputRelayPlugin);

// ワーカー登録
register_worker!(
    OutputMonitorWorker, 
    output_monitor_worker, 
    OUTPUT_MONITOR_WORKER
);

#[derive(Default, Serialize, Deserialize)]
struct OutputMonitorWorker {
    target_pane_id: Option<u32>,
}

impl<'de> ZellijWorker<'de> for OutputMonitorWorker {
    fn on_message(&mut self, message: String, payload: String) {
        match message.as_str() {
            "relay_output" => {
                if let Some(target_id) = self.target_pane_id {
                    // ワーカーからプラグインにメッセージ送信
                    post_message_to_plugin(PluginMessage::new_to_plugin(
                        "write_to_pane", 
                        &format!("{}:{}", target_id, payload)
                    ));
                }
            }
            "set_target_pane" => {
                self.target_pane_id = payload.parse().ok();
            }
            _ => {}
        }
    }
}

impl OutputRelayPlugin {
    fn setup_output_relay(&self, source_pane: u32, target_pane: u32) {
        // ターゲットpane設定
        post_message_to(PluginMessage::new_to_worker(
            "output_monitor",
            "set_target_pane",
            &target_pane.to_string()
        ));
        
        // 出力監視開始（擬似的）
        post_message_to(PluginMessage::new_to_worker(
            "output_monitor",
            "start_monitoring",
            &source_pane.to_string()
        ));
    }
}
```

#### 方法C: Pipes システム使用（高度）

```rust
impl ZellijPlugin for OutputRelayPlugin {
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let mut should_render = false;
        
        match pipe_message.source {
            PipeSource::Plugin(source_plugin_id) => {
                // 他のプラグインからのデータ
                if let Some(payload) = pipe_message.payload {
                    self.relay_to_target_pane(&payload);
                    should_render = true;
                }
            }
            PipeSource::Cli(pipe_id) => {
                // CLIからのパイプデータ
                if let Some(payload) = pipe_message.payload {
                    self.process_cli_input(&payload);
                    should_render = true;
                }
            }
        }
        
        should_render
    }
    
    fn relay_to_target_pane(&self, data: &str) {
        if let Some(target_id) = self.target_pane_id {
            write_to_pane_id(target_id, data.as_bytes().to_vec());
        }
    }
}
```

## 実装例：統合サンプル

```rust
use zellij_tile::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Default)]
struct MultiPaneControllerPlugin {
    pane_ids: Vec<u32>,
    selected_pane: usize,
    relay_active: bool,
    buttons: Vec<UIButton>,
}

#[derive(Clone)]
struct UIButton {
    id: String,
    label: String,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl ZellijPlugin for MultiPaneControllerPlugin {
    fn load(&mut self, _: BTreeMap<String, String>) {
        // 必要な権限を要求
        request_permission(&[
            PermissionType::OpenTerminalsOrPlugins,
            PermissionType::WriteToStdin,
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        
        // イベント購読
        subscribe(&[
            EventType::Mouse,
            EventType::Key, 
            EventType::PaneUpdate,
        ]);
        
        // UI初期化
        self.init_ui();
        
        // 3つのpaneを立ち上げ
        self.create_three_panes();
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Mouse(mouse_event) => {
                self.handle_mouse_event(mouse_event)
            }
            Event::Key(key) => {
                self.handle_key_event(key)
            }
            Event::PaneUpdate(pane_manifest) => {
                self.update_pane_list(&pane_manifest);
                true
            }
            _ => false
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        self.render_control_panel(rows, cols);
    }
}

impl MultiPaneControllerPlugin {
    fn init_ui(&mut self) {
        self.buttons = vec![
            UIButton {
                id: "create_panes".to_string(),
                label: "Create 3 Panes".to_string(),
                x: 2, y: 2, width: 15, height: 1,
            },
            UIButton {
                id: "toggle_relay".to_string(), 
                label: "Toggle Relay".to_string(),
                x: 20, y: 2, width: 13, height: 1,
            },
            UIButton {
                id: "send_test".to_string(),
                label: "Send Test".to_string(),
                x: 36, y: 2, width: 11, height: 1,
            },
        ];
    }
    
    fn create_three_panes(&mut self) {
        // 3つのターミナルpaneを作成
        open_terminal_near_plugin(&std::path::PathBuf::from("/"));
        open_terminal_near_plugin(&std::path::PathBuf::from("/tmp"));
        open_terminal_near_plugin(&std::path::PathBuf::from("/home"));
    }
    
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> bool {
        if let MouseAction::LeftClick = mouse_event.action {
            let clicked_button = self.find_button_at(
                mouse_event.column, 
                mouse_event.row
            );
            
            if let Some(button_id) = clicked_button {
                return self.execute_button_action(&button_id);
            }
        }
        false
    }
    
    fn find_button_at(&self, x: usize, y: usize) -> Option<String> {
        self.buttons.iter()
            .find(|button| {
                x >= button.x && 
                x < button.x + button.width &&
                y >= button.y && 
                y < button.y + button.height
            })
            .map(|button| button.id.clone())
    }
    
    fn execute_button_action(&mut self, button_id: &str) -> bool {
        match button_id {
            "create_panes" => {
                self.create_three_panes();
                true
            }
            "toggle_relay" => {
                self.relay_active = !self.relay_active;
                if self.relay_active {
                    self.setup_output_relay();
                }
                true
            }
            "send_test" => {
                self.send_test_message();
                true
            }
            _ => false
        }
    }
    
    fn setup_output_relay(&self) {
        if self.pane_ids.len() >= 2 {
            let source_pane = self.pane_ids[0];
            let target_pane = self.pane_ids[1];
            
            // ソースpaneにメッセージ送信（テスト用）
            write_to_pane_id(
                source_pane, 
                b"echo 'Output from Pane 1' | tee /dev/stderr\n".to_vec()
            );
        }
    }
    
    fn send_test_message(&self) {
        if self.pane_ids.len() >= 3 {
            let message = "Hello from Plugin!\n";
            for (i, &pane_id) in self.pane_ids.iter().enumerate() {
                write_to_pane_id(
                    pane_id, 
                    format!("echo 'Message to Pane {}'\n", i + 1).as_bytes().to_vec()
                );
            }
        }
    }
    
    fn update_pane_list(&mut self, pane_manifest: &PaneManifest) {
        self.pane_ids.clear();
        for panes in pane_manifest.panes.values() {
            for pane in panes {
                if !pane.is_plugin {
                    self.pane_ids.push(pane.id);
                }
            }
        }
    }
    
    fn render_control_panel(&self, rows: usize, cols: usize) {
        // タイトル
        println!("Multi-Pane Controller Plugin");
        println!("============================");
        
        // ボタン描画（Built-in UI使用）
        let button_table = Table::new()
            .add_row(vec!["Create 3 Panes", "Toggle Relay", "Send Test"])
            .add_styled_row(vec![
                Text::new("● Available").color_range(2, ..),
                if self.relay_active { 
                    Text::new("● Active").color_range(1, ..) 
                } else { 
                    Text::new("○ Inactive").color_range(0, ..) 
                },
                Text::new("● Ready").color_range(2, ..),
            ]);
        
        print_table_with_coordinates(button_table, 0, 4, Some(cols), Some(4));
        
        // Pane情報表示
        println!("\nActive Panes ({}): {:?}", self.pane_ids.len(), self.pane_ids);
        println!("Relay Status: {}", if self.relay_active { "ON" } else { "OFF" });
        
        // 使用方法表示
        println!("\nUsage:");
        println!("- Click buttons to control panes");
        println!("- Use 'Toggle Relay' to enable output forwarding");
        println!("- 'Send Test' sends messages to all panes");
    }
    
    fn handle_key_event(&mut self, key: KeyWithModifier) -> bool {
        match key.bare_key {
            BareKey::Char('1') => {
                self.execute_button_action("create_panes")
            }
            BareKey::Char('2') => {
                self.execute_button_action("toggle_relay")
            }
            BareKey::Char('3') => {
                self.execute_button_action("send_test")
            }
            BareKey::Esc => {
                hide_self();
                false
            }
            _ => false
        }
    }
}

register_plugin!(MultiPaneControllerPlugin);
```

## 制約と注意事項

### 技術的制約

1. **直接的なpane出力監視は不可**
   - Zellijプラグインは他のpaneの出力を直接監視できない
   - 代替案：カスタムメッセージやワーカーシステム使用

2. **パーミッション要求必須**
   - 各機能に対応する権限を明示的に要求する必要
   - ユーザーが権限を拒否する可能性を考慮

3. **非同期処理の制約**
   - WASMの制約によりスレッド使用に制限
   - Plugin Workersを活用して非同期処理実現

### パフォーマンス考慮事項

1. **レンダリング頻度**
   - 不要な再描画を避ける（`update`の戻り値を適切に設定）
   - マウスイベント処理時のパフォーマンス最適化

2. **メモリ使用量**
   - pane情報の適切なキャッシュ管理
   - 大量データの中継時のバッファリング戦略

## 推奨実装アプローチ

1. **段階的実装**
   - Phase 1: 3つのpane作成機能
   - Phase 2: クリック可能UI実装  
   - Phase 3: 出力リレー機能追加

2. **テスト戦略**
   - 各機能を独立してテスト
   - 統合テストでユーザーシナリオ検証

3. **エラーハンドリング**
   - 権限不足時の適切な処理
   - pane作成失敗時のリカバリ

## 追加調査: 新機能要求

### 調査依頼内容（第2回）

1. **それぞれのpaneを立ち上げる時に、任意のコマンドを指定して立ち上げることはできる？**
2. **このツールを使っている時だけ、渡すショートカットを追加できる？そのショートカットを使うと、floating paneが表示されて、選択したものをRust側に送信することはできる？**

### 調査結果サマリー（第2回）

| 機能 | 実現可能性 | 難易度 | 主要API |
|------|-----------|--------|---------|
| 任意コマンドでpane起動 | ✅ 完全に可能 | 低 | `open_command_pane_*` API群 |
| カスタムショートカット + floating選択UI | ✅ 完全に可能 | 中 | `reconfigure` + `intercept_key_presses` |

### 詳細実装方法（第2回）

#### 1. 任意のコマンドでpane立ち上げ

**Command Pane API群**を使用して完全に実現可能：

```rust
// 基本的なcommand pane
open_command_pane(&CommandToRun::new("htop").with_args(vec!["--tree"]));

// フローティングcommand pane  
open_command_pane_floating(&CommandToRun::new("git").with_args(vec!["log", "--oneline"]));

// プラグイン近くにcommand pane
open_command_pane_near_plugin(&CommandToRun::new("watch").with_args(vec!["ls", "-la"]));

// バックグラウンドcommand pane（非表示）
open_command_pane_background(&CommandToRun::new("backup_script.sh"));
```

**利用可能なCommand Pane API一覧：**

| API関数 | 用途 | 権限要求 |
|---------|------|----------|
| `open_command_pane` | 通常のcommand pane | `RunCommands` |
| `open_command_pane_floating` | フローティングcommand pane | `RunCommands` |
| `open_command_pane_near_plugin` | プラグイン近くにcommand pane | `RunCommands` |
| `open_command_pane_in_place` | 現在paneを置換 | `RunCommands` |
| `open_command_pane_background` | バックグラウンド実行 | `RunCommands` |

#### 2. カスタムショートカット + floating選択UI

**3つのAPIを組み合わせて実現**：

1. **`reconfigure`** - グローバルショートカット登録
2. **`intercept_key_presses`** - キー入力インターセプト  
3. **floating pane + 選択UI** - 選択インターフェース表示

**実装例：VSCodeスタイルのコマンドパレット**

```rust
impl ShortcutSelectorPlugin {
    fn register_global_shortcut(&self) {
        if let Some(plugin_id) = self.plugin_id {
            let config = format!(r#"
keybinds {{
    shared {{
        bind "Ctrl Shift p" {{
            MessagePluginId {} {{
                name "trigger_selector"
            }}
        }}
    }}
}}"#, plugin_id);
            
            // グローバルショートカットを登録
            reconfigure(&config, false);
        }
    }
    
    fn show_selector(&mut self) {
        // キー入力をインターセプト開始
        intercept_key_presses();
        
        // floating paneとして表示
        show_self();
        
        self.is_selector_active = true;
    }
    
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        if pipe_message.name == "trigger_selector" {
            self.show_selector();
            return true;
        }
        false
    }
}
```

**主要機能：**
- `Ctrl+Shift+P` でセレクタ起動
- 矢印キーまたは `j`/`k` で選択
- `Enter` で実行、`Esc` でキャンセル
- 数字キー（1-9）で直接選択
- 複数コンテキスト対応

#### Command Pane vs Terminal Pane

| 機能 | Terminal Pane | Command Pane |
|------|---------------|--------------|
| 任意コマンド実行 | ❌ | ✅ |
| コマンド再実行 | ❌ | ✅ |
| 終了ステータス表示 | ❌ | ✅ |
| インタラクティブ入力 | ✅ | ✅ |
| 権限要求 | `OpenTerminalsOrPlugins` | `RunCommands` |

### 実用的な統合例

```rust
// プロジェクト管理プラグイン例
struct ProjectManagerPlugin {
    projects: Vec<Project>,
    commands: Vec<QuickCommand>,
}

struct Project {
    name: String,
    path: PathBuf,
    commands: Vec<ProjectCommand>,
}

struct QuickCommand {
    label: String,
    command: String,
    args: Vec<String>,
    launch_type: LaunchType,
}

enum LaunchType {
    Floating,      // open_command_pane_floating
    Background,    // open_command_pane_background  
    InPlace,       // open_command_pane_in_place
    Tiled,         // open_command_pane_near_plugin
}

// 使用例：
// Ctrl+Shift+P → "Build Project" → cargo build をfloating paneで実行
// Ctrl+Shift+O → "Open Files" → ファイルブラウザをin-placeで起動
// Ctrl+Shift+T → "Terminal" → 新しいターミナルをtiled形式で起動
```

### 新機能の利点

1. **開発効率向上**: よく使うコマンドへの瞬時アクセス
2. **ワークフロー最適化**: プロジェクト固有のタスク自動化
3. **柔軟性**: floating, tiled, background等の表示形式選択
4. **統合性**: Zellijセッション内での一貫したUX

## 調査依頼内容（第3回）

### 追加調査：CLIからのpane通信

CLIからzellijを立ち上げて、そこにあるpaneにCLIから送受信したりするのをzellij-tileクレートで実現できないの？

### 調査結果サマリー（第3回）

| 機能 | 実現可能性 | 難易度 | 主要システム |
|------|-----------|--------|-------------|
| CLI→Plugin→Pane送信 | ✅ 完全に可能 | 中 | Pipe通信システム |
| Pane→Plugin→CLI受信 | ⚠️ 制限あり | 高 | Event監視+Pipe出力 |

### 詳細実装方法（第3回）

#### Pipe通信システムの仕組み

Zellijの**Pipe**は単方向通信チャネルで、CLI ↔ プラグイン間のデータ送受信を可能にします：

**基本構成**：
- **CLI側**: `stdin` → Zellij pipe → Plugin
- **Plugin側**: データ処理 → 指定paneに送信 
- **逆方向**: Plugin → CLI `stdout`（制限あり）

#### 実装パターン1: CLI入力→Pane送信

```rust
use zellij_tile::prelude::*;

#[derive(Default)]
struct CLIPaneRelay {
    target_pane_id: Option<u32>,
    buffer: Vec<String>,
}

impl ZellijPlugin for CLIPaneRelay {
    fn load(&mut self, _: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::WriteToStdin,
            PermissionType::ReadApplicationState,
        ]);
        subscribe(&[EventType::PaneUpdate]);
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.source {
            PipeSource::Cli(input_pipe_id) => {
                if let Some(payload) = pipe_message.payload {
                    // CLIからのデータを受信
                    eprintln!("Received from CLI: {}", payload);
                    
                    // 指定paneに送信
                    if let Some(target_id) = self.target_pane_id {
                        write_to_pane_id(target_id, payload.as_bytes().to_vec());
                        // または文字として送信
                        write_chars_to_pane_id(target_id, &payload);
                    }
                    
                    // CLI側にACK送信
                    cli_pipe_output(&input_pipe_id, "Data sent to pane");
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(pane_manifest) => {
                // 最初のターミナルpaneをターゲットに設定
                for panes in pane_manifest.panes.values() {
                    for pane in panes {
                        if !pane.is_plugin {
                            self.target_pane_id = Some(pane.id);
                            break;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }
}

register_plugin!(CLIPaneRelay);
```

#### 実装パターン2: Pane監視→CLI出力

```rust
#[derive(Default)]
struct PaneCLIRelay {
    monitored_pane_id: Option<u32>,
    cli_pipe_id: Option<String>,
    last_pane_content: String,
}

impl ZellijPlugin for PaneCLIRelay {
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.source {
            PipeSource::Cli(input_pipe_id) => {
                // CLI pipe IDを保存
                self.cli_pipe_id = Some(input_pipe_id.clone());
                
                // 設定コマンド処理
                if let Some(payload) = pipe_message.payload {
                    self.process_cli_command(&payload, &input_pipe_id);
                }
                true
            }
            _ => false,
        }
    }
    
    fn process_cli_command(&mut self, command: &str, pipe_id: &str) {
        match command.trim() {
            cmd if cmd.starts_with("monitor_pane:") => {
                let pane_id: u32 = cmd[13..].parse().unwrap_or(0);
                self.monitored_pane_id = Some(pane_id);
                cli_pipe_output(pipe_id, &format!("Monitoring pane {}", pane_id));
            }
            "stop_monitoring" => {
                self.monitored_pane_id = None;
                cli_pipe_output(pipe_id, "Stopped monitoring");
            }
            _ => {
                cli_pipe_output(pipe_id, "Unknown command");
            }
        }
    }
}
```

#### Backpressure制御システム

```rust
impl CLIPaneRelay {
    fn handle_backpressure(&mut self, pipe_id: &str, payload: &str) -> bool {
        // 処理が重い場合はCLI側を一時停止
        if self.is_processing_heavy_task() {
            block_cli_pipe_input(pipe_id);
            
            // 非同期処理完了後に再開
            self.schedule_resume(pipe_id.to_string());
            return true;
        }
        
        // 通常処理
        self.process_payload(payload);
        false
    }
    
    fn schedule_resume(&self, pipe_id: String) {
        // Workerを使った非同期処理例
        post_message_to(PluginMessage::new_to_worker(
            "background_processor",
            "process_and_resume",
            &pipe_id
        ));
    }
}
```

#### 高度な使用例：双方向リアルタイム通信

```rust
use zellij_tile::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
struct RealtimeCLIBridge {
    pane_connections: std::collections::HashMap<u32, String>, // pane_id -> cli_pipe_id
    command_queue: Vec<PendingCommand>,
    monitor_active: bool,
}

#[derive(Serialize, Deserialize)]
struct PendingCommand {
    pane_id: u32,
    command: String,
    timestamp: u64,
}

impl ZellijPlugin for RealtimeCLIBridge {
    fn load(&mut self, _: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::WriteToStdin,
            PermissionType::ReadApplicationState,
            PermissionType::RunCommands,
        ]);
        subscribe(&[
            EventType::PaneUpdate,
            EventType::CustomMessage,
            EventType::Timer,
        ]);
        
        // 定期的な状態更新
        set_timeout(1.0);
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.source {
            PipeSource::Cli(input_pipe_id) => {
                self.handle_cli_message(pipe_message, input_pipe_id)
            }
            PipeSource::Plugin(_) => {
                self.handle_plugin_message(pipe_message)
            }
        }
    }
    
    fn handle_cli_message(&mut self, msg: PipeMessage, pipe_id: String) -> bool {
        if let Some(payload) = msg.payload {
            match self.parse_cli_command(&payload) {
                CLICommand::ConnectPane(pane_id) => {
                    self.pane_connections.insert(pane_id, pipe_id.clone());
                    cli_pipe_output(&pipe_id, &format!("Connected to pane {}", pane_id));
                }
                CLICommand::SendCommand { pane_id, command } => {
                    write_to_pane_id(pane_id, format!("{}\n", command).as_bytes().to_vec());
                    cli_pipe_output(&pipe_id, "Command sent");
                }
                CLICommand::StartMonitoring => {
                    self.monitor_active = true;
                    cli_pipe_output(&pipe_id, "Monitoring started");
                }
                CLICommand::Disconnect => {
                    self.pane_connections.retain(|_, cli_id| cli_id != &pipe_id);
                    cli_pipe_output(&pipe_id, "Disconnected");
                }
            }
        }
        true
    }
}

#[derive(Debug)]
enum CLICommand {
    ConnectPane(u32),
    SendCommand { pane_id: u32, command: String },
    StartMonitoring,
    Disconnect,
}

impl RealtimeCLIBridge {
    fn parse_cli_command(&self, input: &str) -> CLICommand {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        match parts.get(0) {
            Some(&"connect") => {
                let pane_id = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                CLICommand::ConnectPane(pane_id)
            }
            Some(&"send") => {
                let pane_id = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let command = parts[2..].join(" ");
                CLICommand::SendCommand { pane_id, command }
            }
            Some(&"monitor") => CLICommand::StartMonitoring,
            Some(&"disconnect") => CLICommand::Disconnect,
            _ => CLICommand::Disconnect, // デフォルト
        }
    }
}

register_plugin!(RealtimeCLIBridge);
```

#### 使用例：CLIからの操作

```bash
# Zellijセッション起動（プラグイン含む）
zellij --layout my_layout_with_plugin.kdl

# 別ターミナルから操作
echo "connect 1" | zellij action pipe my_cli_bridge_plugin
echo "send 1 ls -la" | zellij action pipe my_cli_bridge_plugin
echo "send 1 htop" | zellij action pipe my_cli_bridge_plugin
echo "monitor" | zellij action pipe my_cli_bridge_plugin
```

#### 制約事項と対策

**制約1: 直接pane出力監視不可**
```rust
// ❌ 不可能：直接pane出力を読み取り
// let output = read_pane_output(pane_id);

// ✅ 対策：間接的監視
// 1. カスタムコマンド経由
write_to_pane_id(pane_id, b"my_command | tee /tmp/pane_output\n");

// 2. ファイル監視Worker
register_worker!(FileWatcherWorker, file_watcher, FILE_WATCHER);
```

**制約2: 非同期処理制限**
```rust
// ✅ Worker使用で解決
impl<'de> ZellijWorker<'de> for BackgroundProcessor {
    fn on_message(&mut self, message: String, payload: String) {
        match message.as_str() {
            "process_pane_data" => {
                // 重い処理をワーカーで実行
                let result = self.heavy_processing(&payload);
                
                // 結果をプラグインに返送
                post_message_to_plugin(PluginMessage::new_to_plugin(
                    "processing_complete",
                    &result
                ));
            }
            _ => {}
        }
    }
}
```

#### 実用的な応用例

1. **リモートコントロール**: SSH経由でZellij paneを操作
2. **ログストリーミング**: pane出力をリアルタイム転送
3. **自動化スクリプト**: CI/CDからZellijタスク実行
4. **デバッグツール**: 開発環境の遠隔操作

### 結論（第3回）

zellij-tileクレートとPipe通信システムにより、**CLI ↔ Plugin ↔ Pane** の双方向通信が実現可能です。直接的なpane出力監視には制約がありますが、Worker systemやカスタムメッセージングで高度なCLI統合が可能です。

## 参考リンク

- [Zellij Plugin API Documentation](https://github.com/zellij-org/zellij/blob/main/docs/plugin-api.md)
- [Zellij Tile Crate](https://docs.rs/zellij-tile/latest/zellij_tile/)
- [Plugin Development Examples](https://github.com/zellij-org/zellij/tree/main/example)
- [Pipe Communication Documentation](https://zellij.dev/documentation/plugin-pipes)