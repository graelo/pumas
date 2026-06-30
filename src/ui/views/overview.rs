//! Overview tab (MIGRATION.md §9.3, ratatui `tab_overview.rs`).
//!
//! Four outer bordered panels stacked vertically:
//! 1. CPU Clusters — E then P then S clusters, paired two-up via `chunks(2)`;
//! 2. GPU & ANE — two halves;
//! 3. Package + Thermals — a 70/30 split row of two panels;
//! 4. Memory & SWAP — two halves.
//!
//! Only the four outer panels are bordered; the inner cells are NOT (D8). Each
//! cell is a plain `Text` title row, a single-row gauge bar, and a 3-row
//! sparkline. All strings arrive pre-formatted in the [`Frame`]; all widths come
//! from [`OverviewLayout`].
//!
//! [`Frame`]: crate::backend::frame::Frame

use iocraft::prelude::*;

use crate::{
    backend::frame::{Meter, OverviewFrame, SparkText, Thermals},
    ui::{
        components::{
            gauge::{Gauge, RenderedGauge},
            panel::panel,
            sparkline::{RenderedSparkline, Sparkline},
        },
        layout::{OverviewLayout, last_n},
        theme::Theme,
    },
};

/// An empty `View` spacer of the given width (the inter-cell gap).
fn h_gap(width: usize) -> AnyElement<'static> {
    #[expect(clippy::cast_possible_truncation)]
    let w = width as u32;
    element! { View(width: w) }.into_any()
}

/// An empty row spacer of the given height (inter-block `CLUSTER_SPACING`).
fn blank_row(height: u32) -> AnyElement<'static> {
    element! { View(height: height) }.into_any()
}

/// A single meter cell: title row + 1-row gauge + 3-row sparkline (D8). The
/// inner cells are unbordered; only the outer panel draws a border.
fn meter_cell(m: &Meter, width: usize, spark_height: usize, theme: Theme) -> AnyElement<'static> {
    #[expect(clippy::cast_possible_truncation)]
    let w = width as u32;
    let gauge = element! {
        Gauge(gauge: Some(RenderedGauge {
            ratio: m.ratio,
            width,
            height: 1,
            fg: theme.gauge_fg,
            bg: theme.gauge_bg,
        }))
    }
    .into_any();
    let spark = element! {
        Sparkline(sparkline: Some(RenderedSparkline {
            data: last_n(&m.spark, width),
            max: m.spark_max,
            height: spark_height,
            fg: theme.history_fg,
            bg: theme.history_bg,
        }))
    }
    .into_any();
    // `Overflow::Hidden` clips an over-long title to the cell width, matching
    // ratatui's `Paragraph`, which truncates at the block boundary rather than
    // spilling into the neighbouring half.
    element! {
        View(flex_direction: FlexDirection::Column, width: w, overflow: Overflow::Hidden) {
            Text(content: m.title.clone(), wrap: TextWrap::NoWrap)
            #(vec![gauge, spark])
        }
    }
    .into_any()
}

/// A paired row: two half-width cells separated by the gap.
fn pair_row(
    left: &Meter,
    right: &Meter,
    lay: &OverviewLayout,
    theme: Theme,
) -> AnyElement<'static> {
    let cells = vec![
        meter_cell(left, lay.half_width, lay.spark_height, theme),
        h_gap(lay.gap),
        meter_cell(right, lay.half_width, lay.spark_height, theme),
    ];
    element! {
        View(flex_direction: FlexDirection::Row) {
            #(cells)
        }
    }
    .into_any()
}

/// Build the CPU Clusters panel body: E then P then S clusters, each kind paired
/// two-up via `chunks(2)`, with a `CLUSTER_SPACING` blank row between blocks.
fn cluster_blocks(f: &OverviewFrame, lay: &OverviewLayout, theme: Theme) -> AnyElement<'static> {
    // Collect (left, optional right) per block, kept per-kind so an odd E count
    // never pairs with a P cluster.
    let mut specs: Vec<(&Meter, Option<&Meter>)> = Vec::new();
    for kind in [&f.e_meters, &f.p_meters, &f.s_meters] {
        for chunk in kind.chunks(2) {
            match chunk {
                [a] => specs.push((a, None)),
                [a, b] => specs.push((a, Some(b))),
                _ => unreachable!("chunks(2) yields 1 or 2 elements"),
            }
        }
    }

    let last = specs.len().saturating_sub(1);
    let mut rows: Vec<AnyElement<'static>> = Vec::new();
    for (i, (left, right)) in specs.iter().enumerate() {
        let block = match right {
            Some(r) => pair_row(left, r, lay, theme),
            None => meter_cell(left, lay.inner_width, lay.spark_height, theme),
        };
        rows.push(block);
        if i != last {
            rows.push(blank_row(1));
        }
    }

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(rows)
        }
    }
    .into_any()
}

/// Build the Package panel body: title text + 3-row sparkline (no overshoot).
fn package_body(pkg: &SparkText, lay: &OverviewLayout, theme: Theme) -> AnyElement<'static> {
    let spark = element! {
        Sparkline(sparkline: Some(RenderedSparkline {
            data: last_n(&pkg.spark, lay.package_inner),
            max: pkg.spark_max,
            height: lay.spark_height,
            fg: theme.history_fg,
            bg: theme.history_bg,
        }))
    }
    .into_any();
    element! {
        View(flex_direction: FlexDirection::Column) {
            Text(content: pkg.title.clone(), wrap: TextWrap::NoWrap)
            #(vec![spark])
        }
    }
    .into_any()
}

/// Build the Thermals panel body: the `Pressure: {x}` line (accent when nominal,
/// else Yellow), padded to the Package body height so the two panels align.
fn thermals_body(t: &Thermals, theme: Theme) -> AnyElement<'static> {
    let p_color = if t.is_nominal {
        theme.accent
    } else {
        Color::Yellow
    };
    let pads = vec![blank_row(1), blank_row(1), blank_row(1)];
    element! {
        View(flex_direction: FlexDirection::Column) {
            MixedText(
                wrap: TextWrap::NoWrap,
                contents: vec![
                    MixedTextContent::new("Pressure: "),
                    MixedTextContent::new(t.pressure.clone()).color(p_color),
                ],
            )
            #(pads)
        }
    }
    .into_any()
}

/// Render the full Overview tab at `width` (and `height`, only the trailing
/// spacer below the four panels depends on it).
pub(crate) fn overview(f: &OverviewFrame, width: usize, theme: Theme) -> AnyElement<'static> {
    let lay = OverviewLayout::for_frame(width, f);

    let cpu_panel = panel(
        &f.cpu_clusters_title,
        lay.width,
        Color::Reset,
        cluster_blocks(f, &lay, theme),
    );

    let gpu_panel = panel(
        " GPU & ANE ",
        lay.width,
        Color::Reset,
        pair_row(&f.gpu, &f.ane, &lay, theme),
    );

    let pkg_panel = panel(
        " Package ",
        lay.package_width,
        Color::Reset,
        package_body(&f.package, &lay, theme),
    );
    let thr_panel = panel(
        " Thermals ",
        lay.thermals_width,
        Color::Reset,
        thermals_body(&f.thermals, theme),
    );
    let pkg_thr_row = element! {
        View(flex_direction: FlexDirection::Row) {
            #(vec![pkg_panel, thr_panel])
        }
    }
    .into_any();

    let mem_panel = panel(
        " Memory & SWAP ",
        lay.width,
        Color::Reset,
        pair_row(&f.ram, &f.swap, &lay, theme),
    );

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(vec![cpu_panel, gpu_panel, pkg_thr_row, mem_panel])
        }
    }
    .into_any()
}
