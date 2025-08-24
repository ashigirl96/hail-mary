# 設計: complete サブコマンドの実装

## 概要

`complete` サブコマンドは、`.kiro/specs` ディレクトリ内の仕様を完了済みとしてマークするための TUI (Terminal User Interface) を提供します。ユーザーは ratatui を使用したインタラクティブなインターフェースで複数の仕様を選択し、一括で完了処理を実行できます。

## 設計方針

既存のコードベースのパターンに従い、シンプルで実用的な実装を目指します：
- 不要なドメインエンティティは作成しない
- ファイルシステムベースのシンプルな状態管理
- 既存の ProjectRepository パターンを活用

## 実装計画

### 1. 依存関係の追加

**Cargo.toml**
```toml
[dependencies]
# TUI
ratatui = "0.29"
crossterm = "0.28"
```

### 2. CLI層の実装

#### 2.1 コマンド定義の追加 (src/cli/args.rs)

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... 既存のコマンド ...
    
    /// 機能仕様を完了済みとしてマーク
    Complete,
}
```

#### 2.2 CompleteCommand の実装 (src/cli/commands/complete.rs)

```rust
pub struct CompleteCommand;

impl CompleteCommand {
    pub fn new() -> Self {
        Self
    }
    
    pub fn execute(&self) -> Result<()> {
        // 1. PathManager でプロジェクトルートを発見
        // 2. specs ディレクトリから仕様一覧を取得
        // 3. TUI を起動して選択画面を表示
        // 4. 選択された仕様に .completed ファイルを作成
    }
}
```

### 3. TUI コンポーネントの設計

#### 3.1 TUI 状態管理（シンプル版）

```rust
struct App {
    specs: Vec<SpecItem>,          // 仕様のリスト
    selected: HashSet<usize>,      // 選択されたインデックス
    cursor: usize,                  // カーソル位置
}

struct SpecItem {
    name: String,                   // ディレクトリ名
    path: PathBuf,                  // フルパス
    is_completed: bool,             // .completed ファイルの有無
}
```

#### 3.2 キーバインディング

| キー | アクション |
|------|-----------|
| ↑/k | カーソルを上に移動 |
| ↓/j | カーソルを下に移動 |
| Space | 現在の項目を選択/選択解除 |
| Enter | 選択した項目を完了 |
| q/Esc | キャンセルして終了 |

#### 3.3 UI レイアウト

```
> hail-mary complete
Selected: 1 items
[ ] 2025-08-23-user-authentication
[x] 2025-08-23-goose-experiment
[ ] 2025-08-22-database-migration

Space: Select  Enter: Archive  q: Quit
```

### 4. Application層の実装

#### 4.1 Use Case (src/application/use_cases/complete_features.rs)

```rust
pub fn complete_features(
    project_repo: &dyn ProjectRepository,
    spec_names: &[String],
) -> Result<(), ApplicationError> {
    for name in spec_names {
        project_repo.mark_spec_complete(name)?;
    }
    Ok(())
}
```

### 5. Repository層の拡張

#### 5.1 ProjectRepository trait の拡張

```rust
pub trait ProjectRepository {
    // ... 既存のメソッド ...
    
    /// すべての仕様ディレクトリをリスト
    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError>;
    
    /// 仕様を完了済みとしてマーク
    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError>;
}
```

#### 5.2 実装の追加

```rust
impl ProjectRepositoryTrait for ProjectRepository {
    fn list_spec_directories(&self) -> Result<Vec<(String, bool)>, ApplicationError> {
        let specs_dir = self.path_manager.specs_dir(true);
        let mut specs = Vec::new();
        
        if !specs_dir.exists() {
            return Ok(specs);
        }
        
        for entry in fs::read_dir(specs_dir)
            .map_err(|e| ApplicationError::FileSystemError(format!("Failed to read specs directory: {}", e)))? 
        {
            let entry = entry
                .map_err(|e| ApplicationError::FileSystemError(format!("Failed to read directory entry: {}", e)))?;
            
            if entry.file_type()
                .map_err(|e| ApplicationError::FileSystemError(format!("Failed to get file type: {}", e)))?
                .is_dir() 
            {
                let name = entry.file_name().to_string_lossy().to_string();
                specs.push((name, false)); // アーカイブでない限りfalse
            }
        }
        
        // 名前でソート（日付順になる）
        specs.sort_by(|a, b| b.0.cmp(&a.0)); // 新しい順
        Ok(specs)
    }
    
    fn mark_spec_complete(&self, name: &str) -> Result<(), ApplicationError> {
        let source_path = self.path_manager.specs_dir(true).join(name);
        
        if !source_path.exists() {
            return Err(ApplicationError::SpecNotFound(name.to_string()));
        }
        
        if !source_path.is_dir() {
            return Err(ApplicationError::InvalidSpecDirectory(name.to_string()));
        }
        
        // archive ディレクトリを作成
        let archive_dir = self.path_manager.archive_dir(true);
        fs::create_dir_all(&archive_dir)
            .map_err(|e| ApplicationError::FileSystemError(format!("Failed to create archive directory: {}", e)))?;
        
        let dest_path = archive_dir.join(name);
        
        // 同名のディレクトリが既に存在する場合はエラー
        if dest_path.exists() {
            return Err(ApplicationError::ArchiveAlreadyExists(name.to_string()));
        }
        
        // ディレクトリを移動
        fs::rename(&source_path, &dest_path)
            .map_err(|e| ApplicationError::FileSystemError(format!("Failed to move spec to archive: {}", e)))?;
        
        Ok(())
    }
}
```

### 6. main.rs の更新

```rust
Commands::Complete => {
    let command = CompleteCommand::new();
    command.execute()?;
}
```

## 実装順序

1. **フェーズ1: 基盤準備**
   - [x] design.md を更新
   - [ ] Cargo.toml に依存関係を追加

2. **フェーズ2: Repository 層**
   - [ ] ProjectRepository trait にメソッドを追加
   - [ ] 実装を追加

3. **フェーズ3: Application 層**
   - [ ] complete_features use case を作成

4. **フェーズ4: CLI 層**
   - [ ] args.rs に Complete コマンドを追加
   - [ ] CompleteCommand を実装

5. **フェーズ5: TUI 実装**
   - [ ] TUI の状態管理を実装
   - [ ] キーボードイベントハンドリング
   - [ ] UI レンダリング

6. **フェーズ6: 統合**
   - [ ] main.rs でコマンドディスパッチを追加
   - [ ] テストを作成

## エラーハンドリング

- プロジェクトが初期化されていない場合のエラー
- specs ディレクトリが空の場合の処理
- TUI の描画エラー
- ファイルシステムのアクセス権限エラー

## テスト戦略

1. **単体テスト**
   - complete_features use case のテスト
   - ProjectRepository の新メソッドのテスト

2. **統合テスト**
   - コマンド実行の E2E テスト
   - アーカイブディレクトリへの移動検証

## 完了処理の仕様

完了した仕様は `.kiro/archive` ディレクトリに移動：
- 選択された仕様ディレクトリを `.kiro/specs` から `.kiro/archive` に移動
- アーカイブされた仕様は通常のリストには表示されない
- ディレクトリ構造は保持される

## 詳細実装

### PathManager の拡張 (src/infrastructure/filesystem/path_manager.rs)

```rust
impl PathManager {
    /// Get archive directory path
    pub fn archive_dir(&self, absolute: bool) -> PathBuf {
        let path = self.kiro_dir(absolute).join("archive");
        if absolute {
            path
        } else {
            path.strip_prefix(&self.project_root)
                .unwrap_or(&path)
                .to_path_buf()
        }
    }
}
```

### ApplicationError の拡張 (src/application/errors.rs)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    // ... 既存のエラー ...
    
    #[error("Spec directory not found: {0}")]
    SpecNotFound(String),
    
    #[error("Invalid spec directory: {0}")]
    InvalidSpecDirectory(String),
    
    #[error("Spec already exists in archive: {0}")]
    ArchiveAlreadyExists(String),
}
```

### TUI実装の詳細 (src/cli/commands/complete.rs)

```rust
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::collections::HashSet;
use std::io;

pub struct CompleteCommand;

impl CompleteCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self) -> Result<()> {
        // プロジェクトルートを発見
        let path_manager = match PathManager::discover() {
            Ok(pm) => pm,
            Err(_) => {
                println!("{}", format_error("Not in a project directory. Run 'hail-mary init' first."));
                return Err(anyhow::anyhow!("Project not found"));
            }
        };

        // プロジェクトリポジトリを作成
        let project_repo = ProjectRepository::new(path_manager);

        // 仕様一覧を取得
        let specs = match project_repo.list_spec_directories() {
            Ok(specs) => specs,
            Err(e) => {
                println!("{}", format_error(&e.to_string()));
                return Err(anyhow::anyhow!(e));
            }
        };

        if specs.is_empty() {
            println!("{}", format_error("No specifications found in .kiro/specs directory."));
            return Ok(());
        }

        // TUIを起動
        self.run_tui(specs, &project_repo)
    }

    fn run_tui(
        &self,
        specs: Vec<(String, bool)>,
        project_repo: &ProjectRepository,
    ) -> Result<()> {
        // ターミナル初期化
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // アプリケーション状態
        let mut app = App::new(specs);
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        // メインループ
        let result = loop {
            // UIを描画
            terminal.draw(|f| self.draw_ui(f, &mut app, &mut list_state))?;

            // イベント処理
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break Ok(());
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.move_cursor_up(&mut list_state);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.move_cursor_down(&mut list_state);
                        }
                        KeyCode::Char(' ') => {
                            app.toggle_selection(&list_state);
                        }
                        KeyCode::Enter => {
                            let selected_specs = app.get_selected_specs();
                            if selected_specs.is_empty() {
                                continue;
                            }

                            // 完了処理を実行
                            match complete_features(project_repo, &selected_specs) {
                                Ok(()) => {
                                    break Ok(());
                                }
                                Err(e) => {
                                    break Err(anyhow::anyhow!(e));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        };

        // ターミナルクリーンアップ
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        // 結果表示
        match result {
            Ok(()) => {
                let selected_count = app.selected.len();
                if selected_count > 0 {
                    println!(
                        "{}",
                        format_success(&format!(
                            "{} specification(s) moved to archive successfully.",
                            selected_count
                        ))
                    );
                }
            }
            Err(e) => {
                println!("{}", format_error(&e.to_string()));
                return Err(e);
            }
        }

        Ok(())
    }

    fn draw_ui(&self, f: &mut Frame, app: &mut App, list_state: &mut ListState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Selected count
                Constraint::Min(0),    // Spec list
                Constraint::Length(1), // Help
            ])
            .split(f.size());

        // Selected count
        let selected_text = format!("Selected: {} items", app.selected.len());
        let selected_paragraph = Paragraph::new(selected_text);
        f.render_widget(selected_paragraph, chunks[0]);

        // 仕様リスト（シンプル版）
        let items: Vec<ListItem> = app
            .specs
            .iter()
            .enumerate()
            .map(|(i, (name, _))| {
                let checkbox = if app.selected.contains(&i) { "[x]" } else { "[ ]" };
                let content = format!("{} {}", checkbox, name);
                
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
            );

        f.render_stateful_widget(list, chunks[1], list_state);

        // ヘルプテキスト（シンプル版）
        let help_text = "Space: Select  Enter: Archive  q: Quit";
        let help_paragraph = Paragraph::new(help_text);
        f.render_widget(help_paragraph, chunks[2]);
    }
}

struct App {
    specs: Vec<(String, bool)>,
    selected: HashSet<usize>,
}

impl App {
    fn new(specs: Vec<(String, bool)>) -> Self {
        Self {
            specs,
            selected: HashSet::new(),
        }
    }

    fn move_cursor_up(&self, list_state: &mut ListState) {
        let i = match list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.specs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }

    fn move_cursor_down(&self, list_state: &mut ListState) {
        let i = match list_state.selected() {
            Some(i) => {
                if i >= self.specs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }

    fn toggle_selection(&mut self, list_state: &ListState) {
        if let Some(i) = list_state.selected() {
            if self.selected.contains(&i) {
                self.selected.remove(&i);
            } else {
                self.selected.insert(i);
            }
        }
    }

    fn get_selected_specs(&self) -> Vec<String> {
        self.selected
            .iter()
            .map(|&i| self.specs[i].0.clone())
            .collect()
    }
}
```

## 将来の拡張性

- アーカイブした仕様の一覧表示機能
- アーカイブからの復元機能  
- 仕様の検索・フィルタリング
- バッチ操作（全選択、全解除）