//! CPU tab (MIGRATION.md §9.4, ratatui `tab_cpu.rs`).
//!
//! One bordered panel per cluster (E…, then P…, then S…), each holding one row
//! per core, followed by a bordered `Frequencies` table. Every per-row width
//! comes from [`CpuRowLayout`]; all strings arrive pre-formatted in the
//! [`CpuFrame`].
//!
//! A core row is `[id 5][activity][frequency]`, where:
//! - activity = `[sparkline 8 +1 gap][line_gauge: "{:.1}%" label + bar]`;
//! - frequency = `[6 "freq:"][sparkline 8 +1 gap][10 value][line_gauge: default
//!   "{:3.0}%" label + bar]`.
//!
//! The line-gauge bar width is **variable**: ratatui fills `area - label - 1`,
//! so the bar shrinks as the label grows (MIGRATION.md §9.4 traps 1 & 2).

use iocraft::prelude::*;

use crate::{
    backend::frame::{CpuCluster, CpuFrame, CpuRow},
    ui::{
        components::{
            line_gauge::{LineGauge, RenderedLineGauge},
            panel::panel,
        },
        layout::CpuRowLayout,
        theme::Theme,
        views::{freq_table_panel, spark_slot, text_col},
    },
};

/// Build one CPU core row.
fn cpu_row(row: &CpuRow, lay: &CpuRowLayout, theme: Theme) -> AnyElement<'static> {
    let id = text_col(row.id_label.clone(), lay.id_w, theme.accent);

    // Activity: sparkline slot + line gauge (label "{:.1}%").
    let act_label_len = row.act_label.chars().count();
    let act_histo = spark_slot(
        row.act_spark.clone(),
        row.act_spark_max,
        lay.act_spark_slot,
        theme,
    );
    let act_gauge = element! {
        LineGauge(line_gauge: Some(RenderedLineGauge {
            ratio: row.act_ratio,
            width: lay.act_bar(act_label_len),
            label: Some(row.act_label.clone()),
            fg: theme.gauge_fg,
            bg: theme.gauge_bg,
        }))
    }
    .into_any();
    let activity = element! {
        View(flex_direction: FlexDirection::Row) { #(vec![act_histo, act_gauge]) }
    }
    .into_any();

    // Frequency: "freq:" + sparkline slot + value + line gauge (default label).
    let freq_label_len = row.freq_label.chars().count();
    let freq_lbl = text_col("freq:".to_string(), lay.freq_label_w, Color::Reset);
    let freq_histo = spark_slot(
        row.freq_spark.clone(),
        row.freq_spark_max,
        lay.freq_spark_slot,
        theme,
    );
    let freq_val = text_col(row.freq_value.clone(), lay.freq_value_w, Color::Reset);
    let freq_gauge = element! {
        LineGauge(line_gauge: Some(RenderedLineGauge {
            ratio: row.freq_ratio,
            width: lay.freq_bar(freq_label_len),
            label: Some(row.freq_label.clone()),
            fg: theme.gauge_fg,
            bg: theme.gauge_bg,
        }))
    }
    .into_any();
    let frequency = element! {
        View(flex_direction: FlexDirection::Row) {
            #(vec![freq_lbl, freq_histo, freq_val, freq_gauge])
        }
    }
    .into_any();

    element! {
        View(flex_direction: FlexDirection::Row) { #(vec![id, activity, frequency]) }
    }
    .into_any()
}

/// Build one bordered cluster panel (`title` is `" {name}: "`).
fn cluster_panel(c: &CpuCluster, width: usize, theme: Theme) -> AnyElement<'static> {
    let lay = CpuRowLayout::new(width);
    let rows: Vec<AnyElement<'static>> = c.cpus.iter().map(|r| cpu_row(r, &lay, theme)).collect();
    let body = element! {
        View(flex_direction: FlexDirection::Column) { #(rows) }
    }
    .into_any();
    panel(&c.title, width, Color::Reset, body)
}

/// Render the full CPU tab at `width`.
pub(crate) fn cpu(f: &CpuFrame, width: usize, theme: Theme) -> AnyElement<'static> {
    let mut blocks: Vec<AnyElement<'static>> = f
        .clusters
        .iter()
        .map(|c| cluster_panel(c, width, theme))
        .collect();
    blocks.push(freq_table_panel(&f.freq_table, width));

    element! {
        View(flex_direction: FlexDirection::Column) { #(blocks) }
    }
    .into_any()
}
