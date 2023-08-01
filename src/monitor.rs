//! The monitor main loop.

use std::{
    error::Error,
    io::{self, BufRead, BufReader},
    process,
    sync::mpsc,
    thread,
    time::Duration,
};

use ratatui::{
    backend::{Backend, TermionBackend},
    Terminal,
};
use termion::{
    event::Key,
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
    screen::IntoAlternateScreen,
};

use crate::{
    app::App,
    config::RunConfig,
    metrics,
    modules::{powermetrics, soc::SocInfo, sysinfo},
    ui, Result,
};

/// Configure the UI, and launch the main loop.
pub fn run(args: RunConfig) -> Result<()> {
    let stdout = io::stdout().into_alternate_screen()?.into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);

    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let soc_info = SocInfo::new()?;
    let app = App::new(soc_info, args.colors());

    main_loop(
        &mut terminal,
        app,
        Duration::from_millis(args.sample_rate_ms as u64),
    )
    .expect("Cannot continue to run the app");

    Ok(())
}

enum Event {
    Input(Key),
    // Tick,
    Metrics(metrics::Metrics),
}

/// Start the event stream sources and launch the event loop.
fn main_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> std::result::Result<(), Box<dyn Error>> {
    let events = start_event_threads(tick_rate);

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        match events.recv()? {
            // Event::Tick => app.on_tick(),
            Event::Input(key) => match key {
                Key::Char(c) => app.on_key(c),
                Key::Esc => app.on_key('q'),
                // Key::Up => app.on_up(),
                // Key::Down => app.on_down(),
                Key::Left => app.on_left(),
                Key::Right => app.on_right(),
                _ => {}
            },
            Event::Metrics(metrics) => app.on_metrics(metrics),
        }
        if app.should_quit {
            return Ok(());
        }
    }
}

/// Run event threads.
fn start_event_threads(tick_rate: Duration) -> mpsc::Receiver<Event> {
    let (tx, rx) = mpsc::channel();

    let tx_keys = tx.clone();
    thread::spawn(move || {
        let stdin = io::stdin();
        for key in stdin.keys().flatten() {
            if let Err(err) = tx_keys.send(Event::Input(key)) {
                eprintln!("{}", err);
                return;
            }
        }
    });

    // let tx_tick = tx.clone();
    // thread::spawn(move || loop {
    //     if let Err(err) = tx_tick.send(Event::Tick) {
    //         eprintln!("{}", err);
    //         break;
    //     }
    //     thread::sleep(tick_rate);
    // });

    thread::spawn(move || stream_metrics(tick_rate, tx));

    rx
}

/// Stream metrics and send them to the event loop.
///
/// This function starts the powermetrics tool in streaming mode with the configured sampling
/// period (0.5 sec by default), so that it outputs entire plist messages at each period.
///
/// When a plist message is complete, this function also gathers CPU usage from the sysinfo crate
/// for more accurate per-core usage (powermetrics is half-broken on M2 chips).
///
/// This function will run in a separate thread and stream data for the entire duration of the app.
///
/// # Note
///
/// Powermetrics outputs a plist file, but it is not valid XML, so we fix the issues before sending
/// them to the plist parser.
fn stream_metrics(tick_rate: Duration, tx: mpsc::Sender<Event>) {
    let sample_rate_ms = format!("{}", tick_rate.as_millis());

    let binary = "/usr/bin/powermetrics";
    let args = vec![
        "--sample-rate",
        sample_rate_ms.as_str(),
        // "--sample-count",
        // "10",
        "--samplers",
        "cpu_power,gpu_power,thermal",
        "-f",
        "plist",
    ];

    let mut cmd = process::Command::new(binary)
        .args(&args)
        .stdout(process::Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = cmd.stdout.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    let mut buffer = powermetrics::Buffer::new();
    let mut system_state = sysinfo::SystemState::new();

    // Main loop.
    //
    // Read the lines of the plist messages from powermetrics, one by one, for the entire duration
    // of the app.
    //
    // When the last line of a plist message is read: build the `powermetrics::Metrics` struct and
    // gather CPU usage from sysinfo.
    //
    // Finally, send metrics to the event loop.
    //
    for line in stdout_lines.flatten() {
        if line != "</plist>" {
            buffer.append_line(line);
        } else {
            buffer.append_last_line(line);
            let text = buffer.finalize();

            let power_metrics = match metrics::Metrics::from_bytes(text.as_bytes()) {
                Ok(metrics) => metrics,
                Err(err) => {
                    eprintln!("{err}");
                    cmd.kill().unwrap();
                    break;
                }
            };

            let sysinfo_metrics = system_state.latest_metrics();

            let cpu_usage: Vec<f32> = sysinfo_metrics
                .cpu_metrics
                .iter()
                .map(|m| m.active_ratio)
                .collect();

            let metrics = power_metrics.set_cpus_active_ratio(&cpu_usage[..]);

            if let Err(err) = tx.send(Event::Metrics(metrics)) {
                eprintln!("{err}");
                cmd.kill().unwrap();
                break;
            }
        }
    }

    cmd.try_wait().unwrap();
}

// pub fn exec_stream<P: AsRef<Path>>(binary: P, args: Vec<&'static str>) {
//     let mut cmd = process::Command::new(binary.as_ref())
//         .args(&args)
//         .stdout(process::Stdio::piped())
//         .spawn()
//         .unwrap();
//
//     {
//         let stdout = cmd.stdout.as_mut().unwrap();
//         let stdout_reader = BufReader::new(stdout);
//         let stdout_lines = stdout_reader.lines();
//
//         let mut buffer: Vec<String> = vec![];
//         for line in stdout_lines.flatten() {
//             if line != "</plist>" {
//                 // Process all lines but the last.
//                 if line.starts_with(char::from(0)) {
//                     // Trim the leading null character if present (happens only on the 1st line).
//                     let line = line.trim_start_matches(char::from(0)).to_string();
//                     buffer.push(line);
//                 } else {
//                     buffer.push(line);
//                 }
//             } else {
//                 // Process the last line.
//                 buffer.push(line);
//
//                 // Fix a powermetrics bug by removing the last `idle_ratio` line (n-5)th line.
//                 let pos = buffer
//                     .iter()
//                     .rev()
//                     .take(10)
//                     .position(|line| line.starts_with("<key>idle_ratio</key>"));
//                 if let Some(pos) = pos {
//                     buffer.remove(buffer.len() - pos - 1);
//                 }
//
//                 let text = buffer.join("\n");
//                 buffer.clear();
//                 if false {
//                     println!("{}", text);
//                     println!("--");
//                 } else {
//                     let powermetrics = match Metrics::from_bytes(text.as_bytes()) {
//                         Ok(metrics) => metrics,
//                         Err(err) => {
//                             eprintln!("{err}");
//                             eprintln!("{:?}", text.as_bytes());
//                             cmd.kill().unwrap();
//                             break;
//                         }
//                     };
//                     println!("{}", powermetrics.cpu_mw);
//                 }
//             }
//         }
//     }
//
//     cmd.wait().unwrap();
// }
