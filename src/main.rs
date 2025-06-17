use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pinyin::ToPinyin;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use rusqlite::Connection;
use serde::Deserialize;
use std::{
    fs,
    io,
    path::{Path, PathBuf},
    time::Duration,
};
use unicode_width::UnicodeWidthStr;

// --- 主题和配色 ---
struct Theme {
    fg: Color,
    // **修复**: 移除了未使用的 'bg' 字段
    input_fg: Color,
    border: Color,
    header_fg: Color,
    selection_style: Style,
    status_fg: Color,
}

impl Theme {
    fn terminal_default() -> Self {
        Self {
            fg: Color::Reset,
            input_fg: Color::Yellow,
            border: Color::Blue,
            header_fg: Color::Blue,
            selection_style: Style::default().bg(Color::Blue).fg(Color::White),
            status_fg: Color::DarkGray,
        }
    }
}

// --- 配置结构体 ---
#[derive(Deserialize, Debug, Default)]
struct Config {
    library_path: Option<String>,
}

/// 代表一本书的信息
#[derive(Debug, Clone)]
struct Book {
    title: String,
    author: String,
    path: String,
    title_pinyin: String,
    author_pinyin: String,
}

/// 应用程序的状态
struct App {
    all_books: Vec<Book>,
    filtered_books: Vec<Book>,
    table_state: TableState,
    input: String,
    library_path: PathBuf,
    should_quit: bool,
    status_message: Option<String>,
}

impl App {
    fn new(library_path: PathBuf) -> Result<Self> {
        let all_books = load_books_from_db(&library_path)?;
        let filtered_books = all_books.clone();
        let mut table_state = TableState::default();
        if !filtered_books.is_empty() {
            table_state.select(Some(0));
        }

        Ok(Self {
            all_books,
            filtered_books,
            table_state,
            input: String::new(),
            library_path,
            should_quit: false,
            status_message: None,
        })
    }

    fn filter_books(&mut self) {
        let input_lower = self.input.to_lowercase().replace(" ", "");
        self.filtered_books = self.all_books
            .iter()
            .filter(|book| {
                book.title.to_lowercase().contains(&input_lower)
                    || book.author.to_lowercase().contains(&input_lower)
                    || book.title_pinyin.contains(&input_lower)
                    || book.author_pinyin.contains(&input_lower)
            })
            .cloned()
            .collect();

        if !self.filtered_books.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
    }

    fn previous_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if self.filtered_books.is_empty() { 0 }
                else if i == 0 { self.filtered_books.len() - 1 }
                else { i - 1 }
            }
            None if !self.filtered_books.is_empty() => 0,
            _ => 0,
        };
        self.table_state.select(Some(i));
    }

    fn next_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if self.filtered_books.is_empty() { 0 }
                else if i >= self.filtered_books.len() - 1 { 0 }
                else { i + 1 }
            }
            None if !self.filtered_books.is_empty() => 0,
            _ => 0,
        };
        self.table_state.select(Some(i));
    }

    fn open_selected_book(&mut self) {
        if let Some(selected_index) = self.table_state.selected() {
            if let Some(book) = self.filtered_books.get(selected_index) {
                let full_path = self.library_path.join(&book.path);
                self.status_message = Some(format!("Attempting to open: {}", book.title));
                match open::that(&full_path) {
                    Ok(_) => self.status_message = Some(format!("Successfully sent request to open '{}'.", book.title)),
                    Err(e) => self.status_message = Some(format!("Error: {}", e)),
                }
            }
        } else {
            self.status_message = Some("No book selected.".to_string());
        }
    }
}

/// 从标准配置目录加载配置
/// 路径: ~/.config/calibre-tui/config.toml
fn load_config() -> Result<Config> {
    if let Some(mut config_path) = dirs::config_dir() {
        config_path.push("calibre-tui");

        // 如果目录 ~/.config/calibre-tui 不存在，则创建它
        if !config_path.exists() {
            fs::create_dir_all(&config_path)
                .with_context(|| format!("Failed to create config directory at {:?}", config_path))?;
        }

        config_path.push("config.toml");

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file '{}'", config_path.display()))?;
            return Ok(config);
        }
    }

    // 如果无法获取配置目录或文件不存在，返回默认空配置
    Ok(Config::default())
}

fn find_calibre_library_path(config: &Config) -> Option<PathBuf> {
    if let Some(config_path_str) = &config.library_path {
        if !config_path_str.is_empty() {
             let path = PathBuf::from(config_path_str);
            if path.join("metadata.db").exists() {
                return Some(path);
            }
        }
    }

    if let Some(home_dir) = dirs::home_dir() {
        let paths_to_check = [
            home_dir.join("Calibre Library"),
            home_dir.join("Calibre-Bibliothek"),
        ];
        for path in paths_to_check.iter() {
            if path.join("metadata.db").exists() {
                return Some(path.clone());
            }
        }
    }

    if let Some(docs_dir) = dirs::document_dir() {
        let docs_path = docs_dir.join("Calibre Library");
        if docs_path.join("metadata.db").exists() {
            return Some(docs_path);
        }
    }

    None
}

fn main() -> Result<()> {
    let config = load_config()?;
    let library_path = find_calibre_library_path(&config)
        .context("Could not find Calibre library. Please specify `library_path` in config.toml or ensure it's in a standard location.")?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(library_path)?;
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Application error: {:?}", err)
    }

    Ok(())
}

/// 主应用循环
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        if app.should_quit {
            return Ok(());
        }

        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                app.status_message = None;
                match key.code {
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        app.should_quit = true;
                    }
                    KeyCode::Esc => {
                        app.should_quit = true;
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                        app.filter_books();
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                        app.filter_books();
                    }
                    KeyCode::Down => app.next_item(),
                    KeyCode::Up => app.previous_item(),
                    KeyCode::Enter => app.open_selected_book(),
                    _ => {}
                }
            }
        }
    }
}


/// 创建一个带精确高亮匹配项的行 (支持普通文本和拼音)
fn create_highlighted_line<'a>(text: &'a str, query: &'a str) -> Line<'a> {
    if query.is_empty() {
        return Line::from(Span::raw(text));
    }

    let lower_query = query.to_lowercase();
    let text_chars: Vec<char> = text.chars().collect();

    // --- 1. 优先进行普通文本的子字符串匹配 ---
    let query_chars: Vec<char> = lower_query.chars().collect();
    if let Some(window_start_idx) = text_chars.windows(query_chars.len()).position(|window| {
        window.iter().map(|c| c.to_lowercase().to_string()).collect::<String>() == lower_query
    }) {
        let window_end_idx = window_start_idx + query_chars.len();
        let mut spans = Vec::new();

        spans.push(Span::raw(text_chars[0..window_start_idx].iter().collect::<String>()));
        spans.push(Span::styled(
            text_chars[window_start_idx..window_end_idx].iter().collect::<String>(),
            Style::default().fg(Color::Green).add_modifier(Modifier::UNDERLINED),
        ));
        spans.push(Span::raw(text_chars[window_end_idx..].iter().collect::<String>()));

        return Line::from(spans);
    }

    // --- 2. 如果上面不匹配，则进行拼音匹配 ---
    let lower_query_no_space = lower_query.replace(" ", "");
    if lower_query_no_space.chars().all(|c| c.is_ascii_alphabetic()) {
        let char_pinyin_map: Vec<(char, String)> = text.chars().zip(
            text.to_pinyin().map(|p_opt| {
                p_opt.map_or("".to_string(), |p| p.plain().to_string())
            })
        ).collect();

        let full_pinyin: String = char_pinyin_map.iter().map(|(_, p)| p.as_str()).collect();

        if let Some(match_start_idx) = full_pinyin.find(&lower_query_no_space) {
            let match_end_idx = match_start_idx + lower_query_no_space.len();

            let mut spans = Vec::new();
            let mut current_pinyin_len = 0;

            for (character, pinyin_str) in &char_pinyin_map {
                let pinyin_len = pinyin_str.len();
                let pinyin_range_start = current_pinyin_len;
                let pinyin_range_end = current_pinyin_len + pinyin_len;

                if pinyin_len > 0 && std::cmp::max(pinyin_range_start, match_start_idx) < std::cmp::min(pinyin_range_end, match_end_idx) {
                    spans.push(Span::styled(
                        character.to_string(),
                        Style::default().fg(Color::Green).add_modifier(Modifier::UNDERLINED),
                    ));
                } else {
                    spans.push(Span::raw(character.to_string()));
                }
                current_pinyin_len += pinyin_len;
            }
            return Line::from(spans);
        }
    }

    Line::from(Span::raw(text))
}


/// 绘制 UI 界面
fn ui(f: &mut Frame, app: &mut App) {
    let theme = Theme::terminal_default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(f.size());

    let input_paragraph = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(theme.input_fg))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Search (Enter to open, Ctrl+C/Esc to quit) ")
        );
    f.render_widget(input_paragraph, chunks[0]);

    f.set_cursor(
        chunks[0].x + app.input.width() as u16 + 1,
        chunks[0].y + 1,
    );

    let header_cells = ["Title", "Author"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(theme.header_fg).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .height(1)
        .bottom_margin(1);

    let rows = app.filtered_books.iter().map(|item| {
        let title_line = create_highlighted_line(&item.title, &app.input);
        let author_line = create_highlighted_line(&item.author, &app.input);
        Row::new(vec![
            Cell::from(title_line),
            Cell::from(author_line),
        ]).height(1).style(Style::default().fg(theme.fg))
    });

    let widths = [Constraint::Percentage(70), Constraint::Percentage(30)];
    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Book List (↑/↓) ")
        )
        .highlight_style(theme.selection_style);

    f.render_stateful_widget(table, chunks[1], &mut app.table_state);

    let status_text = app.status_message.as_deref().unwrap_or("");
    let status_bar = Paragraph::new(status_text).style(Style::default().fg(theme.status_fg));
    f.render_widget(status_bar, chunks[2]);
}

/// 从 Calibre 的 metadata.db 文件中加载书籍列表
fn load_books_from_db(library_path: &Path) -> Result<Vec<Book>> {
    let db_path = library_path.join("metadata.db");
    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open Calibre database: {:?}", db_path))?;

    let mut stmt = conn.prepare(
        "
        SELECT
            b.title,
            (SELECT name FROM authors WHERE id = (SELECT author FROM books_authors_link WHERE book = b.id LIMIT 1)) as author,
            b.path || '/' || (SELECT name FROM data WHERE book = b.id LIMIT 1) || '.' || lower((SELECT format FROM data WHERE book = b.id LIMIT 1))
        FROM
            books b
        ORDER BY
            b.sort;
        ",
    )?;

    let book_iter = stmt.query_map([], |row| {
        let title: String = row.get(0)?;
        let author: String = row.get(1).unwrap_or_else(|_| "Unknown Author".to_string());

        let mut title_pinyin = String::new();
        for p in title.as_str().to_pinyin() {
            if let Some(p) = p {
                title_pinyin.push_str(p.plain());
            }
        }

        let mut author_pinyin = String::new();
        for p in author.as_str().to_pinyin() {
            if let Some(p) = p {
                author_pinyin.push_str(p.plain());
            }
        }

        Ok(Book {
            path: row.get(2)?,
            title_pinyin,
            author_pinyin,
            title,
            author,
        })
    })?;

    let mut books = Vec::new();
    for book_result in book_iter {
        if let Ok(book) = book_result {
            books.push(book);
        }
    }

    Ok(books)
}
