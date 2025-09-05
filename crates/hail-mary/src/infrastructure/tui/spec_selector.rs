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
use std::io;

pub struct SpecSelectorTui {
    specs: Vec<String>,
    has_new_option: bool,
}

impl SpecSelectorTui {
    pub fn new(specs: Vec<(String, bool)>) -> Self {
        // Filter out archived specs and extract names
        let active_specs: Vec<String> = specs
            .into_iter()
            .filter(|(_, is_archived)| !is_archived)
            .map(|(name, _)| name)
            .collect();

        Self {
            specs: active_specs,
            has_new_option: true,
        }
    }

    pub fn run(&mut self) -> Result<SpecSelectionResult> {
        // Terminal initialization
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let result = loop {
            terminal.draw(|f| self.draw_ui(f, &mut list_state))?;

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break Ok(SpecSelectionResult::Cancelled);
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = list_state.selected() {
                            if self.has_new_option && selected == 0 {
                                break Ok(SpecSelectionResult::CreateNew);
                            } else {
                                let index = if self.has_new_option {
                                    selected - 1
                                } else {
                                    selected
                                };
                                if index < self.specs.len() {
                                    break Ok(SpecSelectionResult::Existing(
                                        self.specs[index].clone(),
                                    ));
                                }
                            }
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.move_cursor_up(&mut list_state);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.move_cursor_down(&mut list_state);
                    }
                    _ => {}
                }
            }
        };

        // Terminal cleanup
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        result
    }

    fn draw_ui(&self, frame: &mut Frame, list_state: &mut ListState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Title
        let title = Paragraph::new("Select a specification to work on").block(
            Block::default()
                .borders(Borders::ALL)
                .title("Kiro Specifications"),
        );
        frame.render_widget(title, chunks[0]);

        // List items
        let mut items: Vec<ListItem> = Vec::new();

        if self.has_new_option {
            items.push(
                ListItem::new("📝 Create new specification")
                    .style(Style::default().fg(Color::Green)),
            );
        }

        for spec in &self.specs {
            items.push(ListItem::new(format!("  {}", spec)));
        }

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[1], list_state);

        // Instructions
        let instructions = Paragraph::new("↑/↓/j/k: Navigate | Enter: Select | q/Esc: Cancel")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(instructions, chunks[2]);
    }

    fn move_cursor_up(&self, list_state: &mut ListState) {
        let total_items = self.specs.len() + if self.has_new_option { 1 } else { 0 };
        let i = match list_state.selected() {
            Some(i) => {
                if i == 0 {
                    total_items - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }

    fn move_cursor_down(&self, list_state: &mut ListState) {
        let total_items = self.specs.len() + if self.has_new_option { 1 } else { 0 };
        let i = match list_state.selected() {
            Some(i) => {
                if i >= total_items - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecSelectionResult {
    Existing(String),
    CreateNew,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_selector_new() {
        let specs = vec![
            ("feature1".to_string(), false),
            ("feature2".to_string(), false),
            ("archived-feature".to_string(), true),
        ];

        let selector = SpecSelectorTui::new(specs);

        // Should filter out archived specs
        assert_eq!(selector.specs.len(), 2);
        assert!(selector.specs.contains(&"feature1".to_string()));
        assert!(selector.specs.contains(&"feature2".to_string()));
        assert!(!selector.specs.contains(&"archived-feature".to_string()));
        assert!(selector.has_new_option);
    }

    #[test]
    fn test_spec_selector_with_empty_specs() {
        let specs = vec![];
        let selector = SpecSelectorTui::new(specs);

        assert_eq!(selector.specs.len(), 0);
        assert!(selector.has_new_option);
    }

    #[test]
    fn test_spec_selector_with_only_archived() {
        let specs = vec![
            ("archived1".to_string(), true),
            ("archived2".to_string(), true),
        ];

        let selector = SpecSelectorTui::new(specs);

        assert_eq!(selector.specs.len(), 0);
        assert!(selector.has_new_option);
    }

    // Note: We don't test run() method in unit tests as it requires terminal interaction
    // This should be tested in integration tests with mock terminals
}
