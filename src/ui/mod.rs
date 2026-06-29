//! Ui.

pub(crate) mod main_screen;
pub(crate) mod startup_screen;
pub(crate) mod tab_cpu;
pub(crate) mod tab_gpu;
pub(crate) mod tab_memory;
pub(crate) mod tab_overview;
pub(crate) mod tab_soc;

// --- iocraft migration (Phase 0 spike) ------------------------------------
// Not yet wired into the live (ratatui) binary; exercised only by the snapshot
// tests in `snapshot.rs`. The `dead_code` allow is scoped to this transitional
// phase and removed once the iocraft frontend consumes them (Phase 1+).
#[allow(dead_code)]
pub(crate) mod components;
#[cfg(test)]
mod snapshot;
#[allow(dead_code)]
pub(crate) mod theme;

use ratatui::Frame;

use crate::app::App;

/// Main UI entry point.
pub(crate) fn draw(f: &mut Frame, app: &mut App) {
    if app.metrics.is_none() {
        startup_screen::draw(f);
    } else {
        main_screen::draw(f, app, f.area());
    }
}
