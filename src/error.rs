//! This crate's error type.

use std::io;

/// Describes all errors from this crate.
///
/// - errors during parsing.
/// - errors reported other crates.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error parsing SocInfo.
    #[error("socinfo parsing error: `{0}`")]
    SocInfoParsingError(String),

    /// Error parsing integer.
    #[error("integer parsing error: `{source}`")]
    ParseIntError {
        #[from]
        /// Source error.
        source: std::num::ParseIntError,
    },

    /// Error parsing a plist.
    #[error("plist parsing error: `{0}`")]
    PlistParsingError(String),

    /// Misalignment of CPU IDs between powermetrics and the sysinfo crate.
    #[error("cpu id misalignment: `{0}`")]
    MisalignedCpuId(String),

    /// Error converting a string to utf8.
    #[error("utf8 conversion error: `{source}`")]
    Utf8ConversionError {
        #[from]
        /// Source error.
        source: std::string::FromUtf8Error,
    },

    /// Some IO error.
    #[error("failed with io: `{source}`")]
    Io {
        #[from]
        /// Source error.
        source: io::Error,
    },

    /// Error spawning powermetrics subprocess.
    #[error("failed to spawn powermetrics: `{0}`")]
    PowermetricsSpawn(io::Error),

    /// Error accessing powermetrics stdout.
    #[error("powermetrics stdout not available")]
    PowermetricsStdout,

    /// Error killing powermetrics subprocess.
    #[error("failed to kill powermetrics: `{0}`")]
    PowermetricsKill(io::Error),
}
