mod config;
mod i18n;
mod pipeline;
mod ui;
mod utils;
mod widget;

use anyhow::Result;
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
    // parse arguements
    let args = Args::parse();

    // setup
    let config = config::load_config()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // pipeline
    let mut pipeline = pipeline::Pipeline::new(&config, "filter-and-open");
    let loop_result = (|| -> Result<()> {
        loop {
            let event = pipeline.event_tick()?;
            pipeline.widget_tick(&mut terminal, &event)?;

            if pipeline.should_exit {
                break;
            }
        }
        Ok(())
    })();

    // cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = loop_result {
        println!("application error: {:?}", err)
    }

    Ok(())
}
