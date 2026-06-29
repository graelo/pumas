//! The monitor entry point.
//!
//! `run` branches on `--json`: the JSON path streams metrics to stdout with no
//! UI or channel; the UI path spawns the backend collector on its own OS thread
//! and runs the iocraft [`PumasApp`] on `smol` (no tokio anywhere — MIGRATION.md
//! §7.7).

use std::{io::Write, thread};

use iocraft::prelude::*;

use crate::{
    Result,
    backend::{self, frame::Frame},
    config::RunConfig,
    error::Error as CrateError,
    modules::soc::SocInfo,
    ui::{app_root::PumasApp, theme::Theme},
};

/// Launch the monitor.
///
/// In UI mode, build the frame channel, spawn the collector thread, and run the
/// fullscreen `PumasApp`. In JSON mode, run the exporter loop directly.
pub fn run(args: RunConfig) -> Result<()> {
    let soc_info = SocInfo::new()?;

    let result = if args.json {
        backend::run_exporter(soc_info, args)
    } else {
        run_ui(soc_info, args)
    };

    if let Err(err) = result {
        eprintln!("{err}");
        if let CrateError::PowermetricsNonZeroExit(status, msg) = &err
            && status.code() == Some(1)
            && msg.contains("superuser")
        {
            eprintln!(
                "macOS requires superuser privileges to access power metrics.\n\n    sudo pumas run\n"
            );
        }
    }

    Ok(())
}

/// Run the iocraft UI: spawn the collector, render `PumasApp` fullscreen, then
/// surface the collector's result (so the sudo hint still prints when
/// powermetrics exits non-zero before any frame arrives).
fn run_ui(soc_info: SocInfo, args: RunConfig) -> Result<()> {
    install_panic_hook();

    let theme = Theme::from(&args.colors());
    let header = backend::frame::render_header(&soc_info);

    // Bounded(4): event-gated repaint, no free-running animation loop.
    let (tx, rx) = smol::channel::bounded::<Frame>(4);

    let collector = thread::spawn(move || backend::run_collector(soc_info, args, tx));

    smol::block_on(
        element! {
            PumasApp(rx: Some(rx), header: Some(header), theme: theme)
        }
        .fullscreen(),
    )?;

    // The UI has exited (user quit, or the collector closed the channel).
    // Joining yields the collector's `Result`; an `Err` here drives the
    // post-run sudo-hint handling in `run`.
    match collector.join() {
        Ok(res) => res,
        Err(_) => Ok(()), // collector panicked; the panic hook already logged it
    }
}

/// Install a panic hook that appends to a log file, since the fullscreen TUI
/// swallows stderr (MIGRATION.md §7.7).
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let path = std::env::temp_dir().join("pumas-panic.log");
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(file, "{info}");
        }
        default_hook(info);
    }));
}
