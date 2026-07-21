use crate::config_file::{CommentedToml, TomlComment, app_config_dir, load_toml_or_reset};
use anyhow::{Context, Result};
use framework_tui::{KeyBindingConfig, KeyBindings};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct KeymapConfig {
  pub browser: KeymapSection,
  pub detail: KeymapSection,
  pub input: KeymapSection,
  pub global: KeymapSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct KeymapSection {
  pub keymap: Vec<KeymapEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeymapEntry {
  pub on: KeymapOn,
  pub run: String,
  pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeymapOn {
  One(String),
  Many(Vec<String>),
}

impl Default for KeymapConfig {
  fn default() -> Self {
    Self {
      browser: KeymapSection {
        keymap: vec![
          key("esc", "quit", "Quit"),
          key("ctrl-c", "quit", "Quit"),
          key("enter", "open", "Open selected books"),
          key("up", "move_up", "Move up"),
          key("down", "move_down", "Move down"),
          key("pgup", "page_up", "Move one page up"),
          key("pgdn", "page_down", "Move one page down"),
          key("pagedown", "page_down", "Move one page down"),
          key("home", "jump_start", "Jump to first result"),
          key("end", "jump_end", "Jump to last result"),
          key("tab", "toggle_selection", "Toggle selection"),
          key("ctrl-a", "select_all", "Select all results"),
          key("ctrl-x", "clear_selection", "Clear selection"),
          key("backspace", "delete_input", "Delete search input"),
          key("ctrl-p", "print_paths", "Print selected paths and quit"),
          key("ctrl-y", "copy_paths", "Copy selected paths"),
          key(["ctrl-s", "t"], "sort title asc", "Sort title ascending"),
          key(["ctrl-s", "T"], "sort title desc", "Sort title descending"),
          key(
            ["ctrl-s", "a"],
            "sort authors asc",
            "Sort authors ascending",
          ),
          key(
            ["ctrl-s", "A"],
            "sort authors desc",
            "Sort authors descending",
          ),
          key(["ctrl-s", "s"], "sort series asc", "Sort series ascending"),
          key(
            ["ctrl-s", "S"],
            "sort series desc",
            "Sort series descending",
          ),
          key(
            ["ctrl-s", "f"],
            "sort formats asc",
            "Sort formats ascending",
          ),
          key(
            ["ctrl-s", "F"],
            "sort formats desc",
            "Sort formats descending",
          ),
          key(["ctrl-s", "g"], "sort tags asc", "Sort tags ascending"),
          key(["ctrl-s", "G"], "sort tags desc", "Sort tags descending"),
        ],
      },
      detail: KeymapSection::default(),
      input: default_input_keymap_section(),
      global: KeymapSection {
        keymap: vec![
          key("f1", "help", "Show key bindings"),
          key("ctrl-t", "command", "Enter command"),
        ],
      },
    }
  }
}

impl KeymapConfig {
  pub fn bindings(&self) -> KeyBindings {
    KeyBindings::from_sections(
      binding_configs(&self.browser.keymap),
      binding_configs(&self.detail.keymap),
      binding_configs(&self.input.keymap),
      binding_configs(&self.global.keymap),
    )
  }
}

impl CommentedToml for KeymapConfig {
  fn comments() -> &'static [TomlComment] {
    &[
      TomlComment {
        path: "",
        lines: &[
          "Keyboard shortcuts are grouped by context.",
          "Use a string in `on` for one key, or an array for a key sequence.",
          "Key names include enter, esc, tab, backspace, up, down, left, right, home, end, pgup, pgdn, delete, insert, space, f1, single characters, ctrl-x, and alt-x.",
        ],
      },
      TomlComment {
        path: "browser",
        lines: &["Active while browsing and searching books."],
      },
      TomlComment {
        path: "browser.keymap",
        lines: &["A browser shortcut. Repeated shortcut fields are documented only once."],
      },
      TomlComment {
        path: "browser.keymap.on",
        lines: &["Key or key sequence that triggers this binding."],
      },
      TomlComment {
        path: "browser.keymap.run",
        lines: &["Action or command string to run."],
      },
      TomlComment {
        path: "browser.keymap.desc",
        lines: &["Description shown in F1 help and which-key hints."],
      },
      TomlComment {
        path: "detail",
        lines: &["Reserved for detail views. Leave empty if unused."],
      },
      TomlComment {
        path: "input",
        lines: &["Active while the command prompt is open."],
      },
      TomlComment {
        path: "input.keymap",
        lines: &["Command prompt shortcut bindings."],
      },
      TomlComment {
        path: "global",
        lines: &["Available from normal browsing contexts."],
      },
      TomlComment {
        path: "global.keymap",
        lines: &["Global shortcut bindings."],
      },
    ]
  }
}

pub fn load_keymap() -> Result<KeyBindings> {
  let config_dir = app_config_dir()?;
  let keymap_path = config_dir.join("keymap.toml");
  let config: KeymapConfig = load_toml_or_reset(&keymap_path, KeymapConfig::default(), "keymap")
    .with_context(|| format!("failed to load keymap file '{}'", keymap_path.display()))?;

  Ok(config.bindings())
}

fn default_input_keymap_section() -> KeymapSection {
  KeymapSection {
    keymap: vec![
      key("f1", "help", "Show key bindings"),
      key("esc", "cancel", "Cancel input"),
      key("enter", "submit", "Submit input"),
      key("backspace", "backspace", "Delete before cursor"),
      key("delete", "delete", "Delete under cursor"),
      key("left", "move_left", "Move cursor left"),
      key("right", "move_right", "Move cursor right"),
      key("home", "move_start", "Move cursor to start"),
      key("ctrl-a", "move_start", "Move cursor to start"),
      key("end", "move_end", "Move cursor to end"),
      key("ctrl-e", "move_end", "Move cursor to end"),
      key("ctrl-u", "kill_before_cursor", "Delete before cursor"),
      key("ctrl-k", "kill_after_cursor", "Delete after cursor"),
      key("tab", "completion_next", "Select next completion"),
      key(
        "backtab",
        "completion_previous",
        "Select previous completion",
      ),
      key("up", "history_previous", "Previous command history"),
      key("down", "history_next", "Next command history"),
    ],
  }
}

fn binding_configs(entries: &[KeymapEntry]) -> Vec<KeyBindingConfig> {
  entries
    .iter()
    .map(|entry| KeyBindingConfig {
      on: keymap_on_values(&entry.on),
      action: entry.run.clone(),
      desc: entry.desc.clone(),
    })
    .collect()
}

fn keymap_on_values(on: &KeymapOn) -> Vec<String> {
  match on {
    KeymapOn::One(value) => vec![value.clone()],
    KeymapOn::Many(values) => values.clone(),
  }
}

fn key(on: impl Into<KeymapOn>, run: &str, desc: &str) -> KeymapEntry {
  KeymapEntry {
    on: on.into(),
    run: run.to_string(),
    desc: desc.to_string(),
  }
}

impl From<&str> for KeymapOn {
  fn from(value: &str) -> Self {
    Self::One(value.to_string())
  }
}

impl<const N: usize> From<[&str; N]> for KeymapOn {
  fn from(values: [&str; N]) -> Self {
    Self::Many(values.into_iter().map(str::to_string).collect())
  }
}
