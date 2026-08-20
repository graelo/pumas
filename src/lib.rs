#![warn(missing_docs)]

//! Pumas is a power usage monitor for Apple Silicon Macs.
//!
//! End-user documentation—installation, usage, and metrics—is maintained in
//! the [README](https://github.com/graelo/pumas#readme).

mod backend;
pub mod config;
pub mod error;
mod metric_key;
mod metrics;
mod modules;
pub mod monitor;
mod ui;
mod units;

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, error::Error>;
