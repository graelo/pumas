//! Main runner

use clap::{CommandFactory, Parser};
use clap_complete::generate;

use pumas::{
    config::{Command, Config},
    monitor, Result,
};

fn main() -> Result<()> {
    let config = Config::parse();
    match config.command {
        Command::Run { args } => {
            monitor::run(args)?;
        }

        Command::Server { port, sample_rate_ms } => {
            monitor::run_server(port, sample_rate_ms)?;
        }

        Command::GenerateCompletion { shell } => {
            let mut app = Config::command();
            let name = app.get_name().to_string();
            generate(shell, &mut app, name, &mut std::io::stdout());
        }
    }

    Ok(())
}
