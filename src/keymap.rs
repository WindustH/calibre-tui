use crate::config_file::{CommentedToml, TomlComment, app_config_dir, load_toml_or_reset};
use anyhow::{Context, Result};
use framework_tui::KeyBindings;
use framework_tui::keymap::{InputKeymapOptions, KeymapSection, default_input_keymap, key};
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
      input: default_input_keymap(&InputKeymapOptions::default()),
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
      self.browser.binding_configs(),
      self.detail.binding_configs(),
      self.input.binding_configs(),
      self.global.binding_configs(),
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
