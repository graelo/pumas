//! Configuration.

use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// Power usage monitor for Apple Silicon.
#[derive(Debug, Parser)]
#[clap(author, about, version)]
#[clap(propagate_version = true)]
pub struct Config {
    /// Selection of commands.
    #[command(subcommand)]
    pub command: Command,
}

/// Indicate whether to run or generate completions.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the power usage monitor.
    Run {
        /// Configuration
        #[command(flatten)]
        args: RunConfig,
    },

    /// Print a shell completion script to stdout.
    GenerateCompletion {
        /// Shell for which you want completion.
        #[arg(value_enum, value_parser = clap::value_parser!(Shell))]
        shell: Shell,
    },
}

/// UI configuration.
#[derive(Debug, clap::Args)]
pub struct RunConfig {
    /// Update rate (milliseconds): min: 100.
    ///
    /// Rate at which metrics are sampled and displayed.
    #[arg(short='i', long="sample-rate", default_value = "1000",
        value_parser = clap::value_parser!(u16).range(100..))]
    pub sample_rate_ms: u16,

    /// Accent color: ASCII code in 0~255.
    #[arg(long, default_value = "2")]
    pub accent_color: u8,

    /// Gauge background color: ASCII code in 0~255.
    #[arg(long, default_value = "7")]
    pub gauge_bg_color: u8,
}
