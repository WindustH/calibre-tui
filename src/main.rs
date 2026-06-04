mod app;
mod config;
mod filter;
mod i18n;
mod keymap;
mod ui;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(long)]
  exit_on_submit: bool,
  #[arg(long)]
  print_path: bool,
}

fn main() -> Result<()> {
  let args = Args::parse();
  let config = config::load_config().context("failed to load configuration")?;
  let keymap = keymap::load_keymap().context("failed to load keymap")?;
  let mut app = app::App::new(config, keymap, args.exit_on_submit, args.print_path)?;

  let mut terminal = setup_terminal()?;
  let result = app.run(&mut terminal);
  restore_terminal(&mut terminal)?;
  drop(terminal);

  for path in result? {
    println!("{}", path.display());
  }

  Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
  enable_raw_mode().context("failed to enable raw mode")?;

  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
    .context("failed to enter alternate screen")?;

  Terminal::new(CrosstermBackend::new(stdout)).context("failed to create terminal")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
  disable_raw_mode().context("failed to disable raw mode")?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )
  .context("failed to leave alternate screen")?;
  terminal.show_cursor().context("failed to show cursor")?;
  Ok(())
}
