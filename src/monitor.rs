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
    parser::{powermetrics::Metrics, soc::SocInfo},
    ui, Result,
};

/// Configures the UI, and launches powermetrics regularly to update the values.
pub fn run(args: RunConfig) -> Result<()> {
    // println!(
    //     "run: {}, {}, {}, {:?}",
    //     args.sample_rate_ms, args.color, args.average, args.max_show_count
    // );

    let stdout = io::stdout().into_alternate_screen()?.into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    // let stdout = AlternateScreen::from(stdout);

    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let soc_info = SocInfo::new()?;
    let app = App::new(soc_info);
    run_app(
        &mut terminal,
        app,
        Duration::from_millis(args.sample_rate_ms as u64),
    )
    .expect("Cannot run app");

    Ok(())
}

/// Starts the events stream source and launches the event loop.
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> std::result::Result<(), Box<dyn Error>> {
    let events = start_event_threads(tick_rate);

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        match events.recv()? {
            Event::Input(key) => match key {
                Key::Char(c) => app.on_key(c),
                // Key::Up => app.on_up(),
                // Key::Down => app.on_down(),
                Key::Left => app.on_left(),
                Key::Right => app.on_right(),
                _ => {}
            },
            // Event::Tick => app.on_tick(),
            Event::Metrics(metrics) => app.on_metrics(metrics),
        }
        if app.should_quit {
            return Ok(());
        }
    }
}

enum Event {
    Input(Key),
    // Tick,
    Metrics(Metrics),
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

    thread::spawn(move || stream_powermetrics(tick_rate, tx));

    rx
}

fn stream_powermetrics(tick_rate: Duration, tx: mpsc::Sender<Event>) {
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

    // A plist output from powermetrics typically contains 713 lines.
    let num_lines = 714;
    // Prepare a buffer to store the set of lines which correspond to a full output.
    let mut buffer: Vec<String> = Vec::<String>::with_capacity(num_lines);

    for line in stdout_lines.flatten() {
        if line != "</plist>" {
            // Process all lines but the last.
            if line.starts_with(char::from(0)) {
                // Trim the leading null character if present (happens only on the 1st line).
                let line = line.trim_start_matches(char::from(0)).to_string();
                buffer.push(line);
            } else {
                buffer.push(line);
            }
        } else {
            // Process the last line.
            buffer.push(line);

            // Fix a powermetrics bug by removing the last `idle_ratio` line. This should be the
            // (n-5)th line, so we only iterate over the last 10 lines.
            let pos = buffer
                .iter()
                .rev()
                .take(10)
                .position(|line| line.starts_with("<key>idle_ratio</key>"));
            if let Some(pos) = pos {
                buffer.remove(buffer.len() - pos - 1);
            }

            let text = buffer.join("\n");
            buffer.clear();

            let powermetrics = match Metrics::from_bytes(text.as_bytes()) {
                Ok(metrics) => metrics,
                Err(err) => {
                    eprintln!("{err}");
                    cmd.kill().unwrap();
                    break;
                }
            };
            if let Err(err) = tx.send(Event::Metrics(powermetrics)) {
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
