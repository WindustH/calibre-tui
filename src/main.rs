mod app;
mod config;
mod config_file;
mod filter;
mod i18n;
mod keymap;
mod layout;
mod sort;
mod theme;
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
  exit_on_open: bool,
}

fn main() -> Result<()> {
  let args = Args::parse();
  let config = config::load_config().context("failed to load configuration")?;
  let keymap = keymap::load_keymap().context("failed to load keymap")?;
  let layout = layout::load_layout().context("failed to load layout")?;
  let theme = theme::load_theme().context("failed to load theme")?;
  let mut app = app::App::new(config, keymap, layout, theme, args.exit_on_open)?;

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
