//! Per-tab view builders for the iocraft UI.
//!
//! Each view is a plain function that turns an owned `Frame` sub-struct plus the
//! frontend [`OverviewLayout`](crate::ui::layout) geometry into an
//! `AnyElement<'static>`. Phase 2A landed the splash and Overview tab; Phase 2B
//! adds the CPU, GPU, Memory and SoC tabs.

pub(crate) mod cpu;
pub(crate) mod gpu;
pub(crate) mod memory;
pub(crate) mod overview;
pub(crate) mod soc;
pub(crate) mod splash;

#[cfg(test)]
mod tests;

use iocraft::prelude::*;

use crate::{
    backend::frame::FreqTable,
    ui::{
        components::{
            panel::panel,
            sparkline::{RenderedSparkline, Sparkline},
        },
        theme::Theme,
    },
};

/// Frequency-table label column width (the original `label_width = 10`, shared by the
/// CPU and GPU `Frequencies` tables).
const FREQ_TABLE_LABEL_WIDTH: usize = 10;

/// Inner rows of the `Frequencies` table block (`FREQUENCY_TABLE_HEIGHT`); the
/// body is padded to this so the bordered block is always `2 + 5` tall.
const FREQ_TABLE_INNER_ROWS: usize = 5;

/// A single-row sparkline confined to its fixed `slot` column (8 data cells +
/// the trailing gap), so the following gauge column-aligns regardless of data
/// length (MIGRATION.md §9.4 trap 2).
pub(crate) fn spark_slot(
    data: Vec<u64>,
    max: u64,
    slot: usize,
    theme: Theme,
) -> AnyElement<'static> {
    #[expect(clippy::cast_possible_truncation)]
    let w = slot as u32;
    element! {
        View(width: w) {
            Sparkline(sparkline: Some(RenderedSparkline {
                data,
                max,
                height: 1,
                fg: theme.history_fg,
                bg: theme.history_bg,
            }))
        }
    }
    .into_any()
}

/// A fixed-width, left-aligned text column (no wrap).
pub(crate) fn text_col(content: String, width: usize, color: Color) -> AnyElement<'static> {
    #[expect(clippy::cast_possible_truncation)]
    let w = width as u32;
    element! {
        View(width: w) {
            Text(content: content, color: color, wrap: TextWrap::NoWrap)
        }
    }
    .into_any()
}

/// One row of a `Frequencies`/SoC-style 2-col table: `[label `width`][1 gap][value
/// bold]` (the original `Table` default `column_spacing = 1`).
pub(crate) fn two_col_row(label: &str, value: &str, label_w: usize) -> AnyElement<'static> {
    let lbl = text_col(label.to_string(), label_w, Color::Reset);
    element! {
        View(flex_direction: FlexDirection::Row) {
            #(vec![lbl])
            Text(content: " ", wrap: TextWrap::NoWrap)
            Text(content: value.to_string(), weight: Weight::Bold, wrap: TextWrap::NoWrap)
        }
    }
    .into_any()
}

/// Build the bordered `Frequencies` table panel (height `2 + 5`), shared by the
/// CPU and GPU tabs.
pub(crate) fn freq_table_panel(ft: &FreqTable, width: usize) -> AnyElement<'static> {
    let mut rows: Vec<AnyElement<'static>> = ft
        .rows
        .iter()
        .map(|(l, r)| two_col_row(l, r, FREQ_TABLE_LABEL_WIDTH))
        .collect();
    while rows.len() < FREQ_TABLE_INNER_ROWS {
        rows.push(element! { View(height: 1u32) }.into_any());
    }
    let body = element! {
        View(flex_direction: FlexDirection::Column) { #(rows) }
    }
    .into_any();
    panel("Frequencies", width, Color::Reset, body)
}
