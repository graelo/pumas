//! The `Frame` data-plane contract (MIGRATION.md §4).
//!
//! A `Frame` is one owned, `Clone` snapshot the collector ships to the frontend
//! per sample. It carries everything the UI needs **already prepared**: every
//! string is pre-formatted (via [`crate::units`]) and every sparkline slice +
//! scaling factor is pre-computed. The frontend pairs/lays-out but does no
//! history math and no string formatting.
//!
//! Widths/heights are deliberately absent — those are a frontend concern
//! (MIGRATION.md §7.9). The sparkline `spark` vectors carry data only; the view
//! trims/scales to its allocated geometry.

use crate::{modules::soc::SocInfo, units};

/// A gauge + its sparkline, fully prepared.
///
/// `ratio` drives the gauge fill, `title` is the pre-formatted gauge/label line,
/// and `spark`/`spark_max` drive the sparkline. There is no peak field — the
/// peak is already baked into `title`.
#[derive(Clone)]
pub(crate) struct Meter {
    /// Pre-formatted gauge label, e.g. `"E-Cluster: 21.8 % @ 973 MHz (peak: 22.1 %)"`.
    pub title: String,
    /// Gauge fill ratio, `0.0..=1.0`.
    pub ratio: f64,
    /// Sparkline data (full history for Overview; last-N where a tab fixes N).
    pub spark: Vec<u64>,
    /// Sparkline scaling ceiling, overshoot already applied (MIGRATION.md §4.4).
    pub spark_max: u64,
}

/// A text title (no gauge) plus a sparkline. Used by the Overview Package block.
#[derive(Clone)]
pub(crate) struct SparkText {
    /// Pre-formatted title, e.g. `"CPU+GPU+ANE: 130.55 mW (peak: 6.48 W)"`.
    pub title: String,
    /// Sparkline data (full history).
    pub spark: Vec<u64>,
    /// Sparkline scaling ceiling = `signal.max` (Package has **no** overshoot).
    pub spark_max: u64,
}

/// Thermal-pressure indicator: the text plus whether it is nominal (accent) or
/// not (Yellow).
#[derive(Clone)]
pub(crate) struct Thermals {
    /// Pressure text, e.g. `"Nominal"`.
    pub pressure: String,
    /// `true` => accent color; `false` => Yellow.
    pub is_nominal: bool,
}

/// The Overview tab snapshot.
#[derive(Clone)]
pub(crate) struct OverviewFrame {
    /// Panel border title, e.g. `" CPU Clusters: 119.67 mW (peak: 6.42 W) "`.
    pub cpu_clusters_title: String,
    /// One meter per E-cluster, in order (frontend pairs via `chunks(2)`).
    pub e_meters: Vec<Meter>,
    /// One meter per P-cluster.
    pub p_meters: Vec<Meter>,
    /// One meter per S-cluster (M5 Pro/Max and above).
    pub s_meters: Vec<Meter>,
    /// GPU gauge + sparkline.
    pub gpu: Meter,
    /// ANE gauge + sparkline (ratio = `ane_w / max_ane_w`).
    pub ane: Meter,
    /// Package power text + sparkline (no overshoot).
    pub package: SparkText,
    /// Thermal pressure.
    pub thermals: Thermals,
    /// RAM gauge + sparkline.
    pub ram: Meter,
    /// Swap gauge + sparkline.
    pub swap: Meter,
}

/// A single CPU core row on the CPU tab.
#[derive(Clone)]
pub(crate) struct CpuRow {
    /// Left accent label, `"{id:2} -"`.
    pub id_label: String,
    /// Activity gauge fill ratio.
    pub act_ratio: f64,
    /// Activity label, `"{:.1}%"`.
    pub act_label: String,
    /// Activity sparkline (last 8).
    pub act_spark: Vec<u64>,
    /// Activity sparkline ceiling (`1.05 * max`).
    pub act_spark_max: u64,
    /// Frequency value, e.g. `"972 MHz"`.
    pub freq_value: String,
    /// Frequency gauge fill ratio.
    pub freq_ratio: f64,
    /// Frequency gauge's default label, `"{:3.0}%"`. The freq LineGauge has no
    /// explicit label, so the original draws this default (MIGRATION.md D9).
    pub freq_label: String,
    /// Frequency sparkline (last 8).
    pub freq_spark: Vec<u64>,
    /// Frequency sparkline ceiling (`1.05 * max`).
    pub freq_spark_max: u64,
}

/// A bordered CPU cluster block on the CPU tab.
#[derive(Clone)]
pub(crate) struct CpuCluster {
    /// Block border title, `" {name}: "`.
    pub title: String,
    /// One row per CPU core in the cluster.
    pub cpus: Vec<CpuRow>,
}

/// 2-column DVFM frequency table shared by the CPU/GPU tabs. Rows are
/// `(left_label, right_value)` with the right value rendered bold.
#[derive(Clone)]
pub(crate) struct FreqTable {
    /// Table rows.
    pub rows: Vec<(String, String)>,
}

/// The CPU tab snapshot.
#[derive(Clone)]
pub(crate) struct CpuFrame {
    /// Clusters in order: E…, then P…, then S….
    pub clusters: Vec<CpuCluster>,
    /// DVFM frequency table.
    pub freq_table: FreqTable,
}

/// The GPU tab snapshot.
#[derive(Clone)]
pub(crate) struct GpuFrame {
    /// Activity gauge fill ratio.
    pub act_ratio: f64,
    /// Activity label, `"{:.1}%"`.
    pub act_label: String,
    /// Activity sparkline (last 8).
    pub act_spark: Vec<u64>,
    /// Activity sparkline ceiling (`1.05 * max`).
    pub act_spark_max: u64,
    /// Frequency value, e.g. `"444 MHz"`.
    pub freq_value: String,
    /// Frequency gauge fill ratio.
    pub freq_ratio: f64,
    /// Frequency gauge's default label, `"{:3.0}%"`. The freq LineGauge has no
    /// explicit label, so the original draws this default (MIGRATION.md D9).
    pub freq_label: String,
    /// Frequency sparkline (last 8).
    pub freq_spark: Vec<u64>,
    /// Frequency sparkline ceiling (`1.05 * max`).
    pub freq_spark_max: u64,
    /// Power value, e.g. `"10.88 mW"`.
    pub power_value: String,
    /// Power sparkline (last 8).
    pub power_spark: Vec<u64>,
    /// Power sparkline ceiling (`1.05 * max`).
    pub power_spark_max: u64,
    /// Peak text, `"Peak: {p1} | {w}"`.
    pub peak_text: String,
    /// Thermal pressure.
    pub thermals: Thermals,
    /// DVFM frequency table.
    pub freq_table: FreqTable,
}

/// Theme color role for a [`MemSpan`] (mapped to a concrete color in the
/// frontend theme, MIGRATION.md §6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ColorRole {
    /// Accent color.
    Accent,
    /// Gauge foreground.
    GaugeFg,
    /// History foreground.
    HistoryFg,
    /// Terminal default.
    Default,
}

/// A colored text span on a Memory-tab line.
#[derive(Clone)]
pub(crate) struct MemSpan {
    /// Span text.
    pub text: String,
    /// Theme color role.
    pub role: ColorRole,
}

/// A single Memory-tab line, made of one or more colored spans.
#[derive(Clone)]
pub(crate) struct MemLine {
    /// Spans composing the line (empty = blank line).
    pub spans: Vec<MemSpan>,
}

/// The Memory tab snapshot. `vm_stat` is collected in the backend (D2), so the
/// lines arrive pre-formatted and pre-colored.
#[derive(Clone)]
pub(crate) struct MemoryFrame {
    /// Activity-Monitor-compatible VM statistics block.
    pub vm_lines: Vec<MemLine>,
    /// Sysinfo statistics block.
    pub sysinfo_lines: Vec<MemLine>,
}

/// One owned, `Clone` snapshot shipped per sample. SoC info and the header are
/// session-static and intentionally **not** carried here (see
/// [`RenderedHeader`] / [`render_soc_rows`]).
#[derive(Clone)]
pub(crate) struct Frame {
    /// Overview tab.
    pub overview: OverviewFrame,
    /// CPU tab.
    pub cpu: CpuFrame,
    /// GPU tab.
    pub gpu: GpuFrame,
    /// Memory tab.
    pub memory: MemoryFrame,
}

/// Session-static title-bar strings (built once, never per-frame).
#[derive(Clone, Default)]
pub(crate) struct RenderedHeader {
    /// Left side, `"Pumas v{version}"`.
    pub program_name: String,
    /// Right side, `" {brand} (cores: {E}E+{P}P+{GPU}GPU) "`.
    pub machine_desc: String,
}

/// Build the session-static header from the SoC info (mirrors
/// `main_screen.rs` title-line formatting).
pub(crate) fn render_header(soc: &SocInfo) -> RenderedHeader {
    RenderedHeader {
        program_name: format!("Pumas v{}", env!("CARGO_PKG_VERSION")),
        machine_desc: format!(
            " {} (cores: {}E+{}P+{}GPU) ",
            soc.cpu_brand_name,
            soc.num_efficiency_cores,
            soc.num_performance_cores,
            soc.num_gpu_cores
        ),
    }
}

/// Session-static SoC tab rows (built once). Mirrors `tab_soc.rs`.
#[derive(Clone, Default)]
pub(crate) struct SocRows {
    /// `(left_label, right_value)` rows; right value rendered bold.
    pub rows: Vec<(String, String)>,
}

/// Build the SoC tab rows from the SoC info.
pub(crate) fn render_soc_rows(soc: &SocInfo) -> SocRows {
    SocRows {
        rows: vec![
            ("SoC brand name:".into(), soc.cpu_brand_name.clone()),
            ("CPU cores:".into(), format!("{}", soc.num_cpu_cores)),
            (
                "- Efficiency cores:".into(),
                format!("{}", soc.num_efficiency_cores),
            ),
            (
                "- Performance cores:".into(),
                format!("{}", soc.num_performance_cores),
            ),
            ("GPU cores:".into(), format!("{}", soc.num_gpu_cores)),
            ("Max CPU power:".into(), units::watts(soc.max_cpu_w)),
            ("Max GPU power:".into(), units::watts(soc.max_gpu_w)),
            ("Max ANE power:".into(), units::watts(soc.max_ane_w)),
        ],
    }
}
