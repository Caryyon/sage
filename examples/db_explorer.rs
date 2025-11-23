//! SpacetimeDB Explorer - A simple TUI for browsing SAGE database
//!
//! Usage: cargo run --release --example db_explorer
//!
//! Keyboard (vim-style):
//!   j/k or Up/Down  - Navigate lists
//!   h/l or Tab      - Switch panels (Tables -> Content -> Query)
//!   g/G             - Jump to top/bottom
//!   Ctrl+d/u        - Half-page scroll
//!   Ctrl+f/b        - Full page scroll
//!   PgUp/PgDn       - Page through database results
//!   Enter           - Execute query (in query mode) / Enter content panel
//!   r               - Refresh data
//!   q               - Quit

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::io;
use std::process::Command;
use std::time::Duration;

// All SAGE tables
const TABLES: &[&str] = &[
    "sage_state",
    "training_metrics",
    "network_snapshots",
    "pattern_progress",
    "training_events",
    "conversations",
    "concept_memory",
    "opinion_history",
    "personality_snapshots",
    "introspection_journal",
    "autonomous_activity",
    "ab_test_results",
    "visual_memories",
    "messages",
    "user_facts",
    "meta_strategy_history",
    "hyperparameter_evolution",
    "architecture_changes",
    "few_shot_adaptations",
    "optimizer_snapshots",
];

#[derive(PartialEq, Clone, Copy)]
enum Panel {
    Tables,
    Content,
    Query,
}

struct App {
    db_name: String,
    tables: Vec<TableInfo>,
    table_state: ListState,
    content_rows: Vec<Vec<String>>,
    content_headers: Vec<String>,
    content_state: TableState,
    current_panel: Panel,
    query_input: String,
    query_result: Option<String>,
    status_message: String,
    page_size: usize,
    current_page: usize,
    total_rows: usize,
}

struct TableInfo {
    name: String,
    row_count: usize,
}

impl App {
    fn new(db_name: &str) -> Self {
        let mut app = Self {
            db_name: db_name.to_string(),
            tables: Vec::new(),
            table_state: ListState::default(),
            content_rows: Vec::new(),
            content_headers: Vec::new(),
            content_state: TableState::default(),
            current_panel: Panel::Tables,
            query_input: String::new(),
            query_result: None,
            status_message: "Loading...".to_string(),
            page_size: 20,
            current_page: 0,
            total_rows: 0,
        };
        app.table_state.select(Some(0));
        app.refresh_tables();
        app
    }

    fn refresh_tables(&mut self) {
        self.tables.clear();
        for table in TABLES {
            let count = self.get_row_count(table);
            self.tables.push(TableInfo {
                name: table.to_string(),
                row_count: count,
            });
        }
        self.status_message = format!("Loaded {} tables", self.tables.len());

        // Load first table content
        if !self.tables.is_empty() {
            self.load_table_content(0);
        }
    }

    fn get_row_count(&self, table: &str) -> usize {
        let output = Command::new("spacetime")
            .args(["sql", &self.db_name, &format!("SELECT COUNT(*) as cnt FROM {}", table)])
            .output();

        match output {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout);
                // Parse count from output - look for numeric line after header
                text.lines()
                    .filter(|l| !l.trim().is_empty() && !l.contains("cnt") && !l.contains("---") && !l.contains("WARNING"))
                    .next()
                    .and_then(|l| l.trim().parse().ok())
                    .unwrap_or(0)
            }
            Err(_) => 0,
        }
    }

    fn load_table_content(&mut self, table_idx: usize) {
        if table_idx >= self.tables.len() {
            return;
        }

        let table_name = &self.tables[table_idx].name;

        // SpacetimeDB doesn't support LIMIT/OFFSET, so fetch all rows
        let query = format!("SELECT * FROM {}", table_name);

        self.execute_query_internal(&query, true);
        self.total_rows = self.content_rows.len();
        self.content_state.select(Some(0));
    }

    fn execute_query_internal(&mut self, query: &str, update_content: bool) {
        let output = Command::new("spacetime")
            .args(["sql", &self.db_name, query])
            .output();

        match output {
            Ok(out) => {
                let text = String::from_utf8_lossy(&out.stdout).to_string();

                if update_content {
                    self.parse_sql_output(&text);
                } else {
                    self.query_result = Some(text);
                }
                self.status_message = "Query executed successfully".to_string();
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                if !update_content {
                    self.query_result = Some(format!("Error: {}", e));
                }
            }
        }
    }

    fn parse_sql_output(&mut self, output: &str) {
        self.content_headers.clear();
        self.content_rows.clear();

        let lines: Vec<&str> = output.lines().collect();
        if lines.is_empty() {
            return;
        }

        // Find header line (contains column names)
        let mut header_idx = None;
        let mut separator_idx = None;

        for (i, line) in lines.iter().enumerate() {
            if line.contains("---") || line.contains("═") {
                separator_idx = Some(i);
                if i > 0 {
                    header_idx = Some(i - 1);
                }
                break;
            }
        }

        // Parse headers
        if let Some(idx) = header_idx {
            self.content_headers = lines[idx]
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Parse data rows
        let start_idx = separator_idx.map(|i| i + 1).unwrap_or(0);
        for line in lines.iter().skip(start_idx) {
            let line = line.trim();
            if line.is_empty() || line.starts_with('(') {
                continue;
            }

            let cells: Vec<String> = line
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if !cells.is_empty() && cells.len() == self.content_headers.len() {
                self.content_rows.push(cells);
            }
        }
    }

    fn execute_user_query(&mut self) {
        if self.query_input.trim().is_empty() {
            return;
        }
        self.execute_query_internal(&self.query_input.clone(), false);
    }

    fn next_table(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.tables.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
        self.current_page = 0;
        self.load_table_content(i);
    }

    fn prev_table(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tables.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
        self.current_page = 0;
        self.load_table_content(i);
    }

    fn next_row(&mut self) {
        let i = match self.content_state.selected() {
            Some(i) => {
                if i >= self.content_rows.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.content_state.select(Some(i));
    }

    fn prev_row(&mut self) {
        let i = match self.content_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.content_rows.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.content_state.select(Some(i));
    }

    fn next_page(&mut self) {
        let max_page = self.total_rows / self.page_size;
        if self.current_page < max_page {
            self.current_page += 1;
            if let Some(idx) = self.table_state.selected() {
                self.load_table_content(idx);
            }
        }
    }

    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            if let Some(idx) = self.table_state.selected() {
                self.load_table_content(idx);
            }
        }
    }

    fn cycle_panel(&mut self) {
        self.current_panel = match self.current_panel {
            Panel::Tables => Panel::Content,
            Panel::Content => Panel::Query,
            Panel::Query => Panel::Tables,
        };
    }

    fn cycle_panel_back(&mut self) {
        self.current_panel = match self.current_panel {
            Panel::Tables => Panel::Query,
            Panel::Content => Panel::Tables,
            Panel::Query => Panel::Content,
        };
    }

    fn goto_top(&mut self) {
        if self.current_panel == Panel::Tables {
            self.table_state.select(Some(0));
            self.current_page = 0;
            self.load_table_content(0);
        } else {
            self.content_state.select(Some(0));
        }
    }

    fn goto_bottom(&mut self) {
        if self.current_panel == Panel::Tables {
            let last = self.tables.len().saturating_sub(1);
            self.table_state.select(Some(last));
            self.current_page = 0;
            self.load_table_content(last);
        } else {
            let last = self.content_rows.len().saturating_sub(1);
            self.content_state.select(Some(last));
        }
    }

    fn half_page_down(&mut self) {
        let half = self.page_size / 2;
        for _ in 0..half {
            if self.current_panel == Panel::Tables {
                self.next_table();
            } else {
                self.next_row();
            }
        }
    }

    fn half_page_up(&mut self) {
        let half = self.page_size / 2;
        for _ in 0..half {
            if self.current_panel == Panel::Tables {
                self.prev_table();
            } else {
                self.prev_row();
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new("sage-db");

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.current_panel {
                    Panel::Query => {
                        match key.code {
                            KeyCode::Esc | KeyCode::Tab => app.cycle_panel(),
                            KeyCode::Enter => app.execute_user_query(),
                            KeyCode::Backspace => { app.query_input.pop(); }
                            KeyCode::Char(c) => {
                                if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                                    break;
                                }
                                app.query_input.push(c);
                            }
                            _ => {}
                        }
                    }
                    _ => {
                        match key.code {
                            // Quit
                            KeyCode::Char('q') => break,
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,

                            // Refresh
                            KeyCode::Char('r') => app.refresh_tables(),

                            // Panel navigation (vim: h/l, also Tab/Shift+Tab)
                            KeyCode::Tab => app.cycle_panel(),
                            KeyCode::BackTab => app.cycle_panel_back(),
                            KeyCode::Char('l') => app.cycle_panel(),
                            KeyCode::Char('h') => app.cycle_panel_back(),

                            // Vertical navigation (vim: j/k, also arrows)
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.current_panel == Panel::Tables {
                                    app.prev_table();
                                } else {
                                    app.prev_row();
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.current_panel == Panel::Tables {
                                    app.next_table();
                                } else {
                                    app.next_row();
                                }
                            }

                            // Page navigation (vim: Ctrl+d/u, also PgUp/PgDn)
                            KeyCode::PageUp => app.prev_page(),
                            KeyCode::PageDown => app.next_page(),
                            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => app.half_page_up(),
                            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => app.half_page_down(),
                            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => app.prev_page(),
                            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => app.next_page(),

                            // Jump to top/bottom (vim: g/G)
                            KeyCode::Char('g') => app.goto_top(),
                            KeyCode::Char('G') => app.goto_bottom(),

                            // Enter content panel from tables
                            KeyCode::Enter => {
                                if app.current_panel == Panel::Tables {
                                    app.current_panel = Panel::Content;
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.size());

    // Main area: Tables list + Content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(40)])
        .split(chunks[0]);

    // Tables list
    render_tables_list(f, app, main_chunks[0]);

    // Content area (table data or query result)
    render_content(f, app, main_chunks[1]);

    // Query input
    render_query_input(f, app, chunks[1]);

    // Status bar
    render_status_bar(f, app, chunks[2]);
}

fn render_tables_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .tables
        .iter()
        .map(|t| {
            let content = format!("{:<20} {:>5}", t.name, t.row_count);
            ListItem::new(content)
        })
        .collect();

    let highlight_style = if app.current_panel == Panel::Tables {
        Style::default().bg(Color::Cyan).fg(Color::Black)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };

    let border_style = if app.current_panel == Panel::Tables {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Tables ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(highlight_style)
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.table_state);
}

fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    // If we have query results, show those
    if let Some(ref result) = app.query_result {
        let border_style = if app.current_panel == Panel::Content {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let paragraph = Paragraph::new(result.as_str())
            .block(
                Block::default()
                    .title(" Query Result ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(paragraph, area);
        return;
    }

    // Otherwise show table content
    let selected_table = app
        .table_state
        .selected()
        .and_then(|i| app.tables.get(i))
        .map(|t| t.name.as_str())
        .unwrap_or("(none)");

    let page_info = format!("{} rows", app.total_rows);

    let title = format!(" {} - {} ", selected_table, page_info);

    let border_style = if app.current_panel == Panel::Content {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    if app.content_headers.is_empty() {
        let paragraph = Paragraph::new("No data")
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );
        f.render_widget(paragraph, area);
        return;
    }

    // Calculate column widths
    let col_count = app.content_headers.len();
    let available_width = area.width.saturating_sub(2) as usize; // borders
    let col_width = (available_width / col_count).max(8);

    let header_cells = app.content_headers.iter().map(|h| {
        Cell::from(truncate_str(h, col_width))
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    });
    let header = Row::new(header_cells).height(1);

    let rows = app.content_rows.iter().map(|row| {
        let cells = row.iter().map(|c| {
            Cell::from(truncate_str(c, col_width))
        });
        Row::new(cells).height(1)
    });

    let widths: Vec<Constraint> = (0..col_count)
        .map(|_| Constraint::Length(col_width as u16))
        .collect();

    let highlight_style = if app.current_panel == Panel::Content {
        Style::default().bg(Color::Cyan).fg(Color::Black)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };

    let table = Table::new(rows, &widths)
        .header(header)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(highlight_style)
        .highlight_symbol("> ");

    f.render_stateful_widget(table, area, &mut app.content_state);
}

fn render_query_input(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.current_panel == Panel::Query {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let input = Paragraph::new(app.query_input.as_str())
        .block(
            Block::default()
                .title(" SQL Query (Tab to focus, Enter to execute) ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(input, area);

    // Show cursor in query mode
    if app.current_panel == Panel::Query {
        f.set_cursor(
            area.x + app.query_input.len() as u16 + 1,
            area.y + 1,
        );
    }
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode = match app.current_panel {
        Panel::Tables => "TABLES",
        Panel::Content => "CONTENT",
        Panel::Query => "QUERY",
    };

    let status = Line::from(vec![
        Span::styled(
            format!(" {} ", mode),
            Style::default().bg(Color::Cyan).fg(Color::Black),
        ),
        Span::raw(" "),
        Span::styled(&app.status_message, Style::default().fg(Color::Gray)),
        Span::raw(" | "),
        Span::styled(
            "j/k:Nav  h/l:Panels  g/G:Top/Bot  ^d/^u:Scroll  r:Refresh  q:Quit",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let paragraph = Paragraph::new(status);
    f.render_widget(paragraph, area);
}

fn truncate_str(s: &str, max_len: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_len {
        s.to_string()
    } else {
        let truncated: String = chars[..max_len.saturating_sub(3)].iter().collect();
        format!("{}...", truncated)
    }
}
