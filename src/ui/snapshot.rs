//! Test-only headless snapshot harness for the iocraft UI.
//!
//! `render_to_text` renders an element to a fixed width and returns the
//! **plain-text** canvas (iocraft's `Canvas: Display` writes glyphs only, no
//! ANSI — see `canvas.rs::write_impl(.., false, ..)`).
//!
//! ⚠️ Plain-text snapshots capture **layout and glyphs only, NOT color**
//!. Color parity is verified separately by
//! [`tests::gauge_and_sparkline_colors`], which inspects `Canvas` cell styles.

use iocraft::prelude::*;

/// Render an element at `width` columns to plain text (no ANSI/color).
pub(crate) fn render_to_text(mut el: AnyElement<'static>, width: usize) -> String {
    el.render(Some(width)).to_string()
}

/// Compare `actual` against the golden at `tests/snapshots/<name>.snap`.
///
/// Set `UPDATE_SNAPSHOTS=1` to (re)write goldens instead of asserting.
pub(crate) fn assert_snapshot(name: &str, actual: &str) {
    let path = format!("{}/tests/snapshots/{name}.snap", env!("CARGO_MANIFEST_DIR"));
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::write(&path, actual).expect("write golden");
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read golden {path}: {e} (run with UPDATE_SNAPSHOTS=1)"));
    assert_eq!(actual, &expected, "snapshot mismatch for {name}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::gauge::{Gauge, RenderedGauge};
    use crate::ui::components::line_gauge::{LineGauge, RenderedLineGauge};
    use crate::ui::components::panel::panel;
    use crate::ui::components::sparkline::{RenderedSparkline, Sparkline};
    use crate::ui::theme::Theme;

    /// Deterministic linear ramp `0..=max` across `n` columns.
    fn ramp(n: usize, max: u64) -> Vec<u64> {
        if n <= 1 {
            return vec![max; n];
        }
        (0..n).map(|i| (i as u64 * max) / (n as u64 - 1)).collect()
    }

    // -- Sparklines ---------------------------------------------------------
    // NOTE: snapshots below capture glyph/layout parity only, not color.

    #[test]
    fn sparkline_multirow_h3() {
        let theme = Theme::default();
        let spark = RenderedSparkline {
            data: ramp(24, 105),
            max: 105,
            height: 3,
            fg: theme.history_fg,
            bg: theme.history_bg,
        };
        let el = element! { Sparkline(sparkline: Some(spark)) }.into_any();
        assert_snapshot("sparkline_h3", &render_to_text(el, 24));
    }

    #[test]
    fn sparkline_multirow_h9() {
        let theme = Theme::default();
        let spark = RenderedSparkline {
            data: ramp(24, 105),
            max: 105,
            height: 9,
            fg: theme.history_fg,
            bg: theme.history_bg,
        };
        let el = element! { Sparkline(sparkline: Some(spark)) }.into_any();
        assert_snapshot("sparkline_h9", &render_to_text(el, 24));
    }

    #[test]
    fn sparkline_single_row_h1() {
        let theme = Theme::default();
        let spark = RenderedSparkline {
            data: ramp(16, 105),
            max: 105,
            height: 1,
            fg: theme.history_fg,
            bg: theme.history_bg,
        };
        let el = element! { Sparkline(sparkline: Some(spark)) }.into_any();
        assert_snapshot("sparkline_h1", &render_to_text(el, 16));
    }

    // -- Gauge --------------------------------------------------------------

    #[test]
    fn gauge_centered_label() {
        let theme = Theme::default();
        let g = RenderedGauge {
            ratio: 0.41,
            width: 30,
            height: 2,
            fg: theme.gauge_fg,
            bg: theme.gauge_bg,
        };
        let el = element! { Gauge(gauge: Some(g)) }.into_any();
        assert_snapshot("gauge_41pct", &render_to_text(el, 30));
    }

    // -- Line gauge (CPU/GPU rows) ------------------------------------------

    #[test]
    fn line_gauge_activity_and_frequency() {
        let theme = Theme::default();
        // Activity row: a "{:.1}%" label, then gap, then the ━ bar.
        let act = element! {
            LineGauge(line_gauge: Some(RenderedLineGauge {
                ratio: 0.43,
                width: 20,
                label: Some("43.0%".to_string()),
                fg: theme.gauge_fg,
                bg: theme.gauge_bg,
            }))
        }
        .into_any();
        // Frequency row: no label, just the leading gap + bar.
        let freq = element! {
            LineGauge(line_gauge: Some(RenderedLineGauge {
                ratio: 0.21,
                width: 20,
                label: None,
                fg: theme.gauge_fg,
                bg: theme.gauge_bg,
            }))
        }
        .into_any();
        let stack = element! {
            View(flex_direction: FlexDirection::Column) {
                #(vec![act, freq])
            }
        }
        .into_any();
        assert_snapshot("line_gauge", &render_to_text(stack, 30));
    }

    // -- Bordered titled panel: gauge + 3-row sparkline ---------------------
    // Roughly an Overview cluster-cell width (62 cols outer, 60 inner).
    // Glyph/layout parity only; color is checked separately below.

    #[test]
    fn panel_with_gauge_and_sparkline() {
        let theme = Theme::default();
        let inner = 60usize;
        let gauge = element! {
            Gauge(gauge: Some(RenderedGauge {
                ratio: 0.066,
                width: inner,
                height: 2,
                fg: theme.gauge_fg,
                bg: theme.gauge_bg,
            }))
        }
        .into_any();
        let spark = element! {
            Sparkline(sparkline: Some(RenderedSparkline {
                data: ramp(inner, 105),
                max: 105,
                height: 3,
                fg: theme.history_fg,
                bg: theme.history_bg,
            }))
        }
        .into_any();
        let body = element! {
            View(flex_direction: FlexDirection::Column) {
                #(vec![gauge, spark])
            }
        }
        .into_any();
        let p = panel(
            " P1-Cluster: 6.6 % @ 1437 MHz (peak: 9.8 %) ",
            62,
            Color::Reset,
            body,
        );
        assert_snapshot("panel_cluster_cell", &render_to_text(p, 62));
    }

    // -- Color parity: inspect Canvas cell styles, not plain text ------

    #[test]
    fn gauge_and_sparkline_colors() {
        let theme = Theme::default();

        // Gauge: a filled `█` cell must carry gauge_fg as its foreground.
        let mut g = element! {
            Gauge(gauge: Some(RenderedGauge {
                ratio: 0.5,
                width: 10,
                height: 2,
                fg: theme.gauge_fg,
                bg: theme.gauge_bg,
            }))
        }
        .into_any();
        let canvas = g.render(Some(10));
        let cell = canvas.cell(0, 0).expect("gauge cell (0,0)");
        assert_eq!(cell.text(), Some("█"), "filled cell glyph");
        assert_eq!(
            cell.text_style().and_then(|s| s.color),
            Some(theme.gauge_fg),
            "filled gauge cell foreground must be gauge_fg",
        );

        // Sparkline: a non-empty bar cell must carry history_fg and the
        // history_bg background.
        let mut s = element! {
            Sparkline(sparkline: Some(RenderedSparkline {
                data: vec![105, 105, 105],
                max: 105,
                height: 1,
                fg: theme.history_fg,
                bg: theme.history_bg,
            }))
        }
        .into_any();
        let canvas = s.render(Some(3));
        let cell = canvas.cell(0, 0).expect("sparkline cell (0,0)");
        assert_eq!(cell.text(), Some("█"), "full bar glyph");
        assert_eq!(
            cell.text_style().and_then(|st| st.color),
            Some(theme.history_fg),
            "sparkline bar foreground must be history_fg",
        );
        assert_eq!(
            cell.background_color,
            Some(theme.history_bg),
            "sparkline background must be history_bg",
        );
    }
}
