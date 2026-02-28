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
    LaunchWithout,
    CreateNew,
    Existing { name: String },
}

pub struct SpecSelectorTui {
    items: Vec<TuiItem>,
}

impl SpecSelectorTui {
    pub fn new(specs: Vec<(String, bool)>) -> Self {
        let active_specs: Vec<String> = specs
            .into_iter()
            .filter(|(_, is_archived)| !is_archived)
            .map(|(name, _)| name)
            .collect();

        let mut items = vec![TuiItem::LaunchWithout, TuiItem::CreateNew];

        for spec_name in active_specs {
            items.push(TuiItem::Existing { name: spec_name });
        }

        Self { items }
    }

    pub fn run(&mut self) -> Result<SpecSelectionResult> {
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
                                TuiItem::LaunchWithout => SpecSelectionResult::NoSpec,
                                TuiItem::CreateNew => SpecSelectionResult::CreateNew,
                                TuiItem::Existing { name } => {
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

        let title = Paragraph::new("Select a specification to work on").block(
            Block::default()
                .borders(Borders::ALL)
                .title("Specifications"),
        );
        frame.render_widget(title, chunks[0]);

        let mut list_items: Vec<ListItem> = Vec::new();

        for item in &self.items {
            let (text, style) = match item {
                TuiItem::LaunchWithout => (
                    "🚀 Launch without specification".to_string(),
                    Style::default().fg(Color::Cyan),
                ),
                TuiItem::CreateNew => (
                    "📝 Create new specification".to_string(),
                    Style::default().fg(Color::Green),
                ),
                TuiItem::Existing { name } => (format!("   {}", name), Style::default()),
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
    CreateNew,
    NoSpec,
    Cancelled,
}
