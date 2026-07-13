use crate::config::Config;
use crate::filter::{BookSearch, SearchResult};
use crate::keymap::{Action, Keymap, KeymapMatch, is_text_input_key};
use crate::ui;
use crate::utils::book::Book;
use crate::utils::db::load_books_from_db;
use anyhow::{Context, Result};
use crossterm::event::{self, Event, MouseEventKind};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::TableState;
use std::collections::BTreeSet;
use std::io::Stdout;
use std::path::PathBuf;
use std::time::Duration;

pub struct App {
  books: Vec<Book>,
  search: BookSearch,
  keymap: Keymap,
  input: String,
  results: Vec<SearchResult>,
  table_state: TableState,
  selected_book_indices: BTreeSet<usize>,
  exit_on_submit: bool,
  print_path: bool,
  output_paths: Vec<PathBuf>,
  page_size: usize,
}

enum EventAction {
  Continue,
  Quit,
}

impl App {
  pub fn new(
    config: Config,
    keymap: Keymap,
    exit_on_submit: bool,
    print_path: bool,
  ) -> Result<Self> {
    let books = load_books_from_db(&config.library_path).with_context(|| {
      format!(
        "failed to load books from '{}'",
        config.library_path.display()
      )
    })?;
    let search = BookSearch::new(&books, &config.filter).context("failed to build search index")?;

    let mut app = Self {
      books,
      search,
      keymap,
      input: String::new(),
      results: Vec::new(),
      table_state: TableState::default(),
      selected_book_indices: BTreeSet::new(),
      exit_on_submit,
      print_path,
      output_paths: Vec::new(),
      page_size: 20,
    };
    app.refresh_results()?;
    Ok(app)
  }

  pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<Vec<PathBuf>> {
    self.draw(terminal)?;

    loop {
      let mut should_draw = false;

      if event::poll(Duration::from_millis(250))? {
        let action = self.handle_event(event::read()?)?;
        if matches!(action, EventAction::Quit) {
          return Ok(std::mem::take(&mut self.output_paths));
        }
        should_draw = true;

        while event::poll(Duration::from_millis(0))? {
          let action = self.handle_event(event::read()?)?;
          if matches!(action, EventAction::Quit) {
            return Ok(std::mem::take(&mut self.output_paths));
          }
          should_draw = true;
        }
      }

      if should_draw {
        self.draw(terminal)?;
      }
    }
  }

  fn draw(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal.draw(|frame| {
      self.page_size = usize::from(frame.area().height.saturating_sub(7)).max(1);
      ui::draw(
        frame,
        frame.area(),
        ui::DrawState {
          input: &self.input,
          books: &self.books,
          results: &self.results,
          table_state: &mut self.table_state,
          selected_book_indices: &self.selected_book_indices,
          print_path: self.print_path,
        },
      );
    })?;
    Ok(())
  }

  fn handle_event(&mut self, event: Event) -> Result<EventAction> {
    match event {
      Event::Key(key) => match self.keymap.match_key(&key) {
        KeymapMatch::Action(action) => match action {
          Action::Quit => return Ok(EventAction::Quit),
          Action::Submit => {
            if self.submit_selected()? {
              return Ok(EventAction::Quit);
            }
          }
          Action::MoveUp => self.previous_item(),
          Action::MoveDown => self.next_item(),
          Action::PageUp => self.page_up(),
          Action::PageDown => self.page_down(),
          Action::JumpStart => self.jump_start(),
          Action::JumpEnd => self.jump_end(),
          Action::ToggleSelection => self.toggle_selected(),
          Action::SelectAll => self.select_all_results(),
          Action::ClearSelection => self.clear_selection(),
          Action::DeleteInput => {
            self.input.pop();
            self.refresh_results()?;
          }
        },
        KeymapMatch::Pending => {}
        KeymapMatch::None => {
          if is_text_input_key(&key)
            && let crossterm::event::KeyCode::Char(ch) = key.code
          {
            self.input.push(ch);
            self.refresh_results()?;
          }
        }
      },
      Event::Mouse(mouse) => match mouse.kind {
        MouseEventKind::ScrollDown => self.next_item(),
        MouseEventKind::ScrollUp => self.previous_item(),
        _ => {}
      },
      Event::Resize(_, _) => {}
      _ => {}
    }

    Ok(EventAction::Continue)
  }

  fn refresh_results(&mut self) -> Result<()> {
    self.results = self.search.search(&self.input)?;
    if self.results.is_empty() {
      self.table_state.select(None);
    } else {
      self.table_state.select(Some(0));
    }
    Ok(())
  }

  fn previous_item(&mut self) {
    if self.results.is_empty() {
      self.table_state.select(None);
      return;
    }

    let index = match self.table_state.selected() {
      Some(0) | None => self.results.len() - 1,
      Some(index) => index.saturating_sub(1),
    };
    self.table_state.select(Some(index));
  }

  fn next_item(&mut self) {
    if self.results.is_empty() {
      self.table_state.select(None);
      return;
    }

    let index = match self.table_state.selected() {
      Some(index) if index + 1 < self.results.len() => index + 1,
      _ => 0,
    };
    self.table_state.select(Some(index));
  }

  fn page_up(&mut self) {
    if self.results.is_empty() {
      self.table_state.select(None);
      return;
    }

    let index = self
      .table_state
      .selected()
      .unwrap_or(0)
      .saturating_sub(self.page_size);
    self.table_state.select(Some(index));
  }

  fn page_down(&mut self) {
    if self.results.is_empty() {
      self.table_state.select(None);
      return;
    }

    let last = self.results.len() - 1;
    let index = self
      .table_state
      .selected()
      .unwrap_or(0)
      .saturating_add(self.page_size)
      .min(last);
    self.table_state.select(Some(index));
  }

  fn jump_start(&mut self) {
    if self.results.is_empty() {
      self.table_state.select(None);
    } else {
      self.table_state.select(Some(0));
    }
  }

  fn jump_end(&mut self) {
    if self.results.is_empty() {
      self.table_state.select(None);
    } else {
      self.table_state.select(Some(self.results.len() - 1));
    }
  }

  fn toggle_selected(&mut self) {
    let Some(book_index) = self.current_book_index() else {
      return;
    };

    if !self.selected_book_indices.insert(book_index) {
      self.selected_book_indices.remove(&book_index);
    }

    self.next_item();
  }

  fn select_all_results(&mut self) {
    self
      .selected_book_indices
      .extend(self.results.iter().map(|result| result.book_index));
  }

  fn clear_selection(&mut self) {
    self.selected_book_indices.clear();
  }

  fn current_book_index(&self) -> Option<usize> {
    let selected = self.table_state.selected()?;
    let result = self.results.get(selected)?;
    Some(result.book_index)
  }

  fn target_book_indices(&self) -> Vec<usize> {
    if self.selected_book_indices.is_empty() {
      return self.current_book_index().into_iter().collect();
    }

    self
      .selected_book_indices
      .iter()
      .copied()
      .filter(|book_index| *book_index < self.books.len())
      .collect()
  }

  fn submit_selected(&mut self) -> Result<bool> {
    let targets = self.target_book_indices();
    if targets.is_empty() {
      return Ok(false);
    }

    if self.print_path {
      self.output_paths = targets
        .into_iter()
        .map(|book_index| self.books[book_index].path.clone())
        .collect();
      return Ok(true);
    }

    for book_index in targets {
      let book = &self.books[book_index];
      open::that(&book.path)
        .with_context(|| format!("failed to open '{}' ({})", book.title, book.path.display()))?;
    }

    self.selected_book_indices.clear();

    Ok(self.exit_on_submit)
  }
}
