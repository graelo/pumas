#![warn(missing_docs)]

//! A power usage monitor for Apple Silicon.
//!
//! Version requirement: _rustc 1.64+_
//!
//! ## Features
//!
//! ![Screenshot](./images/screenshot.png)
//! TODO
//!
//! ## License
//!
//! Licensed under the [MIT License].
//!
//! ### Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in the work by you, as defined in the MIT license, shall
//! be licensed as MIT, without any additional terms or conditions.
//!
//! [MIT license]: http://opensource.org/licenses/MIT

mod app;
pub mod config;
pub mod error;
pub mod monitor;
mod parser;
mod signal;
mod ui;
mod units;

/// Result type for this crate.
pub type Result<T> = std::result::Result<T, error::Error>;
