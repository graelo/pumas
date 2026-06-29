//! Color theme for the iocraft UI.
//!
//! Maps the user-configurable [`UiColors`](crate::config::UiColors) (indexed
//! ANSI codes) onto `iocraft::Color` values. `iocraft::Color` re-exports
//! `crossterm::style::Color`, whose indexed variant is `Color::AnsiValue(u8)`
//! (the equivalent of ratatui's `Color::Indexed(u8)`).

use iocraft::prelude::Color;

use crate::config::UiColors;

/// Resolved theme colors, ready to hand to components. `Copy` so it can be
/// passed by value into props without ceremony.
///
/// Only `accent` is read by the minimal Phase 1 `PumasApp`; the gauge/history
/// colors are consumed by the leaf components in the Phase 2 tab views.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Theme {
    pub accent: Color,
    pub gauge_fg: Color,
    pub gauge_bg: Color,
    pub history_fg: Color,
    pub history_bg: Color,
}

impl From<&UiColors> for Theme {
    fn from(c: &UiColors) -> Self {
        Self {
            accent: Color::AnsiValue(c.accent),
            gauge_fg: Color::AnsiValue(c.gauge_fg),
            gauge_bg: Color::AnsiValue(c.gauge_bg),
            history_fg: Color::AnsiValue(c.history_fg),
            history_bg: Color::AnsiValue(c.history_bg),
        }
    }
}

impl Default for Theme {
    /// The documented CLI defaults: accent/gauge_fg green (2), gauge_bg/
    /// history_bg white (7), history_fg blue (4).
    fn default() -> Self {
        Self {
            accent: Color::AnsiValue(2),
            gauge_fg: Color::AnsiValue(2),
            gauge_bg: Color::AnsiValue(7),
            history_fg: Color::AnsiValue(4),
            history_bg: Color::AnsiValue(7),
        }
    }
}
