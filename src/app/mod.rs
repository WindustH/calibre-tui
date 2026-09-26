//! Application state and the event loop.

mod browser;
mod command;
mod launch;

use crate::config::{Config, OpenConfig};
use crate::layout::Layout;
use crate::library::{self, Book};
use crate::search::BookSearch;
use crate::sort::{BookOrder, SortSpec};
use crate::terminal::Tui;
use crate::theme::Theme;
use crate::ui;
use anyhow::{Context, Result};
use browser::Browser;
use command::Command;
use crossterm::event::{
  self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};
use framework_tui::{
  CommandState, KeyBindings, KeyContext, KeyDispatcher, KeyHelpEntry, MatchResult, Prompt,
  PromptInputResult, handle_prompt_key, handle_prompt_paste, key_event_to_token,
};
use std::mem;
use std::path::PathBuf;
use std::time::Duration;

pub struct App {
  books: Vec<Book>,
  search: BookSearch,
  order: BookOrder,
  browser: Browser,
  /// The input changed since the last search; results are refreshed lazily
  /// so a burst of typed or pasted chars costs one search.
  search_stale: bool,
  keymap: KeyBindings,
  key_dispatcher: KeyDispatcher,
  open_config: OpenConfig,
  layout: Layout,
  theme: Theme,
  prompt: Option<Prompt>,
  command_state: CommandState,
  key_help: bool,
  message: Option<String>,
  exit_on_open: bool,
  output_paths: Vec<PathBuf>,
}

/// What the event loop does after handling an event.
enum Flow {
  Idle,
  Redraw,
  Quit,
}

impl App {
  pub fn new(
    config: Config,
    keymap: KeyBindings,
    layout: Layout,
    theme: Theme,
    exit_on_open: bool,
  ) -> Result<Self> {
    let books = library::load_books(&config.library_path).with_context(|| {
      format!(
        "failed to load books from '{}'",
        config.library_path.display()
      )
    })?;
    let search = BookSearch::new(&books, &config.filter, layout.search_fields().collect());
    let order = BookOrder::new(SortSpec::default(), &books);

    let mut app = Self {
      books,
      search,
      order,
      browser: Browser::new(),
      search_stale: false,
      keymap,
      key_dispatcher: KeyDispatcher::default(),
      open_config: config.open,
      layout,
      theme,
      prompt: None,
      command_state: CommandState::default(),
      key_help: false,
      message: None,
      exit_on_open,
      output_paths: Vec::new(),
    };
    app.refresh_results();
    Ok(app)
  }

  /// Run until the user quits; returns the paths to print (`print_paths`).
  pub fn run(&mut self, terminal: &mut Tui) -> Result<Vec<PathBuf>> {
    self.draw(terminal)?;
    loop {
      // Block for the next event, then drain whatever is already queued
      // (key repeat, pastes, mouse motion) before drawing once.
      let mut redraw = false;
      let mut event = event::read()?;
      loop {
        match self.handle_event(event) {
          Flow::Quit => return Ok(mem::take(&mut self.output_paths)),
          Flow::Redraw => redraw = true,
          Flow::Idle => {}
        }
        if !event::poll(Duration::ZERO)? {
          break;
        }
        event = event::read()?;
      }
      if redraw {
        self.draw(terminal)?;
      }
    }
  }

  fn draw(&mut self, terminal: &mut Tui) -> Result<()> {
    self.flush_search();
    let key_help = self.key_help.then(|| self.key_help_entries());
    let sort_label = self.order.spec().label();
    let mut viewport = None;
    terminal.draw(|frame| {
      viewport = Some(ui::draw(
        frame,
        &ui::View {
          theme: &self.theme,
          layout: &self.layout,
          input: self.browser.input(),
          table: ui::BookTable {
            books: &self.books,
            results: self.browser.results(),
            marked: self.browser.marked(),
            focus: self.browser.focus(),
            scroll: self.browser.scroll(),
          },
          sort_label: &sort_label,
          prompt: self.prompt.as_ref(),
          completion: self.command_state.completion(),
          key_hints: self.key_dispatcher.hints(),
          key_help: key_help.as_deref(),
          message: self.message.as_deref(),
        },
      ));
    })?;
    if let Some(viewport) = viewport {
      self.browser.set_viewport(viewport);
    }
    Ok(())
  }

  fn handle_event(&mut self, event: Event) -> Flow {
    let browsing = !self.key_help && self.prompt.is_none();
    match event {
      Event::Key(key) if key.kind == KeyEventKind::Press => {
        if self.key_help {
          self.handle_key_help_key(key)
        } else if self.prompt.is_some() {
          self.handle_prompt_key(key)
        } else {
          self.handle_browser_key(key)
        }
      }
      Event::Paste(text) => match self.prompt.as_mut() {
        Some(prompt) if !self.key_help => {
          handle_prompt_paste(prompt, &mut self.command_state, &text);
          self.refresh_completion();
          Flow::Redraw
        }
        _ => Flow::Idle,
      },
      Event::Mouse(mouse) if browsing => match mouse.kind {
        MouseEventKind::ScrollDown => self.run_action("move_down"),
        MouseEventKind::ScrollUp => self.run_action("move_up"),
        _ => Flow::Idle,
      },
      Event::Resize(..) => Flow::Redraw,
      _ => Flow::Idle,
    }
  }

  fn handle_browser_key(&mut self, key: KeyEvent) -> Flow {
    if let Some(token) = key_event_to_token(key) {
      let had_pending_sequence = !self.key_dispatcher.pending().is_empty();
      if had_pending_sequence && token == "esc" {
        self.key_dispatcher.clear();
        return Flow::Redraw;
      }
      match self
        .key_dispatcher
        .dispatch(&self.keymap, KeyContext::Browser, token)
      {
        MatchResult::Action(action) => return self.run_action(&action),
        MatchResult::Prefix(_) => return Flow::Redraw,
        MatchResult::None if had_pending_sequence => return Flow::Redraw,
        MatchResult::None => {}
      }
    }

    match search_input_char(&key) {
      Some(ch) => {
        self.browser.push_input(ch);
        self.search_stale = true;
        Flow::Redraw
      }
      None => Flow::Idle,
    }
  }

  fn handle_prompt_key(&mut self, key: KeyEvent) -> Flow {
    let Some(prompt) = self.prompt.as_mut() else {
      return Flow::Idle;
    };
    match handle_prompt_key(prompt, &mut self.command_state, &self.keymap, key) {
      PromptInputResult::Changed => self.refresh_completion(),
      PromptInputResult::Cancel => self.close_prompt(),
      PromptInputResult::Submit => {
        let input = self
          .prompt
          .take()
          .map(|prompt| prompt.buffer().input.clone())
          .unwrap_or_default();
        self.submit_command(&input);
      }
      PromptInputResult::UnknownAction(action) if action == "help" => self.key_help = true,
      PromptInputResult::EditInEditor { .. } => {
        self.set_message("$EDITOR command editing is not available here");
      }
      PromptInputResult::Unhandled | PromptInputResult::UnknownAction(_) => return Flow::Idle,
    }
    Flow::Redraw
  }

  fn handle_key_help_key(&mut self, key: KeyEvent) -> Flow {
    match key_event_to_token(key) {
      Some(token) if matches!(token.as_str(), "esc" | "q" | "enter" | "f1") => {
        self.key_help = false;
        Flow::Redraw
      }
      _ => Flow::Idle,
    }
  }

  /// Run a browser keymap action.
  fn run_action(&mut self, action: &str) -> Flow {
    self.flush_search();
    match action {
      "quit" => return Flow::Quit,
      "open" => return self.open_targets(),
      "print_paths" => {
        self.output_paths = self.target_paths();
        return Flow::Quit;
      }
      "copy_paths" => self.copy_paths(),
      "move_up" => self.browser.move_up(),
      "move_down" => self.browser.move_down(),
      "page_up" => self.browser.page_up(),
      "page_down" => self.browser.page_down(),
      "jump_start" => self.browser.jump_start(),
      "jump_end" => self.browser.jump_end(),
      "toggle_selection" => self.browser.toggle_mark(),
      "select_all" => self.browser.mark_all(),
      "clear_selection" => self.browser.clear_marks(),
      "delete_input" => {
        if self.browser.pop_input() {
          self.search_stale = true;
        }
      }
      "command" => self.open_prompt(),
      "help" => self.key_help = true,
      command if command.starts_with("sort ") => self.run_command(command),
      other => self.set_message(format!("unknown action: {other}")),
    }
    Flow::Redraw
  }

  fn flush_search(&mut self) {
    if mem::take(&mut self.search_stale) {
      self.refresh_results();
    }
  }

  fn refresh_results(&mut self) {
    let mut results = self.search.search(self.browser.input());
    self.order.sort(&mut results);
    self.browser.set_results(results);
  }

  fn open_targets(&mut self) -> Flow {
    let targets = self.browser.target_books();
    if targets.is_empty() {
      return Flow::Redraw;
    }

    let errors = targets
      .iter()
      .filter_map(|&book| launch::open_book(&self.open_config, &self.books[book]).err())
      .collect::<Vec<_>>();
    self.browser.clear_marks();

    match errors.as_slice() {
      [] if self.exit_on_open => return Flow::Quit,
      [] => {}
      [error] => self.set_message(format!("{error:#}")),
      [error, rest @ ..] => self.set_message(format!("{error:#} (and {} more)", rest.len())),
    }
    Flow::Redraw
  }

  /// File paths of the target books; metadata-only books are skipped.
  fn target_paths(&self) -> Vec<PathBuf> {
    self
      .browser
      .target_books()
      .into_iter()
      .filter_map(|book| self.books[book].path.clone())
      .collect()
  }

  fn copy_paths(&mut self) {
    let paths = self.target_paths();
    if paths.is_empty() {
      self.set_message("no book file to copy");
      return;
    }

    let text = paths
      .iter()
      .map(|path| path.display().to_string())
      .collect::<Vec<_>>()
      .join("\n");
    match launch::copy_to_clipboard(&text) {
      Ok(()) => self.set_message(format!("copied {} path(s) to clipboard", paths.len())),
      Err(error) => self.set_message(format!("failed to copy paths: {error:#}")),
    }
  }

  fn open_prompt(&mut self) {
    self.command_state.reset_prompt_state();
    self.prompt = Some(Prompt::command(String::new()));
    self.refresh_completion();
  }

  fn close_prompt(&mut self) {
    self.prompt = None;
    self.command_state.reset_prompt_state();
  }

  fn refresh_completion(&mut self) {
    let completion = self
      .prompt
      .as_ref()
      .filter(|prompt| prompt.is_command())
      .and_then(|prompt| command::completion(&prompt.buffer().input, prompt.buffer().cursor));
    self
      .command_state
      .set_completion_preserving_selection(completion);
  }

  fn submit_command(&mut self, input: &str) {
    let command = input.trim().trim_start_matches(':').trim();
    self.command_state.push_history(command);
    self.run_command(command);
  }

  fn run_command(&mut self, input: &str) {
    match command::parse(input) {
      Ok(Command::Sort(spec)) => {
        self.order = BookOrder::new(spec, &self.books);
        let order = &self.order;
        self.browser.reorder(|results| order.sort(results));
        self.set_message(format!("sort: {}", self.order.spec().label()));
      }
      Ok(Command::Help) => self.key_help = true,
      Err(message) => self.set_message(message),
    }
  }

  fn key_help_entries(&self) -> Vec<KeyHelpEntry> {
    let context = if self.prompt.is_some() {
      KeyContext::Input
    } else {
      KeyContext::Browser
    };
    self.keymap.help_entries(context)
  }

  fn set_message(&mut self, message: impl Into<String>) {
    self.message = Some(message.into());
  }
}

/// The char a key press adds to the search input, if any. Windows reports
/// AltGr as Ctrl+Alt, so that combination still types its char there.
fn search_input_char(key: &KeyEvent) -> Option<char> {
  let KeyCode::Char(ch) = key.code else {
    return None;
  };
  let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
  let alt = key.modifiers.contains(KeyModifiers::ALT);
  let alt_gr = cfg!(windows) && ctrl && alt;
  ((!ctrl && !alt) || alt_gr).then_some(ch)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
  }

  #[test]
  fn search_input_accepts_plain_and_shifted_chars_only() {
    assert_eq!(
      search_input_char(&key(KeyCode::Char('a'), KeyModifiers::NONE)),
      Some('a')
    );
    assert_eq!(
      search_input_char(&key(KeyCode::Char('A'), KeyModifiers::SHIFT)),
      Some('A')
    );
    assert_eq!(
      search_input_char(&key(KeyCode::Char('a'), KeyModifiers::CONTROL)),
      None
    );
    assert_eq!(
      search_input_char(&key(KeyCode::Char('a'), KeyModifiers::ALT)),
      None
    );
    assert_eq!(
      search_input_char(&key(
        KeyCode::Char('@'),
        KeyModifiers::CONTROL | KeyModifiers::ALT
      )),
      cfg!(windows).then_some('@')
    );
    assert_eq!(
      search_input_char(&key(KeyCode::Enter, KeyModifiers::NONE)),
      None
    );
  }
}
