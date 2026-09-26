mod app;
mod config;
mod config_file;
mod keymap;
mod layout;
mod library;
mod search;
mod sort;
mod terminal;
mod theme;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;

/// Search a Calibre library, open books, or print their paths.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  /// Quit after opening books with the `open` action (Enter)
  #[arg(long)]
  exit_on_open: bool,
}

fn main() -> Result<()> {
  let args = Args::parse();
  let config = config::load_config().context("failed to load configuration")?;
  let keymap = keymap::load_keymap().context("failed to load keymap")?;
  let layout = layout::load_layout().context("failed to load layout")?;
  let theme = theme::load_theme().context("failed to load theme")?;
  let mut app = app::App::new(config, keymap, layout, theme, args.exit_on_open)?;

  let mut terminal = terminal::enter()?;
  let result = app.run(&mut terminal);
  drop(terminal);
  let restored = terminal::leave();
  let paths = result?;
  restored?;

  print_paths(&paths)
}

/// Print one path per line, byte-exact; a closed pipe (`| head`) is not an error.
fn print_paths(paths: &[PathBuf]) -> Result<()> {
  let mut stdout = io::stdout().lock();
  let written = paths.iter().try_for_each(|path| {
    stdout.write_all(path.as_os_str().as_encoded_bytes())?;
    stdout.write_all(b"\n")
  });
  match written.and_then(|()| stdout.flush()) {
    Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
    result => result.context("failed to print paths"),
  }
}
