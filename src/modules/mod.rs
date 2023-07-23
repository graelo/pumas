//! Sources of data for the system.
//!
//! The following use parsers for external processes.
//! - powermetrics: CPU, GPU, ANE
//! - soc: num CPUs, num GPUs, CPU brand, etc

pub(crate) mod powermetrics;
pub(crate) mod soc;
