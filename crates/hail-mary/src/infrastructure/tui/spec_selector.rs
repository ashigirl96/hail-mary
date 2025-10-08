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

#[derive(Debug, Clone)]
enum TuiItem {
    LaunchWithoutSpec,
    CreateNewSpec,
    Pbi {
        name: String,
        #[allow(dead_code)]
        sbis: Vec<String>,
    },
    Sbi {
        pbi_name: String,
        sbi_name: String,
    },
    CreateNewSbi {
        pbi_name: String,
    },
    SingleSpec {
        name: String,
    },
}

pub struct SpecSelectorTui {
    items: Vec<TuiItem>,
}

impl SpecSelectorTui {
    pub fn new(
        specs: Vec<(String, bool)>,
        spec_repo: &dyn crate::application::repositories::SpecRepositoryInterface,
    ) -> Self {
        // Filter out archived specs
        let active_specs: Vec<String> = specs
            .into_iter()
            .filter(|(_, is_archived)| !is_archived)
            .map(|(name, _)| name)
            .collect();

        // Build TUI items with SBI detection
        let mut items = vec![TuiItem::LaunchWithoutSpec, TuiItem::CreateNewSpec];

        for spec_name in active_specs {
            // Check if this is a PBI
            if let Ok(true) = spec_repo.is_pbi(&spec_name) {
                // Get SBIs
                if let Ok(sbis) = spec_repo.list_sbis(&spec_name) {
                    items.push(TuiItem::Pbi {
                        name: spec_name.clone(),
                        sbis: sbis.clone(),
                    });

                    // Add each SBI as selectable item
                    for sbi_name in sbis {
                        items.push(TuiItem::Sbi {
                            pbi_name: spec_name.clone(),
                            sbi_name,
                        });
                    }

                    // Add "Create new SBI" option
                    items.push(TuiItem::CreateNewSbi {
                        pbi_name: spec_name,
                    });
                } else {
                    // Fallback to single spec
                    items.push(TuiItem::SingleSpec { name: spec_name });
                }
            } else {
                // Single spec (no SBIs)
                items.push(TuiItem::SingleSpec { name: spec_name });
            }
        }

        Self { items }
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
                        if let Some(selected) = list_state.selected()
                            && selected < self.items.len()
                        {
                            let result = match &self.items[selected] {
                                TuiItem::LaunchWithoutSpec => SpecSelectionResult::NoSpec,
                                TuiItem::CreateNewSpec => SpecSelectionResult::CreateNew,
                                TuiItem::Pbi { name, .. } => SpecSelectionResult::Pbi(name.clone()),
                                TuiItem::Sbi { pbi_name, sbi_name } => {
                                    SpecSelectionResult::Sbi(pbi_name.clone(), sbi_name.clone())
                                }
                                TuiItem::CreateNewSbi { pbi_name } => {
                                    SpecSelectionResult::CreateNewSbi(pbi_name.clone())
                                }
                                TuiItem::SingleSpec { name } => {
                                    SpecSelectionResult::SingleSpec(name.clone())
                                }
                            };
                            break Ok(result);
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
        let mut list_items: Vec<ListItem> = Vec::new();

        for item in &self.items {
            let (text, style) = match item {
                TuiItem::LaunchWithoutSpec => (
                    "🚀 Launch without specification".to_string(),
                    Style::default().fg(Color::Cyan),
                ),
                TuiItem::CreateNewSpec => (
                    "📝 Create new specification".to_string(),
                    Style::default().fg(Color::Green),
                ),
                TuiItem::Pbi { name, .. } => (format!("   {}", name), Style::default()),
                TuiItem::Sbi { sbi_name, .. } => (
                    format!("     {}", sbi_name),
                    Style::default().fg(Color::Yellow),
                ),
                TuiItem::CreateNewSbi { .. } => (
                    "     📝 Create new SBI specification".to_string(),
                    Style::default().fg(Color::Green),
                ),
                TuiItem::SingleSpec { name } => (format!("   {}", name), Style::default()),
            };
            list_items.push(ListItem::new(text).style(style));
        }

        let list = List::new(list_items)
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
        let total_items = self.items.len();
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
        let total_items = self.items.len();
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
    SingleSpec(String),
    Pbi(String),
    Sbi(String, String), // (pbi_name, sbi_name)
    CreateNew,
    CreateNewSbi(String), // pbi_name
    NoSpec,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_helpers::MockSpecRepository;

    #[test]
    fn test_spec_selector_new() {
        let specs = vec![
            ("feature1".to_string(), false),
            ("feature2".to_string(), false),
            ("archived-feature".to_string(), true),
        ];

        let mock_repo = MockSpecRepository::new();
        let selector = SpecSelectorTui::new(specs, &mock_repo);

        // Should have 2 default items + 2 single specs
        assert_eq!(selector.items.len(), 4);
    }

    #[test]
    fn test_spec_selector_with_empty_specs() {
        let specs = vec![];
        let mock_repo = MockSpecRepository::new();
        let selector = SpecSelectorTui::new(specs, &mock_repo);

        // Should have only 2 default items
        assert_eq!(selector.items.len(), 2);
    }

    // Note: We don't test run() method in unit tests as it requires terminal interaction
    // This should be tested in integration tests with mock terminals
}
