//! Terminal setup.
//!
//! The UI is drawn on stdout, or on stderr when stdout is not a terminal, so
//! `calibre-tui | xargs ...` and `$(calibre-tui)` receive only the printed
//! paths. The terminal is restored on normal exit, on error, and on panic.

use anyhow::{Context, Result};
use framework_tui::{TerminalOptions, TerminalOutput, TerminalSession, install_panic_hook};

pub type Tui = TerminalSession;

pub fn enter() -> Result<Tui> {
  // A panic on the UI thread restores the terminal before its message is
  // printed. The only other thread waits for launched viewers.
  install_panic_hook(|_, _| {});
  TerminalSession::enter(TerminalOptions {
    output: TerminalOutput::stdout_if_terminal(),
    bracketed_paste: false,
    ..TerminalOptions::default()
  })
  .context("failed to set up the terminal")
}
