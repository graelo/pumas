//! Phase 2A snapshot tests: chrome (title bar, tab bar), splash, and Overview.
//!
//! The fixture mirrors the M5 Max in `~/Downloads/screenshots/2.tab-overview.png`
//! (two P-clusters paired + a single S-cluster, no separate E-clusters). All
//! snapshots capture **glyph/layout only, not color** (MIGRATION.md §7.8); color
//! parity is guarded by `snapshot::tests::gauge_and_sparkline_colors` and the
//! live smoke check.

use crate::backend::frame::{Meter, OverviewFrame, SparkText, Thermals};
use crate::ui::components::tab_bar::tab_bar;
use crate::ui::components::title_bar::title_bar;
use crate::ui::snapshot::{assert_snapshot, render_to_text};
use crate::ui::theme::Theme;
use crate::ui::views::overview::overview;
use crate::ui::views::splash::splash;

/// Deterministic ascending sparkline data (length `n`, ceiling `max`).
fn ramp(n: usize, max: u64) -> Vec<u64> {
    if n <= 1 {
        return vec![max; n];
    }
    (0..n).map(|i| (i as u64 * max) / (n as u64 - 1)).collect()
}

fn meter(title: &str, ratio: f64) -> Meter {
    Meter {
        title: title.to_string(),
        ratio,
        spark: ramp(120, 100),
        spark_max: 105,
    }
}

/// Build the M5 Max Overview fixture matching the reference screenshot.
fn fixture() -> OverviewFrame {
    OverviewFrame {
        cpu_clusters_title: " CPU Clusters: 178.98 mW (peak: 2.94 W) ".to_string(),
        e_meters: vec![],
        p_meters: vec![
            meter("P0-Cluster: 0.0 % @ 0 MHz (peak: 0.0 %)", 0.0),
            meter("P1-Cluster: 6.6 % @ 1437 MHz (peak: 9.8 %)", 0.066),
        ],
        s_meters: vec![meter("S-Cluster: 0.6 % @ 1991 MHz (peak: 10.8 %)", 0.006)],
        gpu: meter(
            "GPU: 5.8 % @ 338 MHz | 72.95 mW (peak: 10.2 % | 121.64 mW)",
            0.058,
        ),
        ane: meter("ANE: 0.0 % | 0.00 W (peak: 0.0 % | 0.00 W)", 0.0),
        package: SparkText {
            title: "CPU+GPU+ANE: 251.93 mW (peak: 2.97 W)".to_string(),
            spark: ramp(120, 300),
            spark_max: 300,
        },
        thermals: Thermals {
            pressure: "Nominal".to_string(),
            is_nominal: true,
        },
        ram: meter(
            "Memory Used: 41.4 % = 53.0 GiB / 128.0 GiB (peak: 41.5 % = 53.1 GiB)",
            0.414,
        ),
        swap: meter("SWAP: 0.0 % = 0.0 B / 0.0 B (peak: 0.0 B)", 0.0),
    }
}

#[test]
fn title_bar_snapshot() {
    let theme = Theme::default();
    let el = title_bar(
        "Pumas v0.5.0".to_string(),
        " Apple M5 Max (cores: 12E+6P+40GPU) ".to_string(),
        theme.accent,
        120,
    );
    assert_snapshot("title_bar", &render_to_text(el, 120));
}

#[test]
fn tab_bar_overview_active_snapshot() {
    let theme = Theme::default();
    let el = tab_bar(0, theme.accent, 120);
    assert_snapshot("tab_bar_overview", &render_to_text(el, 120));
}

#[test]
fn splash_snapshot() {
    let el = splash(120, 40);
    assert_snapshot("splash", &render_to_text(el, 120));
}

#[test]
fn overview_snapshot() {
    let theme = Theme::default();
    let f = fixture();
    let el = overview(&f, 120, theme);
    assert_snapshot("overview", &render_to_text(el, 120));
}

/// At 120 columns the RAM title is clipped at the half boundary; a wider
/// terminal shows it in full and confirms the paired-cluster geometry scales.
#[test]
fn overview_wide_snapshot() {
    let theme = Theme::default();
    let f = fixture();
    let el = overview(&f, 160, theme);
    assert_snapshot("overview_wide", &render_to_text(el, 160));
}
