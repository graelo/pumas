//! Block-bar gauge with a centered `NN%` label (Overview tab).
//!
//! Faithful re-implementation of the original `Gauge` widget
//! (`render_gauge`), which the dead branch did NOT match (it appended
//! the label after the bar instead of centering it). Semantics:
//!
//! - The bar is `height` rows tall. The original Overview wraps the gauge in a
//!   borderless titled `Block`, whose title consumes the top row, leaving a
//!   single bar row (`GAUGE_HEIGHT = 2` = title row + 1 bar row, MIGRATION.md
//!   D8). The Overview view therefore renders the title as a separate `Text`
//!   and drives the gauge at `height = 1`. The Phase 0 de-risk snapshots used
//!   `height = 2`.
//! - `end = round(ratio * width)` columns are filled with `█` (full block),
//!   foreground `gauge_fg` on background `gauge_bg`; the remaining columns are
//!   spaces showing `gauge_bg`.
//! - The label `"{NN}%"` (integer percent) is centered horizontally at
//!   `label_col = (width - label_width) / 2` and vertically on row
//!   `height / 2` (the single row when height is 1, the bottom row for height
//!   2), overlaid on the bar.
//!
//! Width is an explicit prop: iocraft computes flex sizes only *after* element
//! construction, so a component cannot know its allocated width at build time.
//! The frontend (Phase 2) derives the width from the terminal size + layout.

use iocraft::prelude::*;

use super::{Cell, render_grid};

/// Fully-owned gauge inputs (no borrows — `'static` element output).
#[derive(Clone, Debug)]
pub(crate) struct RenderedGauge {
    /// Fill ratio in `0.0..=1.0`.
    pub ratio: f64,
    /// Bar width in columns.
    pub width: usize,
    /// Bar height in rows (1 for the Overview, 2 for the Phase 0 spikes).
    pub height: usize,
    /// Filled-block foreground (gauge_fg).
    pub fg: Color,
    /// Background of the whole bar (gauge_bg).
    pub bg: Color,
}

impl RenderedGauge {
    /// Build the `height × width` cell grid, faithfully.
    fn cells(&self) -> Vec<Vec<Cell>> {
        let width = self.width;
        let height = self.height.max(1);
        let ratio = self.ratio.clamp(0.0, 1.0);

        // Integer-percent label, e.g. ratio 0.066 -> "7%".
        let label: Vec<char> = format!("{}%", (ratio * 100.0).round()).chars().collect();
        let label_w = label.len().min(width);
        let label_col = (width - label_w) / 2;
        let label_row = height / 2;

        // `end` = number of filled columns (no-unicode path: round).
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let end = ((ratio * width as f64).round() as usize).min(width);

        (0..height)
            .map(|row| {
                (0..width)
                    .map(|x| {
                        let in_label =
                            row == label_row && x >= label_col && x < label_col + label_w;
                        if in_label {
                            // Label glyph: default fg (matches the original label
                            // span style), background follows filled/unfilled.
                            let bg = if x < end { self.fg } else { self.bg };
                            Cell::new(label[x - label_col], Color::Reset, bg)
                        } else if x < end {
                            Cell::new('█', self.fg, self.bg)
                        } else {
                            Cell::new(' ', self.fg, self.bg)
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

#[derive(Default, Props)]
pub(crate) struct GaugeProps {
    pub gauge: Option<RenderedGauge>,
}

#[component]
pub(crate) fn Gauge(props: &mut GaugeProps) -> impl Into<AnyElement<'static>> {
    let Some(g) = props.gauge.take() else {
        return element! { View }.into_any();
    };
    render_grid(g.cells())
}
