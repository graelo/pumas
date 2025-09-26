//! Ui.

pub(crate) mod main_screen;
pub(crate) mod startup_screen;
pub(crate) mod tab_cpu;
pub(crate) mod tab_gpu;
pub(crate) mod tab_memory;
pub(crate) mod tab_overview;
pub(crate) mod tab_soc;

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
