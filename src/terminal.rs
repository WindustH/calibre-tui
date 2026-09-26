//! Terminal setup and restore.
//!
//! The UI is drawn on stdout, or on stderr when stdout is not a terminal, so
//! `calibre-tui | xargs ...` and `$(calibre-tui)` receive only the printed
//! paths. The terminal is restored on normal exit, on error, and on panic.

use anyhow::{Context, Result};
use crossterm::{
  cursor::Show,
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, BufWriter, IsTerminal, Write};
use std::panic;

pub type Tui = Terminal<CrosstermBackend<BufWriter<UiOutput>>>;

/// The stream the UI is drawn on.
pub enum UiOutput {
  Stdout(io::Stdout),
  Stderr(io::Stderr),
}

impl UiOutput {
  fn current() -> Self {
    if io::stdout().is_terminal() {
      Self::Stdout(io::stdout())
    } else {
      Self::Stderr(io::stderr())
    }
  }
}

impl Write for UiOutput {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self {
      Self::Stdout(stdout) => stdout.write(buf),
      Self::Stderr(stderr) => stderr.write(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match self {
      Self::Stdout(stdout) => stdout.flush(),
      Self::Stderr(stderr) => stderr.flush(),
    }
  }
}

pub fn enter() -> Result<Tui> {
  install_panic_hook();
  enable_raw_mode().context("failed to enable raw mode")?;
  let mut output = BufWriter::new(UiOutput::current());
  let setup = execute!(output, EnterAlternateScreen, EnableMouseCapture)
    .context("failed to enter alternate screen")
    .and_then(|()| {
      Terminal::new(CrosstermBackend::new(output)).context("failed to create terminal")
    });
  if setup.is_err() {
    let _ = leave();
  }
  setup
}

pub fn leave() -> Result<()> {
  let raw_mode = disable_raw_mode().context("failed to disable raw mode");
  execute!(
    UiOutput::current(),
    LeaveAlternateScreen,
    DisableMouseCapture,
    Show
  )
  .context("failed to leave alternate screen")?;
  raw_mode
}

/// Restore the terminal before the panic message is printed, so it is
/// readable and the shell is usable.
fn install_panic_hook() {
  let default_hook = panic::take_hook();
  panic::set_hook(Box::new(move |info| {
    let _ = leave();
    default_hook(info);
  }));
}
