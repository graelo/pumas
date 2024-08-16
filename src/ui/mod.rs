//! User interface.

mod main_screen;
mod startup_screen;
mod tab_cpu;
mod tab_gpu;
mod tab_overview;
mod tab_soc;

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
