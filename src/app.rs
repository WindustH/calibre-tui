use crate::config::{Config, OpenConfig};
use crate::filter::{BookSearch, SearchResult};
use crate::layout::{BookField, Layout};
use crate::sort::{SortSpec, sort_results};
use crate::theme::Theme;
use crate::ui;
use crate::utils::book::Book;
use crate::utils::db::load_books_from_db;
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
use framework_tui::{
  CommandCompletion, CommandState, KeyBindings, KeyContext, KeyDispatcher, KeyHelpEntry,
  MatchResult, Prompt, PromptInputResult, current_word_start, filter_completion_candidates,
  handle_prompt_key, handle_prompt_paste, key_event_to_token,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::TableState;
use std::collections::BTreeSet;
use std::io::{Stdout, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const COMMAND_NAMES: &[&str] = &["help", "sort"];
const SORT_FIELDS: &[&str] = &["title", "authors", "series", "formats", "tags"];
const SORT_DIRECTIONS: &[&str] = &["asc", "desc"];

pub struct App {
  books: Vec<Book>,
  search: BookSearch,
  keymap: KeyBindings,
  key_dispatcher: KeyDispatcher,
  open_config: OpenConfig,
  layout: Layout,
  theme: Theme,
  input: String,
  results: Vec<SearchResult>,
  table_state: TableState,
  selected_book_indices: BTreeSet<usize>,
  exit_on_open: bool,
  output_paths: Vec<PathBuf>,
  page_size: usize,
  sort_spec: SortSpec,
  prompt: Option<Prompt>,
  command_state: CommandState,
  key_help: bool,
  message: Option<String>,
}

enum EventAction {
  Continue,
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
    let books = load_books_from_db(&config.library_path).with_context(|| {
      format!(
        "failed to load books from '{}'",
        config.library_path.display()
      )
    })?;
    let search =
      BookSearch::new(&books, &config.filter, &layout).context("failed to build search index")?;

    let mut app = Self {
      books,
      search,
      keymap,
      key_dispatcher: KeyDispatcher::default(),
      open_config: config.open.clone(),
      layout,
      theme,
      input: String::new(),
      results: Vec::new(),
      table_state: TableState::default(),
      selected_book_indices: BTreeSet::new(),
      exit_on_open,
      output_paths: Vec::new(),
      page_size: 20,
      sort_spec: SortSpec::default(),
      prompt: None,
      command_state: CommandState::default(),
      key_help: false,
      message: None,
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
      self.page_size = usize::from(frame.area().height.saturating_sub(8)).max(1);
      let key_help_entries = self
        .key_help
        .then(|| self.key_help_entries())
        .unwrap_or_default();
      ui::draw(
        frame,
        frame.area(),
        ui::DrawState {
          input: &self.input,
          books: &self.books,
          results: &self.results,
          table_state: &mut self.table_state,
          selected_book_indices: &self.selected_book_indices,
          layout: &self.layout,
          theme: &self.theme,
          prompt: self.prompt.as_ref(),
          command_completion: self.command_state.completion(),
          key_hints: self.key_dispatcher.hints(),
          key_help_entries: self.key_help.then_some(key_help_entries.as_slice()),
          message: self.message.as_deref(),
          sort_label: &self.sort_spec.label(),
        },
      );
    })?;
    Ok(())
  }

  fn handle_event(&mut self, event: Event) -> Result<EventAction> {
    if self.key_help {
      self.handle_key_help_input(event);
      return Ok(EventAction::Continue);
    }

    if self.prompt.is_some() {
      return self.handle_prompt_event(event);
    }

    match event {
      Event::Key(key) => {
        if let Some(token) = key_event_to_token(key) {
          let had_pending_key_sequence = !self.key_dispatcher.pending().is_empty();
          if had_pending_key_sequence && token == "esc" {
            self.key_dispatcher.clear();
            return Ok(EventAction::Continue);
          }
          match self
            .key_dispatcher
            .dispatch(&self.keymap, KeyContext::Browser, token)
          {
            MatchResult::Action(action) => return self.handle_action(&action),
            MatchResult::Prefix(_) => return Ok(EventAction::Continue),
            MatchResult::None if had_pending_key_sequence => return Ok(EventAction::Continue),
            MatchResult::None => {}
          }
        }

        if is_search_input_key(&key)
          && let crossterm::event::KeyCode::Char(ch) = key.code
        {
          self.input.push(ch);
          self.refresh_results()?;
        }
      }
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

  fn handle_prompt_event(&mut self, event: Event) -> Result<EventAction> {
    match event {
      Event::Key(key) => {
        let result = {
          let Some(prompt) = self.prompt.as_mut() else {
            return Ok(EventAction::Continue);
          };
          handle_prompt_key(prompt, &mut self.command_state, &self.keymap, key)
        };

        match result {
          PromptInputResult::Changed => self.refresh_command_completion(),
          PromptInputResult::Cancel => self.close_prompt(),
          PromptInputResult::Submit => {
            let input = self
              .prompt
              .take()
              .map(|prompt| prompt.buffer().input.clone())
              .unwrap_or_default();
            self.submit_command(input)?;
          }
          PromptInputResult::UnknownAction(action) if action == "help" => {
            self.key_help = true;
          }
          PromptInputResult::EditInEditor { .. } => {
            self.set_message("$EDITOR command editing is not available here");
          }
          PromptInputResult::Unhandled | PromptInputResult::UnknownAction(_) => {}
        }
      }
      Event::Paste(value) => {
        if let Some(prompt) = self.prompt.as_mut() {
          handle_prompt_paste(prompt, &mut self.command_state, &value);
          self.refresh_command_completion();
        }
      }
      Event::Resize(_, _) => {}
      _ => {}
    }

    Ok(EventAction::Continue)
  }

  fn handle_action(&mut self, action: &str) -> Result<EventAction> {
    if action.starts_with("sort ") {
      self.execute_command(action)?;
      return Ok(EventAction::Continue);
    }

    match action {
      "quit" => return Ok(EventAction::Quit),
      "open" => {
        if self.open_selected()? {
          return Ok(EventAction::Quit);
        }
      }
      "print_paths" => {
        self.output_paths = self
          .target_book_indices()
          .into_iter()
          .map(|book_index| self.books[book_index].path.clone())
          .collect();
        return Ok(EventAction::Quit);
      }
      "copy_paths" => self.copy_paths_to_clipboard(),
      "move_up" => self.previous_item(),
      "move_down" => self.next_item(),
      "page_up" => self.page_up(),
      "page_down" => self.page_down(),
      "jump_start" => self.jump_start(),
      "jump_end" => self.jump_end(),
      "toggle_selection" => self.toggle_selected(),
      "select_all" => self.select_all_results(),
      "clear_selection" => self.clear_selection(),
      "delete_input" => {
        self.input.pop();
        self.refresh_results()?;
      }
      "command" => self.start_command(),
      "help" => self.key_help = true,
      other => self.set_message(format!("unknown action: {other}")),
    }

    Ok(EventAction::Continue)
  }

  fn refresh_results(&mut self) -> Result<()> {
    self.results = self.search.search(&self.input)?;
    self.sort_results(None);
    if self.results.is_empty() {
      self.table_state.select(None);
    } else {
      self.table_state.select(Some(0));
    }
    Ok(())
  }

  fn sort_results(&mut self, preserve_book_index: Option<usize>) {
    sort_results(
      &mut self.results,
      &self.books,
      &self.sort_spec,
      &self.layout,
    );
    if let Some(book_index) = preserve_book_index
      && let Some(row_index) = self
        .results
        .iter()
        .position(|result| result.book_index == book_index)
    {
      self.table_state.select(Some(row_index));
    }
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

  fn open_selected(&mut self) -> Result<bool> {
    let targets = self.target_book_indices();
    if targets.is_empty() {
      return Ok(false);
    }

    for book_index in targets {
      let book = &self.books[book_index];
      self.open_book(book)?;
    }

    self.selected_book_indices.clear();

    Ok(self.exit_on_open)
  }

  fn copy_paths_to_clipboard(&mut self) {
    let targets = self.target_book_indices();
    if targets.is_empty() {
      self.set_message("no book to copy");
      return;
    }

    let text = targets
      .iter()
      .map(|book_index| self.books[*book_index].path.display().to_string())
      .collect::<Vec<_>>()
      .join("\n");

    match copy_to_clipboard(&text) {
      Ok(()) => self.set_message(format!("copied {} path(s) to clipboard", targets.len())),
      Err(error) => self.set_message(format!("failed to copy paths: {error:#}")),
    }
  }

  fn open_book(&self, book: &Book) -> Result<()> {
    if let Some(command) = opener_command_for_path(&self.open_config, &book.path) {
      open_with_command(command, &book.path)
        .with_context(|| format!("failed to open '{}' ({})", book.title, book.path.display()))?;
      return Ok(());
    }

    open::that(&book.path)
      .with_context(|| format!("failed to open '{}' ({})", book.title, book.path.display()))?;
    Ok(())
  }

  fn start_command(&mut self) {
    self.command_state.reset_prompt_state();
    self.prompt = Some(Prompt::command(String::new()));
    self.refresh_command_completion();
  }

  fn close_prompt(&mut self) {
    self.prompt = None;
    self.command_state.reset_prompt_state();
  }

  fn submit_command(&mut self, input: String) -> Result<()> {
    let command = input.trim().trim_start_matches(':').trim().to_string();
    self.command_state.push_history(command.clone());
    self.execute_command(&command)
  }

  fn refresh_command_completion(&mut self) {
    let Some(prompt) = &self.prompt else {
      self.command_state.clear_completion();
      return;
    };
    if !prompt.is_command() {
      self.command_state.clear_completion();
      return;
    }

    let buffer = prompt.buffer();
    let completion = command_completion_for(&buffer.input, buffer.cursor);
    self
      .command_state
      .set_completion_preserving_selection(completion);
  }

  fn execute_command(&mut self, input: &str) -> Result<()> {
    let command = input.trim().trim_start_matches(':');
    let mut parts = command.split_whitespace();
    match parts.next() {
      Some("sort") => self.execute_sort_command(parts.collect()),
      Some("help") if parts.next().is_none() => {
        self.key_help = true;
        Ok(())
      }
      Some("") | None => {
        self.set_message("empty command");
        Ok(())
      }
      Some(other) => {
        self.set_message(format!("unknown command: {other}"));
        Ok(())
      }
    }
  }

  fn execute_sort_command(&mut self, args: Vec<&str>) -> Result<()> {
    match SortSpec::parse(&args) {
      Ok(sort_spec) => {
        let focused = self.current_book_index();
        self.sort_spec = sort_spec;
        self.sort_results(focused);
        self.set_message(format!("sort: {}", self.sort_spec.label()));
      }
      Err(error) => self.set_message(error.to_string()),
    }
    Ok(())
  }

  fn handle_key_help_input(&mut self, event: Event) {
    let Event::Key(key) = event else {
      return;
    };
    let Some(token) = key_event_to_token(key) else {
      return;
    };
    if matches!(token.as_str(), "esc" | "q" | "enter" | "f1") {
      self.key_help = false;
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

fn is_search_input_key(key: &KeyEvent) -> bool {
  key.kind == KeyEventKind::Press
    && matches!(key.code, crossterm::event::KeyCode::Char(_))
    && !key.modifiers.contains(KeyModifiers::CONTROL)
    && !key.modifiers.contains(KeyModifiers::ALT)
}

fn opener_command_for_path<'a>(open_config: &'a OpenConfig, path: &Path) -> Option<&'a [String]> {
  let extension = path
    .extension()
    .and_then(|extension| extension.to_str())
    .map(normalize_format_key)?;

  open_config.commands.iter().find_map(|(format, command)| {
    (normalize_format_key(format) == extension && !command.is_empty()).then_some(command.as_slice())
  })
}

fn normalize_format_key(format: &str) -> String {
  format.trim().trim_start_matches('.').to_ascii_lowercase()
}

fn copy_to_clipboard(text: &str) -> Result<()> {
  let candidates = clipboard_commands();
  let mut errors = Vec::new();

  for (program, args) in candidates {
    match run_clipboard_command(program, args, text) {
      Ok(()) => return Ok(()),
      Err(error) => errors.push(format!("{program}: {error:#}")),
    }
  }

  anyhow::bail!(
    "no clipboard command succeeded{}",
    if errors.is_empty() {
      String::new()
    } else {
      format!(" ({})", errors.join("; "))
    }
  )
}

fn clipboard_commands() -> Vec<(&'static str, Vec<&'static str>)> {
  #[cfg(target_os = "macos")]
  {
    return vec![("pbcopy", vec![])];
  }

  #[cfg(target_os = "windows")]
  {
    return vec![("clip", vec![])];
  }

  #[cfg(all(unix, not(target_os = "macos")))]
  {
    let mut commands = Vec::new();
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
      commands.push(("wl-copy", vec![]));
    }
    if std::env::var_os("DISPLAY").is_some() {
      commands.push(("xclip", vec!["-selection", "clipboard"]));
      commands.push(("xsel", vec!["--clipboard", "--input"]));
    }
    commands.push(("wl-copy", vec![]));
    commands.push(("xclip", vec!["-selection", "clipboard"]));
    commands.push(("xsel", vec!["--clipboard", "--input"]));
    commands
  }
}

fn run_clipboard_command(program: &str, args: Vec<&str>, text: &str) -> Result<()> {
  let mut child = Command::new(program)
    .args(args)
    .stdin(Stdio::piped())
    .spawn()
    .with_context(|| "failed to start command")?;

  let Some(mut stdin) = child.stdin.take() else {
    anyhow::bail!("failed to open command stdin");
  };
  stdin
    .write_all(text.as_bytes())
    .with_context(|| "failed to write clipboard data")?;
  drop(stdin);

  let status = child
    .wait()
    .with_context(|| "failed to wait for clipboard command")?;
  if !status.success() {
    anyhow::bail!("command exited with {status}");
  }

  Ok(())
}

fn open_with_command(command: &[String], path: &Path) -> Result<()> {
  let Some((program, args)) = command
    .split_first()
    .filter(|(program, _)| !program.is_empty())
  else {
    open::that(path).with_context(|| format!("failed to open '{}'", path.display()))?;
    return Ok(());
  };

  let path_text = path.to_string_lossy();
  let mut includes_path = false;
  let mut child = Command::new(program);

  for arg in args {
    if arg.contains("{path}") {
      includes_path = true;
      child.arg(arg.replace("{path}", &path_text));
    } else {
      child.arg(arg);
    }
  }

  if !includes_path {
    child.arg(path);
  }

  child
    .spawn()
    .with_context(|| format!("failed to run open command '{}'", command.join(" ")))?;
  Ok(())
}

fn command_completion_for(input: &str, cursor: usize) -> Option<CommandCompletion> {
  let cursor = cursor.min(input.len());
  let before_cursor = input.get(..cursor)?;
  let normalized = before_cursor.trim_start_matches(':');
  let tokens = normalized.split_whitespace().collect::<Vec<_>>();
  let ends_with_space = normalized.chars().last().is_some_and(char::is_whitespace);
  let word_start = current_word_start(input, cursor);
  let prefix = if ends_with_space {
    ""
  } else {
    input.get(word_start..cursor).unwrap_or_default()
  };

  if tokens.is_empty() || (tokens.len() == 1 && !ends_with_space) {
    return completion_from_candidates(
      word_start,
      cursor,
      prefix,
      filter_completion_candidates(COMMAND_NAMES.iter().copied(), prefix),
      ends_with_space,
      true,
    );
  }

  match tokens[0] {
    "sort" => sort_command_completion(&tokens[1..], ends_with_space, word_start, cursor, prefix),
    "help" => None,
    _ => None,
  }
}

fn sort_command_completion(
  args: &[&str],
  ends_with_space: bool,
  word_start: usize,
  cursor: usize,
  prefix: &str,
) -> Option<CommandCompletion> {
  let completed_args = if ends_with_space {
    args
  } else {
    args.get(..args.len().saturating_sub(1))?
  };
  let candidates = sort_completion_candidates(completed_args)?;
  let replace_start = if ends_with_space { cursor } else { word_start };
  let prefix = if ends_with_space { "" } else { prefix };

  completion_from_candidates(
    replace_start,
    cursor,
    prefix,
    filter_completion_candidates(candidates, prefix),
    ends_with_space,
    true,
  )
}

fn completion_from_candidates(
  replace_start: usize,
  replace_end: usize,
  prefix: &str,
  candidates: Vec<String>,
  ends_with_space: bool,
  append_space: bool,
) -> Option<CommandCompletion> {
  if !ends_with_space && candidates.len() == 1 && candidates[0].eq_ignore_ascii_case(prefix) {
    return None;
  }

  Some(CommandCompletion::new(
    replace_start,
    replace_end,
    prefix,
    candidates,
    append_space,
    0,
  ))
}

fn sort_completion_candidates(completed_args: &[&str]) -> Option<Vec<&'static str>> {
  let mut expecting_field = true;
  for arg in completed_args {
    if expecting_field {
      BookField::parse(arg)?;
      expecting_field = false;
      continue;
    }

    if is_sort_direction(arg) {
      expecting_field = true;
    } else if BookField::parse(arg).is_some() {
      expecting_field = false;
    } else {
      return None;
    }
  }

  if expecting_field {
    Some(SORT_FIELDS.to_vec())
  } else {
    let mut candidates = SORT_DIRECTIONS.to_vec();
    candidates.extend(SORT_FIELDS);
    Some(candidates)
  }
}

fn is_sort_direction(input: &str) -> bool {
  matches!(
    input.to_ascii_lowercase().as_str(),
    "asc" | "ascending" | "desc" | "descending"
  )
}
