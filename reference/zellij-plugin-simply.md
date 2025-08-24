# Zellij Plugin Development - Practical Guide

A simplified guide to building Zellij plugins, focusing on the most commonly used features and patterns.

## Quick Start

### Minimal Plugin Structure

```rust
use zellij_tile::prelude::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct State {
    // Your plugin state here
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _config: BTreeMap<String, String>) {
        // Request permissions and subscribe to events
        request_permission(&[
            PermissionType::ReadApplicationState,
        ]);
        subscribe(&[EventType::Key]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key) => {
                // Handle key presses
                true // return true to trigger render
            }
            _ => false
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        println!("Hello, Zellij!");
    }
}
```

## Core Concepts

### Plugin Lifecycle

1. **load()** - Called when plugin starts. Use for:
   - Requesting permissions
   - Subscribing to events
   - Initial setup

2. **update()** - Called when subscribed events occur. Return `true` to trigger render.

3. **render()** - Called to draw the plugin UI. Print to stdout.

### Permissions

Request permissions in `load()` method:

```rust
request_permission(&[
    PermissionType::ReadApplicationState,     // Read panes, tabs
    PermissionType::ChangeApplicationState,   // Modify panes, tabs
    PermissionType::OpenFiles,                // Open files in editor
    PermissionType::RunCommands,              // Run system commands
    PermissionType::WriteToStdin,             // Send input to terminals
]);
```

## Essential Events

Subscribe to events you need in `load()`:

```rust
subscribe(&[
    EventType::Key,           // User key presses
    EventType::TabUpdate,     // Tab changes
    EventType::PaneUpdate,    // Pane changes
]);
```

### Key Events

```rust
Event::Key(key) => {
    match key.bare_key {
        BareKey::Char('q') => hide_self(),
        BareKey::Enter => {
            // Do something
        }
        BareKey::Esc => hide_self(),
        _ => {}
    }
}
```

### Tab/Pane Updates

```rust
Event::TabUpdate(tab_info) => {
    // tab_info contains all tab information
    // Access with tab_info.iter()
}

Event::PaneUpdate(pane_manifest) => {
    // pane_manifest.panes contains panes by tab position
    // Access with pane_manifest.panes.get(&tab_position)
}
```

## Essential Commands

### File Operations

```rust
// Open file in editor
open_file(FileToOpen::new(path_buf), BTreeMap::new());

// Open file at specific line
open_file_with_line(FileToOpen::new(path_buf), line_number, BTreeMap::new());

// Open floating file editor
open_file_floating(FileToOpen::new(path_buf), None, BTreeMap::new());
```

### Terminal Operations

```rust
// Open new terminal
open_terminal(&path_buf);

// Open floating terminal
open_terminal_floating(&path_buf, None);

// Run command in new pane
open_command_pane(
    CommandToRun::new("ls").with_args(vec!["-la"]),
    BTreeMap::new()
);
```

### Navigation

```rust
// Focus specific pane
focus_terminal_pane(pane_id, true);

// Switch to tab by index
switch_tab_to(tab_index);

// Move between panes
focus_next_pane();
focus_previous_pane();
```

### Plugin Control

```rust
// Hide the plugin
hide_self();

// Show and focus the plugin
show_self();

// Close the plugin
close_self();
```

## Working Examples

### Simple Pane Switcher

```rust
use zellij_tile::prelude::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct PaneSwitcher {
    panes: Vec<PaneInfo>,
    selected: usize,
}

register_plugin!(PaneSwitcher);

impl ZellijPlugin for PaneSwitcher {
    fn load(&mut self, _config: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[EventType::Key, EventType::PaneUpdate]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PaneUpdate(pane_manifest) => {
                // Update pane list
                self.panes.clear();
                for panes in pane_manifest.panes.values() {
                    for pane in panes {
                        if !pane.is_plugin {
                            self.panes.push(pane.clone());
                        }
                    }
                }
                true
            }
            Event::Key(key) => {
                match key.bare_key {
                    BareKey::Down => {
                        if !self.panes.is_empty() {
                            self.selected = (self.selected + 1) % self.panes.len();
                        }
                        true
                    }
                    BareKey::Up => {
                        if !self.panes.is_empty() {
                            self.selected = if self.selected == 0 {
                                self.panes.len() - 1
                            } else {
                                self.selected - 1
                            };
                        }
                        true
                    }
                    BareKey::Enter => {
                        if let Some(pane) = self.panes.get(self.selected) {
                            focus_terminal_pane(pane.id, true);
                            hide_self();
                        }
                        false
                    }
                    BareKey::Esc => {
                        hide_self();
                        false
                    }
                    _ => false
                }
            }
            _ => false
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        for (i, pane) in self.panes.iter().enumerate() {
            let marker = if i == self.selected { ">" } else { " " };
            println!("{} {}", marker, pane.title);
        }
    }
}
```

### Key Pattern: State Management

```rust
#[derive(Default)]
struct State {
    // UI state
    selected_index: usize,
    items: Vec<String>,
    
    // Application state
    current_tab: Option<TabInfo>,
    current_panes: Vec<PaneInfo>,
    
    // Plugin state
    loading: bool,
    error_message: Option<String>,
}

impl State {
    fn handle_navigation(&mut self, key: BareKey) -> bool {
        match key {
            BareKey::Down | BareKey::Char('j') => {
                if !self.items.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.items.len();
                    true
                }
            }
            BareKey::Up | BareKey::Char('k') => {
                if !self.items.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.items.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                    true
                }
            }
            _ => false
        }
    }
}
```

## Plugin Workers (For Background Tasks)

When you need to do heavy work without blocking the UI:

```rust
use zellij_tile::prelude::*;
use serde::{Deserialize, Serialize};

// Define worker
#[derive(Default, Serialize, Deserialize)]
pub struct SearchWorker {
    // worker state
}

impl<'de> ZellijWorker<'de> for SearchWorker {
    fn on_message(&mut self, message: String, payload: String) {
        // Handle messages from main plugin
        // Do heavy work here
        
        // Send results back to plugin
        post_message_to_plugin(PluginMessage::new_to_plugin(
            "search_complete",
            &results_json,
        ));
    }
}

// Register worker
register_worker!(SearchWorker, search_worker, SEARCH_WORKER);

// In main plugin, send work to worker
post_message_to(PluginMessage::new_to_worker(
    "search",
    "start_search",
    &search_params,
));
```

## Development Workflow

### Project Setup

```bash
# Create new Rust project
cargo new my-zellij-plugin --lib
cd my-zellij-plugin

# Add to Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
zellij-tile = "0.40.0"
```

### Build and Test

```bash
# Build plugin
cargo build --target wasm32-wasi --release

# Test in Zellij
zellij action start-or-reload-plugin \
  file:target/wasm32-wasi/release/my_plugin.wasm
```

### Development Layout

Create a development environment with this layout:

```kdl
layout {
    pane edit="src/lib.rs"
    pane command="bash" {
        args "-c" "cargo build --target wasm32-wasi --release && zellij action start-or-reload-plugin file:target/wasm32-wasi/release/my_plugin.wasm"
    }
    pane {
        plugin location="file:target/wasm32-wasi/release/my_plugin.wasm"
    }
}
```

### Debugging

- Use `eprintln!()` for debug output - appears in Zellij log
- Log location: `/tmp/zellij-<UID>/zellij-log/zellij.log`
- Use `zellij action start-or-reload-plugin` for hot reloading

## Best Practices

1. **Always handle errors gracefully** - plugins shouldn't crash Zellij
2. **Request minimal permissions** - only what you actually need
3. **Use workers for heavy tasks** - keep UI responsive
4. **Cache expensive operations** - especially file system access
5. **Provide visual feedback** - loading states, error messages
6. **Make navigation intuitive** - follow common keybindings (j/k, arrows, Enter, Esc)

## Common Patterns

### Modal Dialog

```rust
fn render_modal(&self, title: &str, items: &[String]) {
    println!("┌─{}─┐", "─".repeat(title.len()));
    println!("│ {} │", title);
    println!("├─{}─┤", "─".repeat(title.len()));
    
    for (i, item) in items.iter().enumerate() {
        let marker = if i == self.selected { ">" } else { " " };
        println!("│{} {} │", marker, item);
    }
    
    println!("└─{}─┘", "─".repeat(title.len()));
    println!("ESC: Cancel | Enter: Select");
}
```

### Error Handling

```rust
fn update(&mut self, event: Event) -> bool {
    match event {
        Event::Key(key) => {
            match self.handle_key(key) {
                Ok(should_render) => should_render,
                Err(e) => {
                    self.error_message = Some(format!("Error: {}", e));
                    true
                }
            }
        }
        _ => false
    }
}
```

This guide covers the essential 80% of Zellij plugin development. For advanced features, refer to the complete documentation.