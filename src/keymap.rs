use anyhow::{Context, Result, bail};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone)]
pub struct Keymap {
  bindings: Vec<ActionBinding>,
  pending: Vec<KeyBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct KeymapConfig {
  quit: Vec<String>,
  submit: Vec<String>,
  move_up: Vec<String>,
  move_down: Vec<String>,
  page_up: Vec<String>,
  page_down: Vec<String>,
  jump_start: Vec<String>,
  jump_end: Vec<String>,
  toggle_selection: Vec<String>,
  select_all: Vec<String>,
  clear_selection: Vec<String>,
  delete_input: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
  Quit,
  Submit,
  MoveUp,
  MoveDown,
  PageUp,
  PageDown,
  JumpStart,
  JumpEnd,
  ToggleSelection,
  SelectAll,
  ClearSelection,
  DeleteInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActionBinding {
  action: Action,
  sequence: Vec<KeyBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeymapMatch {
  Action(Action),
  Pending,
  None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyBinding {
  code: BindingCode,
  modifiers: KeyModifiers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BindingCode {
  Char(char),
  Backspace,
  Enter,
  Esc,
  Tab,
  BackTab,
  Up,
  Down,
  Left,
  Right,
  Home,
  End,
  PageUp,
  PageDown,
  Delete,
  Insert,
}

impl Default for KeymapConfig {
  fn default() -> Self {
    Self {
      quit: vec!["esc".to_string(), "ctrl-c".to_string()],
      submit: vec!["enter".to_string()],
      move_up: vec!["up".to_string()],
      move_down: vec!["down".to_string()],
      page_up: vec!["pgup".to_string()],
      page_down: vec!["pgdown".to_string()],
      jump_start: vec!["home".to_string()],
      jump_end: vec!["end".to_string()],
      toggle_selection: vec!["tab".to_string()],
      select_all: vec!["ctrl-a".to_string()],
      clear_selection: vec!["ctrl-x".to_string()],
      delete_input: vec!["backspace".to_string()],
    }
  }
}

impl Keymap {
  pub fn match_key(&mut self, key: &KeyEvent) -> KeymapMatch {
    let pressed = KeyBinding::from_key_event(key);

    if !self.pending.is_empty() {
      let mut candidate = self.pending.clone();
      candidate.push(pressed.clone());

      match self.match_sequence(&candidate) {
        KeymapMatch::Action(action) => {
          self.pending.clear();
          return KeymapMatch::Action(action);
        }
        KeymapMatch::Pending => {
          self.pending = candidate;
          return KeymapMatch::Pending;
        }
        KeymapMatch::None => self.pending.clear(),
      }
    }

    match self.match_sequence(std::slice::from_ref(&pressed)) {
      KeymapMatch::Pending => {
        self.pending = vec![pressed];
        KeymapMatch::Pending
      }
      result => result,
    }
  }

  fn match_sequence(&self, sequence: &[KeyBinding]) -> KeymapMatch {
    let mut has_prefix = false;

    for binding in &self.bindings {
      if binding.sequence == sequence {
        return KeymapMatch::Action(binding.action);
      }

      if binding.sequence.len() > sequence.len() && binding.sequence.starts_with(sequence) {
        has_prefix = true;
      }
    }

    if has_prefix {
      KeymapMatch::Pending
    } else {
      KeymapMatch::None
    }
  }
}

pub fn load_keymap() -> Result<Keymap> {
  let config_dir = dirs::config_dir()
    .context("could not get config directory")?
    .join("calibre-tui");
  fs::create_dir_all(&config_dir)
    .with_context(|| format!("failed to create config directory at {:?}", config_dir))?;

  let keymap_path = config_dir.join("keymap.toml");
  if !keymap_path.exists() {
    let default_keymap = toml::to_string_pretty(&KeymapConfig::default())?;
    fs::write(&keymap_path, default_keymap)
      .with_context(|| format!("failed to write keymap file to {:?}", keymap_path))?;
  }

  let content = fs::read_to_string(&keymap_path)
    .with_context(|| format!("failed to read keymap file: {:?}", keymap_path))?;
  let config: KeymapConfig = toml::from_str(&content)
    .with_context(|| format!("failed to parse keymap file '{}'", keymap_path.display()))?;

  config.compile()
}

impl KeymapConfig {
  fn compile(self) -> Result<Keymap> {
    let mut bindings = Vec::new();
    append_bindings(&mut bindings, Action::Quit, "quit", &self.quit)?;
    append_bindings(&mut bindings, Action::Submit, "submit", &self.submit)?;
    append_bindings(&mut bindings, Action::MoveUp, "move_up", &self.move_up)?;
    append_bindings(
      &mut bindings,
      Action::MoveDown,
      "move_down",
      &self.move_down,
    )?;
    append_bindings(&mut bindings, Action::PageUp, "page_up", &self.page_up)?;
    append_bindings(
      &mut bindings,
      Action::PageDown,
      "page_down",
      &self.page_down,
    )?;
    append_bindings(
      &mut bindings,
      Action::JumpStart,
      "jump_start",
      &self.jump_start,
    )?;
    append_bindings(&mut bindings, Action::JumpEnd, "jump_end", &self.jump_end)?;
    append_bindings(
      &mut bindings,
      Action::ToggleSelection,
      "toggle_selection",
      &self.toggle_selection,
    )?;
    append_bindings(
      &mut bindings,
      Action::SelectAll,
      "select_all",
      &self.select_all,
    )?;
    append_bindings(
      &mut bindings,
      Action::ClearSelection,
      "clear_selection",
      &self.clear_selection,
    )?;
    append_bindings(
      &mut bindings,
      Action::DeleteInput,
      "delete_input",
      &self.delete_input,
    )?;

    Ok(Keymap {
      bindings,
      pending: Vec::new(),
    })
  }
}

impl KeyBinding {
  fn parse(input: &str) -> Result<Self> {
    let normalized = input.trim().replace('+', "-");
    if normalized.is_empty() {
      bail!("empty key binding");
    }

    let mut modifiers = KeyModifiers::empty();
    let mut code_parts = Vec::new();
    for token in normalized.split('-').filter(|token| !token.is_empty()) {
      match token.to_lowercase().as_str() {
        "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
        "alt" => modifiers |= KeyModifiers::ALT,
        "shift" => modifiers |= KeyModifiers::SHIFT,
        _ => code_parts.push(token),
      }
    }

    if code_parts.is_empty() {
      bail!("invalid key binding '{}': missing key code", input);
    }

    let code_token = code_parts.join("-");
    let code = BindingCode::parse(&code_token, &mut modifiers)?;
    if matches!(code, BindingCode::BackTab) {
      modifiers.remove(KeyModifiers::SHIFT);
    }

    Ok(Self { code, modifiers })
  }

  fn from_key_event(key: &KeyEvent) -> Self {
    let mut modifiers = relevant_modifiers(key.modifiers);
    let code = BindingCode::from_key_code(key.code, &mut modifiers);
    Self { code, modifiers }
  }
}

impl BindingCode {
  fn parse(input: &str, modifiers: &mut KeyModifiers) -> Result<Self> {
    let lower = input.to_lowercase();
    let code = match lower.as_str() {
      "backspace" | "bs" => Self::Backspace,
      "enter" | "return" => Self::Enter,
      "esc" | "escape" => Self::Esc,
      "tab" if modifiers.contains(KeyModifiers::SHIFT) => Self::BackTab,
      "tab" => Self::Tab,
      "backtab" | "shift-tab" => Self::BackTab,
      "up" => Self::Up,
      "down" => Self::Down,
      "left" => Self::Left,
      "right" => Self::Right,
      "home" => Self::Home,
      "end" => Self::End,
      "pgup" | "pageup" | "page-up" => Self::PageUp,
      "pgdown" | "pgdn" | "pagedown" | "page-down" => Self::PageDown,
      "delete" | "del" => Self::Delete,
      "insert" | "ins" => Self::Insert,
      "space" => Self::Char(' '),
      _ => {
        let mut chars = input.chars();
        let Some(ch) = chars.next() else {
          bail!("empty key code");
        };
        if chars.next().is_some() {
          bail!("unsupported key code '{}'", input);
        }
        if ch.is_uppercase() {
          *modifiers |= KeyModifiers::SHIFT;
        }
        Self::Char(ch.to_ascii_lowercase())
      }
    };

    Ok(code)
  }

  fn from_key_code(code: KeyCode, modifiers: &mut KeyModifiers) -> Self {
    match code {
      KeyCode::Char(ch) => {
        if ch.is_uppercase() {
          *modifiers |= KeyModifiers::SHIFT;
        }
        Self::Char(ch.to_ascii_lowercase())
      }
      KeyCode::Backspace => Self::Backspace,
      KeyCode::Enter => Self::Enter,
      KeyCode::Esc => Self::Esc,
      KeyCode::Tab => Self::Tab,
      KeyCode::BackTab => {
        modifiers.remove(KeyModifiers::SHIFT);
        Self::BackTab
      }
      KeyCode::Up => Self::Up,
      KeyCode::Down => Self::Down,
      KeyCode::Left => Self::Left,
      KeyCode::Right => Self::Right,
      KeyCode::Home => Self::Home,
      KeyCode::End => Self::End,
      KeyCode::PageUp => Self::PageUp,
      KeyCode::PageDown => Self::PageDown,
      KeyCode::Delete => Self::Delete,
      KeyCode::Insert => Self::Insert,
      _ => Self::Char('\0'),
    }
  }
}

fn append_bindings(
  target: &mut Vec<ActionBinding>,
  action: Action,
  action_name: &str,
  bindings: &[String],
) -> Result<()> {
  for binding in bindings {
    let sequence = parse_sequence(binding)
      .with_context(|| format!("invalid keymap binding for '{action_name}'"))?;
    target.push(ActionBinding { action, sequence });
  }
  Ok(())
}

fn parse_sequence(input: &str) -> Result<Vec<KeyBinding>> {
  let sequence = input
    .split_whitespace()
    .map(KeyBinding::parse)
    .collect::<Result<Vec<_>>>()?;

  if sequence.is_empty() {
    bail!("empty key sequence");
  }

  Ok(sequence)
}

fn relevant_modifiers(modifiers: KeyModifiers) -> KeyModifiers {
  modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT)
}

pub fn is_text_input_key(key: &KeyEvent) -> bool {
  matches!(key.code, KeyCode::Char(_))
    && !key.modifiers.contains(KeyModifiers::CONTROL)
    && !key.modifiers.contains(KeyModifiers::ALT)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_ctrl_bindings() {
    let binding = KeyBinding::parse("ctrl-a").unwrap();
    let key = KeyBinding::from_key_event(&KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));

    assert_eq!(binding, key);
  }

  #[test]
  fn parses_uppercase_sequence_keys() {
    let sequence = parse_sequence("ctrl-g G").unwrap();
    let expected_second =
      KeyBinding::from_key_event(&KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT));

    assert_eq!(sequence[1], expected_second);
  }

  #[test]
  fn compiles_default_keymap() {
    let mut keymap = KeymapConfig::default().compile().unwrap();
    let select_all = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let clear_selection = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);

    assert_eq!(
      keymap.match_key(&select_all),
      KeymapMatch::Action(Action::SelectAll)
    );
    assert_eq!(
      keymap.match_key(&clear_selection),
      KeymapMatch::Action(Action::ClearSelection)
    );
  }

  #[test]
  fn matches_key_sequences() {
    let config = KeymapConfig {
      jump_end: vec!["ctrl-g G".to_string()],
      ..KeymapConfig::default()
    };
    let mut keymap = config.compile().unwrap();

    assert_eq!(
      keymap.match_key(&KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL)),
      KeymapMatch::Pending
    );
    assert_eq!(
      keymap.match_key(&KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT)),
      KeymapMatch::Action(Action::JumpEnd)
    );
  }
}
