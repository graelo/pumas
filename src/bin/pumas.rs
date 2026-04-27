//! Main runner

use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::exit;

use clap::{CommandFactory, Parser};
use clap_complete::generate;

use pumas::{
    Result,
    config::{Command, Config},
    monitor,
};

/// Re-exec via `sudo` if not already running as root.
fn ensure_root() {
    if is_root() {
        return;
    }

    // Credentials still cached from a recent run — elevate straight away.
    if sudo_credentials_cached() {
        reexec_sudo();
    }

    // Prompt for password and validate until successful.
    loop {
        let password = rpassword::prompt_password(
            "Pumas requires root privileges to read power metrics.\nPassword: ",
        )
        .unwrap_or_else(|_| {
            eprintln!("error: failed to read password");
            exit(1);
        });

        let mut child = std::process::Command::new("sudo")
            .arg("-S")
            .arg("-v")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap_or_else(|_| {
                eprintln!("error: failed to execute sudo");
                exit(1);
            });

        if let Some(mut stdin) = child.stdin.take() {
            let _ = writeln!(stdin, "{password}");
        }

        let status = child.wait().unwrap_or_else(|_| {
            eprintln!("error: failed to wait for sudo");
            exit(1);
        });

        if status.success() {
            break;
        }

        eprintln!("Sorry, incorrect password. Try again.");
    }

    reexec_sudo();
}

/// Check whether sudo credentials are already cached (no password needed).
fn sudo_credentials_cached() -> bool {
    std::process::Command::new("sudo")
        .arg("-n")
        .arg("-v")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Replace the current process with `sudo pumas …`.
///
/// Uses `exec()` so sudo takes over this process directly — no parent/child
/// split that could interfere with the TUI's terminal handling.
fn reexec_sudo() -> ! {
    let args: Vec<String> = std::env::args().collect();
    let err = std::process::Command::new("sudo")
        .args(&args)
        .exec();

    eprintln!("error: failed to execute sudo: {err}");
    exit(1);
}

fn is_root() -> bool {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<u32>().ok())
        .is_some_and(|uid| uid == 0)
}

fn main() -> Result<()> {
    let config = Config::parse();
    match config.command {
        Command::Run { args } => {
            ensure_root();
            monitor::run(args)?;
        }

        Command::GenerateCompletion { shell } => {
            let mut app = Config::command();
            let name = app.get_name().to_string();
            generate(shell, &mut app, name, &mut std::io::stdout());
        }
    }

    Ok(())
}
