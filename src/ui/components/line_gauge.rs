//! Single-row line gauge (`━`, U+2501) used by the CPU/GPU rows.
//!
//! Matches the original `LineGauge`: an optional
//! left label, then a one-column gap, then the bar drawn with the THICK
//! horizontal symbol. Filled columns use `gauge_fg` as the glyph foreground,
//! unfilled columns use `gauge_bg`; there is no background fill (unlike the
//! block `Gauge`). Fill count is `floor(ratio * width)`.
//!
//! Activity rows pass a `"{:.1}%"` label; frequency rows pass `None` (just the
//! leading gap + line). Width is the number of `━` cells, an explicit prop for
//! the same reason as the block gauge.

use iocraft::prelude::*;

use super::{Cell, render_grid};

/// THICK horizontal line symbol (`symbols::line::THICK.horizontal`).
const LINE: char = '━';

/// Fully-owned line-gauge inputs.
#[derive(Clone, Debug)]
pub(crate) struct RenderedLineGauge {
    /// Fill ratio in `0.0..=1.0`.
    pub ratio: f64,
    /// Bar width in columns (number of `━` cells).
    pub width: usize,
    /// Optional left label (activity rows); `None` => frequency row.
    pub label: Option<String>,
    /// Filled-bar foreground (gauge_fg).
    pub fg: Color,
    /// Unfilled-bar foreground (gauge_bg).
    pub bg: Color,
}

impl RenderedLineGauge {
    fn cells(&self) -> Vec<Vec<Cell>> {
        let ratio = self.ratio.clamp(0.0, 1.0);
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let filled = ((ratio * self.width as f64).floor() as usize).min(self.width);

        let mut row: Vec<Cell> = Vec::new();
        // Optional label, then the one-column gap the original always inserts.
        if let Some(label) = &self.label {
            for ch in label.chars() {
                row.push(Cell::new(ch, Color::Reset, Color::Reset));
            }
        }
        row.push(Cell::new(' ', Color::Reset, Color::Reset));
        // The bar.
        for x in 0..self.width {
            let fg = if x < filled { self.fg } else { self.bg };
            row.push(Cell::new(LINE, fg, Color::Reset));
        }
        vec![row]
    }
}

#[derive(Default, Props)]
pub(crate) struct LineGaugeProps {
    pub line_gauge: Option<RenderedLineGauge>,
}

#[component]
pub(crate) fn LineGauge(props: &mut LineGaugeProps) -> impl Into<AnyElement<'static>> {
    let Some(lg) = props.line_gauge.take() else {
        return element! { View }.into_any();
    };
    render_grid(lg.cells())
}
