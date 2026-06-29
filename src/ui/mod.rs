//! Ui.

// --- ratatui frontend (superseded; deleted in Phase 3) --------------------
// The iocraft `PumasApp` replaces `draw`/`main_screen`/`startup_screen`/`tab_*`,
// but they stay compiled until Phase 3 removes them. They have no live caller
// now, hence the module-scoped `dead_code` allows.
#[allow(dead_code)]
pub(crate) mod main_screen;
#[allow(dead_code)]
pub(crate) mod startup_screen;
#[allow(dead_code)]
pub(crate) mod tab_cpu;
#[allow(dead_code)]
pub(crate) mod tab_gpu;
#[allow(dead_code)]
pub(crate) mod tab_memory;
#[allow(dead_code)]
pub(crate) mod tab_overview;
#[allow(dead_code)]
pub(crate) mod tab_soc;

// --- iocraft frontend -----------------------------------------------------
pub(crate) mod app_root;
pub(crate) mod theme;

// Leaf components (gauge/sparkline/line_gauge/panel) are consumed by the tab
// views in Phase 2; for now they are exercised only by the snapshot tests.
#[allow(dead_code)]
pub(crate) mod components;
#[cfg(test)]
mod snapshot;

use ratatui::Frame;

use crate::app::App;

/// Main UI entry point (ratatui; superseded by `PumasApp`, removed in Phase 3).
#[allow(dead_code)]
pub(crate) fn draw(f: &mut Frame, app: &mut App) {
    if app.metrics.is_none() {
        startup_screen::draw(f);
    } else {
        main_screen::draw(f, app, f.area());
    }
}
