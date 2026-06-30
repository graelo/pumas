//! Multi-row vertical sparkline (`▁▂▃▄▅▆▇█`), matching the original `Sparkline`.
//!
//! The dead branch used only 4 glyphs on a single row; this rebuilds the full
//! 8-level (`NINE_LEVELS`) multi-row bars per `MIGRATION.md` §5.
//!
//! Algorithm: one column per data point, `height` rows tall. For each column
//! value `v` (clamped to `max`), the total bar height in eighths is
//! `e = round(v / max * height * 8)`. For row `r` (0 = top), the cell shows the
//! glyph for `clamp(e - 8*(height-1-r), 0, 8)`, where level 0 is a space and
//! levels 1..=8 map to `▁▂▃▄▅▆▇█`. Color: `history_fg` on `history_bg`.
//!
//! The single-row case (`height == 1`, used by CPU/GPU per-core rows) falls out
//! naturally as `clamp(round(v/max*8), 0, 8)`.

use iocraft::prelude::*;

use super::{Cell, render_grid};

/// `NINE_LEVELS`: index 0 = empty, 1..=8 = increasing block heights.
const BARS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Fully-owned sparkline inputs (owned `Vec<u64>`, no borrows).
#[derive(Clone, Debug)]
pub(crate) struct RenderedSparkline {
    /// One value per column (already trimmed to the last N by the backend).
    pub data: Vec<u64>,
    /// Scaling ceiling (`spark_max`; overshoot already applied upstream).
    pub max: u64,
    /// Number of rows.
    pub height: usize,
    /// Bar foreground (history_fg).
    pub fg: Color,
    /// Background (history_bg).
    pub bg: Color,
}

impl RenderedSparkline {
    /// Build the `height × data.len()` cell grid (top row first).
    fn cells(&self) -> Vec<Vec<Cell>> {
        let height = self.height.max(1);
        let max = self.max.max(1); // avoid div-by-zero; max 0 -> all empty

        // Per-column total height in eighths.
        let eighths: Vec<i64> = self
            .data
            .iter()
            .map(|&v| {
                let v = v.min(self.max) as f64;
                #[expect(clippy::cast_possible_truncation)]
                let e = (v / max as f64 * height as f64 * 8.0).round() as i64;
                e
            })
            .collect();

        (0..height)
            .map(|row| {
                eighths
                    .iter()
                    .map(|&e| {
                        let level = (e - 8 * (height - 1 - row) as i64).clamp(0, 8) as usize;
                        Cell::new(BARS[level], self.fg, self.bg)
                    })
                    .collect()
            })
            .collect()
    }
}

#[derive(Default, Props)]
pub(crate) struct SparklineProps {
    pub sparkline: Option<RenderedSparkline>,
}

#[component]
pub(crate) fn Sparkline(props: &mut SparklineProps) -> impl Into<AnyElement<'static>> {
    let Some(s) = props.sparkline.take() else {
        return element! { View }.into_any();
    };
    if s.data.is_empty() {
        return element! { View }.into_any();
    }
    render_grid(s.cells())
}
