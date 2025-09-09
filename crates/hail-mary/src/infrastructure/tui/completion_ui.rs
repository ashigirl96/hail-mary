use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::collections::HashSet;
use std::io;

/// Displays a TUI for selecting specifications to complete
pub fn select_specs_for_completion(specs: Vec<(String, bool)>) -> Result<Option<Vec<String>>> {
    if specs.is_empty() {
        return Ok(Some(Vec::new()));
    }

    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Application state
    let mut app = App::new(specs);
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    // Main loop
    let result = loop {
        // Draw UI
        terminal.draw(|f| draw_ui(f, &mut app, &mut list_state))?;

        // Event handling
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    break Ok(None); // User cancelled
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
                    break Ok(Some(selected_specs));
                }
                _ => {}
            }
        }
    };

    // Terminal cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn draw_ui(f: &mut Frame, app: &mut App, list_state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Selected count
            Constraint::Min(0),    // Spec list
            Constraint::Length(1), // Help
        ])
        .split(f.area());

    // Selected count
    let selected_text = format!("Selected: {} items", app.selected.len());
    let selected_paragraph = Paragraph::new(selected_text).style(Style::default().fg(Color::Cyan));
    f.render_widget(selected_paragraph, chunks[0]);

    // Spec list
    let items: Vec<ListItem> = app
        .specs
        .iter()
        .enumerate()
        .map(|(i, (name, _))| {
            let checkbox = if app.selected.contains(&i) {
                "[x]"
            } else {
                "[ ]"
            };
            let content = format!("{} {}", checkbox, name);

            let style = if app.selected.contains(&i) {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Specifications"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_stateful_widget(list, chunks[1], list_state);

    // Help text
    let help_text = "Space: Select  Enter: Archive  q: Quit  ↑↓/jk: Navigate";
    let help_paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::Gray));
    f.render_widget(help_paragraph, chunks[2]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new() {
        let specs = vec![
            ("2025-01-01-feature-a".to_string(), false),
            ("2025-01-02-feature-b".to_string(), false),
        ];
        let app = App::new(specs.clone());

        assert_eq!(app.specs.len(), 2);
        assert!(app.selected.is_empty());
    }

    #[test]
    fn test_app_toggle_selection() {
        let specs = vec![
            ("2025-01-01-feature-a".to_string(), false),
            ("2025-01-02-feature-b".to_string(), false),
        ];
        let mut app = App::new(specs);
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        // Toggle selection on
        app.toggle_selection(&list_state);
        assert!(app.selected.contains(&0));

        // Toggle selection off
        app.toggle_selection(&list_state);
        assert!(!app.selected.contains(&0));
    }

    #[test]
    fn test_app_get_selected_specs() {
        let specs = vec![
            ("2025-01-01-feature-a".to_string(), false),
            ("2025-01-02-feature-b".to_string(), false),
            ("2025-01-03-feature-c".to_string(), false),
        ];
        let mut app = App::new(specs);

        app.selected.insert(0);
        app.selected.insert(2);

        let selected = app.get_selected_specs();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&"2025-01-01-feature-a".to_string()));
        assert!(selected.contains(&"2025-01-03-feature-c".to_string()));
    }

    #[test]
    fn test_app_move_cursor_up() {
        let specs = vec![
            ("2025-01-01-feature-a".to_string(), false),
            ("2025-01-02-feature-b".to_string(), false),
            ("2025-01-03-feature-c".to_string(), false),
        ];
        let app = App::new(specs);
        let mut list_state = ListState::default();

        // Start at position 1
        list_state.select(Some(1));

        // Move up to position 0
        app.move_cursor_up(&mut list_state);
        assert_eq!(list_state.selected(), Some(0));

        // Move up from 0 wraps to last position (2)
        app.move_cursor_up(&mut list_state);
        assert_eq!(list_state.selected(), Some(2));
    }

    #[test]
    fn test_app_move_cursor_down() {
        let specs = vec![
            ("2025-01-01-feature-a".to_string(), false),
            ("2025-01-02-feature-b".to_string(), false),
            ("2025-01-03-feature-c".to_string(), false),
        ];
        let app = App::new(specs);
        let mut list_state = ListState::default();

        // Start at position 1
        list_state.select(Some(1));

        // Move down to position 2
        app.move_cursor_down(&mut list_state);
        assert_eq!(list_state.selected(), Some(2));

        // Move down from last position wraps to 0
        app.move_cursor_down(&mut list_state);
        assert_eq!(list_state.selected(), Some(0));
    }
}
