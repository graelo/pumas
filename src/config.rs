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

    /// Run as a Prometheus exporter.
    Server {
        /// Port to listen on.
        #[arg(short, long, default_value = "2333")]
        port: u16,

        /// Update rate [ms], min=100.
        #[arg(short='i', long="sample-rate", default_value = "1000",
            value_parser = clap::value_parser!(u16).range(100..))]
        sample_rate_ms: u16,
    },
}

/// UI configuration.
#[derive(Debug, clap::Args)]
pub struct RunConfig {
    /// Update rate [ms], min=100.
    ///
    /// PowerMetrics samples at this rate.
    #[arg(short='i', long="sample-rate", default_value = "1000",
        value_parser = clap::value_parser!(u16).range(100..))]
    pub sample_rate_ms: u16,

    /// History buffer size.
    ///
    /// Number of recent samples to keep in history for each metric.
    #[arg(long, default_value = "128")]
    pub history_size: usize,

    /// ASCII code for labels, max: 255, default: green.
    #[arg(long, default_value = "2")]
    pub accent_color: u8,

    /// ASCII code, max=255, default: green.
    #[arg(long, default_value = "2")]
    pub gauge_fg_color: u8,

    /// ASCII code, max=255, default: white.
    #[arg(long, default_value = "7")]
    pub gauge_bg_color: u8,

    /// ASCII code, max=255, default: blue.
    #[arg(long, default_value = "4")]
    pub history_fg_color: u8,

    /// ASCII code, max=255, default: white.
    #[arg(long, default_value = "7")]
    pub history_bg_color: u8,

    /// Print metrics to stdout as JSON instead of running the UI.
    #[arg(long, default_value = "false")]
    pub json: bool,
}

impl RunConfig {
    /// Return colors.
    pub fn colors(&self) -> UiColors {
        UiColors {
            accent: self.accent_color,
            gauge_fg: self.gauge_fg_color,
            gauge_bg: self.gauge_bg_color,
            history_fg: self.history_fg_color,
            history_bg: self.history_bg_color,
        }
    }
}

/// Hold color configuration.
#[derive(Debug)]
pub struct UiColors {
    /// Accent color: ASCII code in 0~255.
    pub accent: u8,
    /// Gauge foreground color: ASCII code in 0~255.
    pub gauge_fg: u8,
    /// Gauge background color: ASCII code in 0~255.
    pub gauge_bg: u8,
    /// History foreground color: ASCII code in 0~255.
    pub history_fg: u8,
    /// History background color: ASCII code in 0~255.
    pub history_bg: u8,
}
