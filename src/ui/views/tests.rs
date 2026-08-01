//! Snapshot tests: chrome (title bar, tab bar), splash, and Overview.
//!
//! The fixture mirrors the M5 Max in `~/Downloads/screenshots/2.tab-overview.png`
//! (two P-clusters paired + a single S-cluster, no separate E-clusters). All
//! snapshots capture **glyph/layout only, not color**; color
//! parity is guarded by `snapshot::tests::gauge_and_sparkline_colors` and the
//! live smoke check.

use crate::backend::frame::{
    ColorRole, CpuCluster, CpuFrame, CpuRow, FreqTable, GpuFrame, MemLine, MemSpan, MemoryFrame,
    Meter, OverviewFrame, SocRows, SparkText, Thermals,
};
use crate::ui::components::tab_bar::tab_bar;
use crate::ui::components::title_bar::title_bar;
use crate::ui::snapshot::{assert_snapshot, render_to_text};
use crate::ui::theme::Theme;
use crate::ui::views::cpu::cpu;
use crate::ui::views::gpu::gpu;
use crate::ui::views::memory::memory;
use crate::ui::views::overview::overview;
use crate::ui::views::soc::soc;
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

// ─── CPU fixture (mirrors screenshots/3.tab-cpu.png) ─────────────────────────

/// Build one CPU core row. Sparklines are flat (`level` repeated 8×) for a
/// deterministic golden; geometry/alignment is what the snapshot guards.
fn cpu_core(id: u16, act: f64, freq_mhz: &str, freq_pct: f64) -> CpuRow {
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let act_level = act.round() as u64;
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let freq_level = freq_pct.round() as u64;
    CpuRow {
        id_label: format!("{id:2} -"),
        act_ratio: act / 100.0,
        act_label: format!("{act:.1}%"),
        act_spark: vec![act_level; 8],
        act_spark_max: 105,
        freq_value: freq_mhz.to_string(),
        freq_ratio: freq_pct / 100.0,
        freq_label: format!("{freq_pct:3.0}%"),
        freq_spark: vec![freq_level; 8],
        freq_spark_max: 105,
    }
}

fn cpu_fixture() -> CpuFrame {
    let p0 = CpuCluster {
        title: " P0-Cluster: ".to_string(),
        cpus: (0..6).map(|id| cpu_core(id, 0.0, "0 MHz", 0.0)).collect(),
    };
    let p1 = CpuCluster {
        title: " P1-Cluster: ".to_string(),
        cpus: vec![
            cpu_core(6, 15.4, "1447 MHz", 3.0),
            cpu_core(7, 7.8, "1404 MHz", 2.0),
            cpu_core(8, 7.8, "1426 MHz", 3.0),
            cpu_core(9, 2.9, "1486 MHz", 5.0),
            cpu_core(10, 1.9, "1514 MHz", 6.0),
            cpu_core(11, 0.0, "1500 MHz", 5.0),
        ],
    };
    let s = CpuCluster {
        title: " S-Cluster: ".to_string(),
        cpus: vec![
            cpu_core(12, 1.9, "2138 MHz", 25.0),
            cpu_core(13, 1.9, "2271 MHz", 29.0),
            cpu_core(14, 0.0, "2165 MHz", 26.0),
            cpu_core(15, 0.0, "0 MHz", 0.0),
            cpu_core(16, 0.0, "0 MHz", 0.0),
            cpu_core(17, 0.0, "0 MHz", 0.0),
        ],
    };
    CpuFrame {
        clusters: vec![p0, p1, s],
        freq_table: FreqTable {
            rows: vec![
                (
                    "P-Cluster:".to_string(),
                    "1344 1644 1992 2304 2652 2964 3240 3504 3696 3876 4044 4176 4284 4308 4380"
                        .to_string(),
                ),
                (
                    "S-Cluster:".to_string(),
                    "1308 1620 1980 2292 2580 2880 3180 3432 3648 3828 3984 4104 4188 4236 4284 4308 4332 4428 4512 4608"
                        .to_string(),
                ),
                (String::new(), String::new()),
                (
                    "Note:".to_string(),
                    "Hardware-wise, CPUs quickly shift between the above frequencies.".to_string(),
                ),
            ],
        },
    }
}

#[test]
fn cpu_snapshot() {
    let theme = Theme::default();
    let f = cpu_fixture();
    let el = cpu(&f, 120, theme);
    assert_snapshot("cpu", &render_to_text(el, 120));
}

// ─── GPU fixture (mirrors screenshots/4.tab-gpu.png) ─────────────────────────

fn gpu_fixture() -> GpuFrame {
    GpuFrame {
        act_ratio: 0.011,
        act_label: "1.1%".to_string(),
        act_spark: vec![1; 8],
        act_spark_max: 105,
        freq_value: "338 MHz".to_string(),
        freq_ratio: 0.0,
        freq_label: format!("{:3.0}%", 0.0),
        freq_spark: vec![0; 8],
        freq_spark_max: 105,
        power_value: "9.73 mW".to_string(),
        power_spark: vec![1; 8],
        power_spark_max: 105,
        peak_text: "Peak: 10.9 % | 121.64 mW".to_string(),
        thermals: Thermals {
            pressure: "Nominal".to_string(),
            is_nominal: true,
        },
        freq_table: FreqTable {
            rows: vec![
                (
                    "GPU:".to_string(),
                    " 338  486  636  796  888  988 1084 1182 1278 1374 1470 1578 1620".to_string(),
                ),
                (String::new(), String::new()),
                (
                    "Note:".to_string(),
                    "Hardware-wise, GPUs quickly shift between the above frequencies.".to_string(),
                ),
            ],
        },
    }
}

#[test]
fn gpu_snapshot() {
    let theme = Theme::default();
    let f = gpu_fixture();
    let el = gpu(&f, 120, theme);
    assert_snapshot("gpu", &render_to_text(el, 120));
}

// ─── Memory fixture (mirrors screenshots/5.tab-memory.png) ───────────────────

fn ml(spans: Vec<(&str, ColorRole)>) -> MemLine {
    MemLine {
        spans: spans
            .into_iter()
            .map(|(t, role)| MemSpan {
                text: t.to_string(),
                role,
            })
            .collect(),
    }
}

fn blank_line() -> MemLine {
    MemLine { spans: Vec::new() }
}

fn memory_fixture() -> MemoryFrame {
    use ColorRole::{Accent, Default as Def, GaugeFg, HistoryFg};
    MemoryFrame {
        vm_lines: vec![
            ml(vec![
                ("Physical Memory Total: ", Accent),
                ("121.84 GB", Def),
            ]),
            blank_line(),
            ml(vec![("═══ ACTIVITY MONITOR CALCULATION ═══", Accent)]),
            ml(vec![
                ("App Memory (Anonymous): ", GaugeFg),
                ("46.88 GB", Def),
            ]),
            ml(vec![
                ("Wired Memory:         + ", GaugeFg),
                ("4.32 GB", Def),
            ]),
            ml(vec![
                ("Compressed:           + ", GaugeFg),
                ("1.62 GB", Def),
            ]),
            ml(vec![("                      ─────────", HistoryFg)]),
            ml(vec![
                ("Memory Used Total:      ", Accent),
                ("52.82 GB", Def),
            ]),
            blank_line(),
            ml(vec![("═══ OTHER MEMORY CATEGORIES ═══", HistoryFg)]),
            ml(vec![("Cached Files:         ", GaugeFg), ("20.92 GB", Def)]),
            ml(vec![("Free:                 ", GaugeFg), ("53.21 GB", Def)]),
            ml(vec![("Active:               ", GaugeFg), ("33.15 GB", Def)]),
            ml(vec![("Inactive:             ", GaugeFg), ("31.16 GB", Def)]),
        ],
        sysinfo_lines: vec![
            ml(vec![
                ("RAM Used: ", Accent),
                ("41.3 % = 52.8 GiB / 128.0 GiB (41.3%)", Def),
            ]),
            ml(vec![
                ("Swap Used: ", Accent),
                ("0.0 % = 0.0 B / 0.0 B", Def),
            ]),
            blank_line(),
            ml(vec![
                ("Note: ", HistoryFg),
                (
                    "RAM Used now uses vm_stat for Activity Monitor compatibility",
                    Def,
                ),
            ]),
        ],
    }
}

#[test]
fn memory_snapshot() {
    let theme = Theme::default();
    let f = memory_fixture();
    let el = memory(&f, 120, theme);
    assert_snapshot("memory", &render_to_text(el, 120));
}

// ─── SoC fixture (mirrors screenshots/6.tab-soc.png) ─────────────────────────

fn soc_fixture() -> SocRows {
    SocRows {
        rows: vec![
            ("SoC brand name:".to_string(), "Apple M5 Max".to_string()),
            ("CPU cores:".to_string(), "18".to_string()),
            ("- Efficiency cores:".to_string(), "12".to_string()),
            ("- Performance cores:".to_string(), "6".to_string()),
            ("GPU cores:".to_string(), "40".to_string()),
            ("Max CPU power:".to_string(), "78 W".to_string()),
            ("Max GPU power:".to_string(), "75 W".to_string()),
            ("Max ANE power:".to_string(), "12 W".to_string()),
        ],
    }
}

#[test]
fn soc_snapshot() {
    let theme = Theme::default();
    let rows = soc_fixture();
    let el = soc(&rows, 120, theme);
    assert_snapshot("soc", &render_to_text(el, 120));
}
