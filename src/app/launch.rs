//! Handing books to other programs: file openers and the clipboard.

use crate::config::OpenConfig;
use crate::library::Book;
use anyhow::{Context, Result, bail};
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

/// Open a book's file with its configured command or the system opener.
/// Returns without waiting for the viewer.
pub fn open_book(config: &OpenConfig, book: &Book) -> Result<()> {
  let Some(path) = &book.path else {
    bail!("'{}' has no book file", book.title);
  };
  if !path.is_file() {
    bail!("book file not found: {}", path.display());
  }

  match opener_command(config, path) {
    Some(command) => spawn_opener(command, path),
    None => open::that_detached(path).map_err(Into::into),
  }
  .with_context(|| format!("failed to open '{}'", book.title))
}

/// The configured command for the file's extension (case-insensitive).
fn opener_command<'a>(config: &'a OpenConfig, path: &Path) -> Option<&'a [String]> {
  let extension = path.extension()?.to_str()?;
  config.commands.iter().find_map(|(format, command)| {
    let format = format.trim().trim_start_matches('.');
    let usable = command.first().is_some_and(|program| !program.is_empty());
    (usable && format.eq_ignore_ascii_case(extension)).then_some(command.as_slice())
  })
}

fn spawn_opener(command: &[String], path: &Path) -> Result<()> {
  let (program, args) = command.split_first().context("empty open command")?;
  let mut child = Command::new(program);
  let mut includes_path = false;
  for arg in args {
    if arg.contains("{path}") {
      includes_path = true;
      child.arg(substitute_path(arg, path));
    } else {
      child.arg(arg);
    }
  }
  if !includes_path {
    child.arg(path);
  }

  // Keep the viewer's output off the TUI, and give it its own process group
  // so it is not killed along with the terminal.
  child
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null());
  #[cfg(unix)]
  std::os::unix::process::CommandExt::process_group(&mut child, 0);

  let mut child = child
    .spawn()
    .with_context(|| format!("failed to run open command '{}'", command.join(" ")))?;
  // Reap the viewer when it exits instead of leaving a zombie behind.
  thread::spawn(move || child.wait());
  Ok(())
}

/// Replace every `{path}` in `arg` without lossy UTF-8 conversion.
fn substitute_path(arg: &str, path: &Path) -> OsString {
  let mut value = OsString::new();
  for (index, part) in arg.split("{path}").enumerate() {
    if index > 0 {
      value.push(path);
    }
    value.push(part);
  }
  value
}

type ClipboardCommand = (&'static str, &'static [&'static str]);

pub fn copy_to_clipboard(text: &str) -> Result<()> {
  let mut errors = Vec::new();
  for (program, args) in clipboard_commands() {
    match run_clipboard_command(program, args, text) {
      Ok(()) => return Ok(()),
      Err(error) => errors.push(format!("{program}: {error:#}")),
    }
  }
  bail!("no clipboard command succeeded ({})", errors.join("; "))
}

#[cfg(target_os = "macos")]
fn clipboard_commands() -> Vec<ClipboardCommand> {
  vec![("pbcopy", &[])]
}

#[cfg(windows)]
fn clipboard_commands() -> Vec<ClipboardCommand> {
  vec![("clip", &[])]
}

/// Prefer the tool for the running display server, then try the others.
#[cfg(all(unix, not(target_os = "macos")))]
fn clipboard_commands() -> Vec<ClipboardCommand> {
  const WL_COPY: ClipboardCommand = ("wl-copy", &[]);
  const XCLIP: ClipboardCommand = ("xclip", &["-selection", "clipboard"]);
  const XSEL: ClipboardCommand = ("xsel", &["--clipboard", "--input"]);

  let mut commands = Vec::new();
  if std::env::var_os("WAYLAND_DISPLAY").is_some() {
    commands.push(WL_COPY);
  }
  if std::env::var_os("DISPLAY").is_some() {
    commands.extend([XCLIP, XSEL]);
  }
  for command in [WL_COPY, XCLIP, XSEL] {
    if !commands.contains(&command) {
      commands.push(command);
    }
  }
  commands
}

fn run_clipboard_command(program: &str, args: &[&str], text: &str) -> Result<()> {
  // Clipboard tools may fork a daemon that keeps inherited handles open, so
  // their output is discarded rather than piped.
  let mut child = Command::new(program)
    .args(args)
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .context("failed to start command")?;

  let mut stdin = child.stdin.take().context("failed to open command stdin")?;
  let written = stdin
    .write_all(text.as_bytes())
    .context("failed to write clipboard data");
  drop(stdin);

  let status = child
    .wait()
    .context("failed to wait for clipboard command")?;
  written?;
  if !status.success() {
    bail!("command exited with {status}");
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::BTreeMap;
  use std::path::PathBuf;

  fn config(entries: &[(&str, &[&str])]) -> OpenConfig {
    OpenConfig {
      commands: entries
        .iter()
        .map(|(format, command)| {
          (
            format.to_string(),
            command.iter().map(|arg| arg.to_string()).collect(),
          )
        })
        .collect::<BTreeMap<_, _>>(),
    }
  }

  #[test]
  fn opener_matches_extension_case_insensitively() {
    let config = config(&[(".PDF", &["zathura"]), ("epub", &[]), ("mobi", &[""])]);
    let path = PathBuf::from("/books/a.pdf");
    assert_eq!(
      opener_command(&config, &path),
      Some(["zathura".to_string()].as_slice())
    );
    // Empty commands fall back to the system opener.
    assert_eq!(opener_command(&config, Path::new("a.epub")), None);
    assert_eq!(opener_command(&config, Path::new("a.mobi")), None);
    assert_eq!(opener_command(&config, Path::new("no-extension")), None);
  }

  #[test]
  fn path_placeholder_is_substituted_in_place() {
    let path = Path::new("/books/Title {x}/a b.pdf");
    assert_eq!(
      substitute_path("--file={path}#{path}", path),
      OsString::from("--file=/books/Title {x}/a b.pdf#/books/Title {x}/a b.pdf")
    );
  }

  #[test]
  fn books_without_files_are_reported_not_opened() {
    let book = Book {
      title: "Metadata only".to_string(),
      ..Book::default()
    };
    let error = open_book(&OpenConfig::default(), &book).unwrap_err();
    assert_eq!(error.to_string(), "'Metadata only' has no book file");

    let book = Book {
      path: Some(PathBuf::from("/nonexistent/calibre-tui/book.epub")),
      ..book
    };
    let error = open_book(&OpenConfig::default(), &book).unwrap_err();
    assert!(error.to_string().starts_with("book file not found"));
  }
}
