//! Frontend geometry for the tab views (MIGRATION.md §7.9).
//!
//! iocraft cannot expose a flex-allocated width at element-*build* time, so the
//! leaf widgets (gauge/sparkline/line_gauge) take explicit dimensions. This
//! module is the single place that mirrors ratatui's constraint math, turning a
//! terminal `width` plus the [`OverviewFrame`] shape into the per-cell widths
//! and heights the Overview view passes down. The backend [`Frame`] stays
//! width-free; all pixel geometry lives here.
//!
//! [`Frame`]: crate::backend::frame::Frame

use crate::backend::frame::OverviewFrame;

/// 2-column gap between paired halves (ratatui `Constraint::Length(2)` in
/// `tab_overview.rs`).
pub(crate) const GAP: usize = 2;

/// ratatui `GAUGE_HEIGHT`: the borderless titled `Block` consumes its top row
/// for the title, leaving a single bar row, so this vertical budget is
/// `title row + 1 bar row` (MIGRATION.md D8).
const GAUGE_HEIGHT: usize = 2;

/// ratatui `SPARKLINE_HEIGHT`.
const SPARKLINE_HEIGHT: usize = 3;

/// ratatui `CLUSTER_SPACING`: a blank row between stacked cluster blocks.
const CLUSTER_SPACING: usize = 1;

/// ratatui `PKG_TEXT_HEIGHT`: the Package block's title-text row.
const PKG_TEXT_HEIGHT: usize = 1;

/// Number of horizontal blocks for `n` clusters: `ceil(n / 2)` (the clusters
/// are paired two-up). Mirrors ratatui's `num_blocks_for`.
pub(crate) fn num_blocks_for(n: usize) -> usize {
    n.div_ceil(2)
}

/// Return the last `n` values of `data` (ratatui draws each sparkline from
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
    /// Effective sparkline height. ratatui's GPU/ANE/RAM/SWAP sparklines request
    /// a nominal 9 rows but the outer block is only sized for
    /// `GAUGE_HEIGHT + SPARKLINE_HEIGHT` inner rows, so ratatui clips them to
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

        // Package + Thermals share the row 7/10 vs 3/10 (ratatui Ratio).
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
}
