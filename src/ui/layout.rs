//! Frontend geometry for the tab views (MIGRATION.md §7.9).
//!
//! iocraft cannot expose a flex-allocated width at element-*build* time, so the
//! leaf widgets (gauge/sparkline/line_gauge) take explicit dimensions. This
//! module is the single place that mirrors the original constraint math, turning a
//! terminal `width` plus the [`OverviewFrame`] shape into the per-cell widths
//! and heights the Overview view passes down. The backend [`Frame`] stays
//! width-free; all pixel geometry lives here.
//!
//! [`Frame`]: crate::backend::frame::Frame

use crate::backend::frame::OverviewFrame;

/// 2-column gap between paired halves (the original `Constraint::Length(2)` in
/// `tab_overview.rs`).
pub(crate) const GAP: usize = 2;

/// the original `GAUGE_HEIGHT`: the borderless titled `Block` consumes its top row
/// for the title, leaving a single bar row, so this vertical budget is
/// `title row + 1 bar row` (MIGRATION.md D8).
const GAUGE_HEIGHT: usize = 2;

/// the original `SPARKLINE_HEIGHT`.
const SPARKLINE_HEIGHT: usize = 3;

/// the original `CLUSTER_SPACING`: a blank row between stacked cluster blocks.
const CLUSTER_SPACING: usize = 1;

/// the original `PKG_TEXT_HEIGHT`: the Package block's title-text row.
const PKG_TEXT_HEIGHT: usize = 1;

/// Number of horizontal blocks for `n` clusters: `ceil(n / 2)` (the clusters
/// are paired two-up). Mirrors the original `num_blocks_for`.
pub(crate) fn num_blocks_for(n: usize) -> usize {
    n.div_ceil(2)
}

/// Return the last `n` values of `data` (the original draws each sparkline from
/// `as_slice_last_n(area.width)`; the backend ships full history, so the view
/// trims here to its allocated width).
pub(crate) fn last_n(data: &[u64], n: usize) -> Vec<u64> {
    let start = data.len().saturating_sub(n);
    data[start..].to_vec()
}

/// Computed Overview geometry. All widths/heights are in terminal cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OverviewLayout {
    /// Total terminal width.
    pub width: usize,
    /// Width inside a full-width panel's left/right borders (`width - 2`). Also
    /// the width of a single (un-paired) cluster cell and its gauge/sparkline.
    pub inner_width: usize,
    /// Width of one half-cell in a paired row (`(inner_width - GAP) / 2`).
    /// Used for paired clusters and the GPU/ANE and RAM/SWAP halves.
    pub half_width: usize,
    /// Gap between the two halves of a paired row.
    pub gap: usize,
    /// Gauge bar height inside a cell (1 row — the title is a separate `Text`,
    /// MIGRATION.md D8).
    pub gauge_height: usize,
    /// Effective sparkline height. The original GPU/ANE/RAM/SWAP sparklines request
    /// a nominal 9 rows but the outer block is only sized for
    /// `GAUGE_HEIGHT + SPARKLINE_HEIGHT` inner rows, so the original clips them to
    /// `SPARKLINE_HEIGHT` (= 3). Every Overview sparkline therefore renders at 3.
    pub spark_height: usize,
    /// Outer width of the Package panel (`7/10` of `width`).
    pub package_width: usize,
    /// Package sparkline width (`package_width - 2`, inside its borders).
    pub package_inner: usize,
    /// Outer width of the Thermals panel (`3/10` of `width`).
    pub thermals_width: usize,
    /// Outer height (incl. borders) of the CPU Clusters panel.
    pub cpu_panel_height: usize,
    /// Outer height of the GPU & ANE panel.
    pub gpu_panel_height: usize,
    /// Outer height of the Package + Thermals panel.
    pub pkg_panel_height: usize,
    /// Outer height of the Memory & SWAP panel.
    pub mem_panel_height: usize,
}

impl OverviewLayout {
    /// Compute the Overview geometry for a terminal `width` and the given
    /// number of efficiency / performance / super clusters.
    pub(crate) fn new(width: usize, n_e: usize, n_p: usize, n_s: usize) -> Self {
        let inner_width = width.saturating_sub(2);
        let half_width = inner_width.saturating_sub(GAP) / 2;

        // CPU Clusters panel height = borders + per-block heights + the
        // CLUSTER_SPACING blank rows between blocks.
        let n_blocks = num_blocks_for(n_e) + num_blocks_for(n_p) + num_blocks_for(n_s);
        let cls_block_height = GAUGE_HEIGHT + SPARKLINE_HEIGHT;
        let cpu_block_height =
            cls_block_height * n_blocks + n_blocks.saturating_sub(1) * CLUSTER_SPACING;

        // Package + Thermals share the row 7/10 vs 3/10 (the original Ratio).
        let package_width = width * 7 / 10;
        let thermals_width = width.saturating_sub(package_width);

        Self {
            width,
            inner_width,
            half_width,
            gap: GAP,
            gauge_height: 1,
            spark_height: SPARKLINE_HEIGHT,
            package_width,
            package_inner: package_width.saturating_sub(2),
            thermals_width,
            cpu_panel_height: 2 + cpu_block_height,
            gpu_panel_height: 2 + (GAUGE_HEIGHT + SPARKLINE_HEIGHT),
            pkg_panel_height: 2 + (PKG_TEXT_HEIGHT + SPARKLINE_HEIGHT),
            mem_panel_height: 2 + (GAUGE_HEIGHT + SPARKLINE_HEIGHT),
        }
    }

    /// Compute the geometry directly from an [`OverviewFrame`].
    pub(crate) fn for_frame(width: usize, f: &OverviewFrame) -> Self {
        Self::new(width, f.e_meters.len(), f.p_meters.len(), f.s_meters.len())
    }
}

// ─── CPU/GPU row geometry (MIGRATION.md §9.4/§9.5) ───────────────────────────

/// CPU id column width (`Constraint::Length(5)` in `tab_cpu.rs`).
const ID_WIDTH: usize = 5;

/// `freq:` label column width (`FREQUENCY_LABEL_WIDTH` in tab_cpu/tab_gpu).
const FREQ_LABEL_WIDTH: usize = 6;

/// Frequency value column width (`FREQUENCY_VALUE_WIDTH`, e.g. `"1085 MHz "`).
const FREQ_VALUE_WIDTH: usize = 10;

/// Sparkline history slot: `HISTORY_LENGTH (8) + 1` (`Constraint::Length(8+1)`).
/// The sparkline emits exactly 8 cells; the `+1` is one trailing space before
/// the gauge so the gauge column-aligns with the original (MIGRATION.md §9.4 trap 2).
const HISTORY_SLOT: usize = 9;

/// Geometry of one CPU core row (MIGRATION.md §9.4).
///
/// Layout: `[id 5][activity: sparkline 8+1, line_gauge][frequency: "freq:" 6,
/// sparkline 8+1, value 10, line_gauge]`. The two halves split the post-id
/// remainder via the original `Ratio(1,2)/Ratio(1,2)`, which gives the **first**
/// (activity) half the odd column (verified against the original `Layout::split`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CpuRowLayout {
    /// Id column width (always [`ID_WIDTH`]).
    pub id_w: usize,
    /// Activity sparkline slot (always [`HISTORY_SLOT`]).
    pub act_spark_slot: usize,
    /// Activity line-gauge area width (`activity_half - HISTORY_SLOT`).
    pub act_gauge_w: usize,
    /// `freq:` label column (always [`FREQ_LABEL_WIDTH`]).
    pub freq_label_w: usize,
    /// Frequency sparkline slot (always [`HISTORY_SLOT`]).
    pub freq_spark_slot: usize,
    /// Frequency value column (always [`FREQ_VALUE_WIDTH`]).
    pub freq_value_w: usize,
    /// Frequency line-gauge area width
    /// (`frequency_half - FREQ_LABEL_WIDTH - HISTORY_SLOT - FREQ_VALUE_WIDTH`).
    pub freq_gauge_w: usize,
}

impl CpuRowLayout {
    /// Compute the row geometry for a cluster panel of total outer `width`
    /// (i.e. the terminal width; the rows sit inside the panel's `±1` borders).
    pub(crate) fn new(width: usize) -> Self {
        let content = width.saturating_sub(2); // inside the cluster L/R borders
        let other = content.saturating_sub(ID_WIDTH);
        // the original Ratio(1,2)/Ratio(1,2): the activity half takes the odd column.
        let act_w = other.div_ceil(2);
        let freq_w = other - act_w;
        Self {
            id_w: ID_WIDTH,
            act_spark_slot: HISTORY_SLOT,
            act_gauge_w: act_w.saturating_sub(HISTORY_SLOT),
            freq_label_w: FREQ_LABEL_WIDTH,
            freq_spark_slot: HISTORY_SLOT,
            freq_value_w: FREQ_VALUE_WIDTH,
            freq_gauge_w: freq_w.saturating_sub(FREQ_LABEL_WIDTH + HISTORY_SLOT + FREQ_VALUE_WIDTH),
        }
    }

    /// Activity line-gauge **bar** width: the original fills `area - label - 1`
    /// (label + the always-present 1-col gap; MIGRATION.md §9.4 trap 1).
    pub(crate) fn act_bar(&self, label_len: usize) -> usize {
        self.act_gauge_w.saturating_sub(label_len + 1)
    }

    /// Frequency line-gauge **bar** width (`freq_gauge_w - label - 1`). The freq
    /// label is the original default `"{:3.0}%"` (always 4 cols), but we derive
    /// from the actual length for safety.
    pub(crate) fn freq_bar(&self, label_len: usize) -> usize {
        self.freq_gauge_w.saturating_sub(label_len + 1)
    }
}

/// Geometry of the single GPU block (MIGRATION.md §9.5).
///
/// Two rows inside the block's `margin(1)`: top = activity | frequency, bottom =
/// power | peak. Each `|` is a `Ratio(1,2)/Ratio(1,2)` split of the inner width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GpuLayout {
    /// Activity sparkline slot ([`HISTORY_SLOT`]).
    pub act_spark_slot: usize,
    /// Activity line-gauge area width.
    pub act_gauge_w: usize,
    /// `freq:` label column.
    pub freq_label_w: usize,
    /// Frequency sparkline slot.
    pub freq_spark_slot: usize,
    /// Frequency value column.
    pub freq_value_w: usize,
    /// Frequency line-gauge area width.
    pub freq_gauge_w: usize,
    /// Power sparkline slot.
    pub power_spark_slot: usize,
    /// Power value area width (`power_half - HISTORY_SLOT`).
    pub power_value_w: usize,
    /// Peak text area width (the right half of the bottom row).
    pub peak_w: usize,
}

impl GpuLayout {
    /// Compute the GPU block geometry for a block of total outer `width`.
    pub(crate) fn new(width: usize) -> Self {
        let inner = width.saturating_sub(2); // inside the GPU block L/R borders
        let act_w = inner.div_ceil(2);
        let freq_w = inner - act_w;
        let power_w = inner.div_ceil(2);
        let peak_w = inner - power_w;
        Self {
            act_spark_slot: HISTORY_SLOT,
            act_gauge_w: act_w.saturating_sub(HISTORY_SLOT),
            freq_label_w: FREQ_LABEL_WIDTH,
            freq_spark_slot: HISTORY_SLOT,
            freq_value_w: FREQ_VALUE_WIDTH,
            freq_gauge_w: freq_w.saturating_sub(FREQ_LABEL_WIDTH + HISTORY_SLOT + FREQ_VALUE_WIDTH),
            power_spark_slot: HISTORY_SLOT,
            power_value_w: power_w.saturating_sub(HISTORY_SLOT),
            peak_w,
        }
    }

    /// Activity line-gauge bar width (`act_gauge_w - label - 1`).
    pub(crate) fn act_bar(&self, label_len: usize) -> usize {
        self.act_gauge_w.saturating_sub(label_len + 1)
    }

    /// Frequency line-gauge bar width (`freq_gauge_w - label - 1`).
    pub(crate) fn freq_bar(&self, label_len: usize) -> usize {
        self.freq_gauge_w.saturating_sub(label_len + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn num_blocks_for_is_ceil_half() {
        assert_eq!(num_blocks_for(0), 0);
        assert_eq!(num_blocks_for(1), 1);
        assert_eq!(num_blocks_for(2), 1);
        assert_eq!(num_blocks_for(3), 2);
        assert_eq!(num_blocks_for(4), 2);
    }

    #[test]
    fn last_n_trims_to_tail() {
        assert_eq!(last_n(&[1, 2, 3, 4, 5], 3), vec![3, 4, 5]);
        // Fewer points than requested -> all of them.
        assert_eq!(last_n(&[1, 2], 5), vec![1, 2]);
        assert_eq!(last_n(&[], 4), Vec::<u64>::new());
    }

    #[test]
    fn paired_halves_plus_gap_fill_inner_width() {
        // At width 120 each paired cluster half is 58 wide with a 2-col gap.
        let l = OverviewLayout::new(120, 0, 2, 1);
        assert_eq!(l.inner_width, 118);
        assert_eq!(l.half_width, 58);
        assert_eq!(l.half_width * 2 + l.gap, l.inner_width);
    }

    #[test]
    fn package_and_thermals_split_seven_three() {
        let l = OverviewLayout::new(120, 0, 2, 1);
        assert_eq!(l.package_width, 84);
        assert_eq!(l.thermals_width, 36);
        assert_eq!(l.package_width + l.thermals_width, 120);
        assert_eq!(l.package_inner, 82);
    }

    #[test]
    fn cpu_panel_height_tracks_cluster_count() {
        // 2 P + 1 S = num_blocks_for(2) + num_blocks_for(1) = 1 + 1 = 2 blocks.
        // height = 2 borders + 2*(2+3) + (2-1)*1 spacing = 2 + 10 + 1 = 13.
        let l = OverviewLayout::new(120, 0, 2, 1);
        assert_eq!(l.cpu_panel_height, 13);

        // A single cluster: 1 block, no inter-block spacing.
        // height = 2 + 1*(2+3) + 0 = 7.
        let one = OverviewLayout::new(120, 0, 1, 0);
        assert_eq!(one.cpu_panel_height, 7);

        // 4 E + 8 P + 2 S = 2 + 4 + 1 = 7 blocks.
        // height = 2 + 7*5 + 6*1 = 2 + 35 + 6 = 43.
        let big = OverviewLayout::new(120, 4, 8, 2);
        assert_eq!(big.cpu_panel_height, 43);
    }

    #[test]
    fn fixed_panel_heights() {
        let l = OverviewLayout::new(120, 0, 2, 1);
        assert_eq!(l.gpu_panel_height, 7);
        assert_eq!(l.pkg_panel_height, 6);
        assert_eq!(l.mem_panel_height, 7);
        assert_eq!(l.gauge_height, 1);
        assert_eq!(l.spark_height, 3);
    }

    #[test]
    fn cpu_row_geometry_at_120() {
        // content = 118, other = 113, activity half = 57 (odd col), freq = 56.
        // Verified against the original `Layout::split` (MIGRATION.md §9.4).
        let l = CpuRowLayout::new(120);
        assert_eq!(l.id_w, 5);
        assert_eq!(l.act_spark_slot, 9);
        assert_eq!(l.act_gauge_w, 48); // 57 - 9
        assert_eq!(l.freq_label_w, 6);
        assert_eq!(l.freq_spark_slot, 9);
        assert_eq!(l.freq_value_w, 10);
        assert_eq!(l.freq_gauge_w, 31); // 56 - 6 - 9 - 10

        // The whole row tiles the 118-col content area exactly.
        let activity = l.act_spark_slot + l.act_gauge_w; // 57
        let frequency = l.freq_label_w + l.freq_spark_slot + l.freq_value_w + l.freq_gauge_w; // 56
        assert_eq!(l.id_w + activity + frequency, 118);
    }

    #[test]
    fn cpu_line_gauge_bar_widths_subtract_label_and_gap() {
        let l = CpuRowLayout::new(120);
        // Activity label varies 4..6 cols; bar = gauge_area - label - 1.
        assert_eq!(l.act_bar("0.0%".len()), 43); // 48 - 4 - 1
        assert_eq!(l.act_bar("100.0%".len()), 41); // 48 - 6 - 1
        // Freq default label "{:3.0}%" is always 4 cols.
        assert_eq!(l.freq_bar("  0%".chars().count()), 26); // 31 - 4 - 1
        assert_eq!(l.freq_bar("100%".chars().count()), 26);
    }

    #[test]
    fn gpu_geometry_at_120() {
        // inner = 118 (even): each half = 59.
        let g = GpuLayout::new(120);
        assert_eq!(g.act_spark_slot, 9);
        assert_eq!(g.act_gauge_w, 50); // 59 - 9
        assert_eq!(g.freq_gauge_w, 34); // 59 - 6 - 9 - 10
        assert_eq!(g.power_spark_slot, 9);
        assert_eq!(g.power_value_w, 50); // 59 - 9
        assert_eq!(g.peak_w, 59);
        // Top row tiles the 118-col inner exactly.
        let activity = g.act_spark_slot + g.act_gauge_w; // 59
        let frequency = g.freq_label_w + g.freq_spark_slot + g.freq_value_w + g.freq_gauge_w; // 59
        assert_eq!(activity + frequency, 118);
        // Freq bar subtracts the 4-col default label + 1 gap.
        assert_eq!(g.freq_bar(4), 29); // 34 - 4 - 1
        assert_eq!(g.act_bar("1.1%".len()), 45); // 50 - 4 - 1
    }
}
