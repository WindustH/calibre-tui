mod config;
mod i18n;
mod pipeline;
mod ui;
mod utils;
mod widget;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    exit_on_open: bool,
}

enum ControlCode {
    Quit,
}

// constants for tick rates
const UI_REFRESH_RATE: u64 = 60;
const TICK_RATE: u64 = 60;

fn main() -> Result<()> {
    // parse arguments
    let _args = Args::parse();

    // setup
    let config = config::load_config().context("failed to load configuration")?;
    enable_raw_mode().context("failed to enable raw mode")?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Arc::new(Mutex::new(
        Terminal::new(backend).context("failed to create terminal")?,
    ));

    // pipeline
    let pipeline = Arc::new(pipeline::Pipeline::new(&config, "filter-and-open"));
    pipeline
        .update_ui_rects(terminal.lock().unwrap().size()?)
        .context("failed to update ui rect")?;

    let (event_tx, event_rx) = mpsc::channel::<Event>();
    let (control_tx, _control_rx) = mpsc::channel::<ControlCode>();

    // widgets tick threads
    let mut widget_control_txs = HashMap::new();
    for widget_id in pipeline.widgets.keys() {
        let (tx, rx) = mpsc::channel::<ControlCode>();
        widget_control_txs.insert(widget_id.clone(), tx);

        let widget = pipeline.widgets.get(widget_id).unwrap().clone();
        thread::spawn(move || {
            let tick_duration = Duration::from_millis(1000 / TICK_RATE);
            loop {
                let start = Instant::now();
                widget.tick();

                if let Ok(ControlCode::Quit) = rx.try_recv() {
                    break;
                }

                let elapsed = start.elapsed();
                if elapsed < tick_duration {
                    thread::sleep(tick_duration - elapsed);
                }
            }
        });
    }

    // ui draw threads
    let mut ui_control_txs = HashMap::new();
    let ui_rects = pipeline.ui_rects.lock().unwrap().clone();
    for (widget_id, _rect) in ui_rects.iter() {
        let (tx, rx) = mpsc::channel::<ControlCode>();
        ui_control_txs.insert(widget_id.clone(), tx);

        let widget = pipeline.widgets.get(widget_id).unwrap().clone();
        let terminal = Arc::clone(&terminal);
        let pipeline_clone = Arc::clone(&pipeline);
        let widget_id_clone = widget_id.clone();

        thread::spawn(move || {
            let frame_duration = Duration::from_millis(1000 / UI_REFRESH_RATE);
            loop {
                let start = Instant::now();
                if let Some(ui) = widget.as_ui() {
                    if let Some(rect) = pipeline_clone.ui_rects.lock().unwrap().get(&widget_id_clone).cloned() {
                        // Errors should be handled, e.g., by logging
                        let _ = ui.draw_tick(Arc::clone(&terminal), rect);
                    }
                }

                if let Ok(ControlCode::Quit) = rx.try_recv() {
                    break;
                }

                let elapsed = start.elapsed();
                if elapsed < frame_duration {
                    thread::sleep(frame_duration - elapsed);
                }
            }
        });
    }

    // event polling thread
    let pipeline_clone = Arc::clone(&pipeline);
    let event_thread_control_tx = control_tx.clone();
    thread::spawn(move || {
        let tick_duration = Duration::from_millis(1000 / TICK_RATE);
        loop {
            let start = Instant::now();
            if let Some(event) = pipeline_clone.event_tick().unwrap() {
                if event_tx.send(event).is_err() {
                    break; // Main thread has disconnected
                }
            }
            if *pipeline_clone.should_exit.lock().unwrap() {
                let _ = event_thread_control_tx.send(ControlCode::Quit);
                break;
            }

            let elapsed = start.elapsed();
            if elapsed < tick_duration {
                thread::sleep(tick_duration - elapsed);
            }
        }
    });

    // main event loop
    // this loop is event-driven and does not need a rate limit
    // tt blocks on event_rx until an event is received
    for event in event_rx {
        for widget in pipeline.widgets.values() {
            if let Some(ui) = widget.as_ui() {
                let _ = ui.event_tick(&event);
            }
        }
    }

    // shutdown
    // Send quit signal to all spawned threads
    for tx in widget_control_txs.values() {
        let _ = tx.send(ControlCode::Quit);
    }
    for tx in ui_control_txs.values() {
        let _ = tx.send(ControlCode::Quit);
    }
    // the pipeline event thread will exit on its own when the main event loop ends.

    // cleanup
    {
        // Drop the terminal lock before cleanup to avoid deadlocks
        let mut terminal = terminal.lock().unwrap();
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
    }

    Ok(())
}