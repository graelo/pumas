//! GPU tab (MIGRATION.md §9.5, ratatui `tab_gpu.rs`).
//!
//! A single bordered `GPU:` block (height 4) with two inner rows — top =
//! activity | frequency, bottom = power | peak — then a `Thermals` block and the
//! shared `Frequencies` table. Widths come from [`GpuLayout`]; strings arrive
//! pre-formatted in the [`GpuFrame`].

use iocraft::prelude::*;

use crate::{
    backend::frame::{GpuFrame, Thermals},
    ui::{
        components::{
            line_gauge::{LineGauge, RenderedLineGauge},
            panel::panel,
        },
        layout::GpuLayout,
        theme::Theme,
        views::{freq_table_panel, spark_slot, text_col},
    },
};

/// Build the top row: activity (sparkline + line gauge) | frequency.
fn top_row(f: &GpuFrame, lay: &GpuLayout, theme: Theme) -> AnyElement<'static> {
    let act_histo = spark_slot(
        f.act_spark.clone(),
        f.act_spark_max,
        lay.act_spark_slot,
        theme,
    );
    let act_gauge = element! {
        LineGauge(line_gauge: Some(RenderedLineGauge {
            ratio: f.act_ratio,
            width: lay.act_bar(f.act_label.chars().count()),
            label: Some(f.act_label.clone()),
            fg: theme.gauge_fg,
            bg: theme.gauge_bg,
        }))
    }
    .into_any();
    let activity = element! {
        View(flex_direction: FlexDirection::Row) { #(vec![act_histo, act_gauge]) }
    }
    .into_any();

    let freq_lbl = text_col("freq:".to_string(), lay.freq_label_w, Color::Reset);
    let freq_histo = spark_slot(
        f.freq_spark.clone(),
        f.freq_spark_max,
        lay.freq_spark_slot,
        theme,
    );
    let freq_val = text_col(f.freq_value.clone(), lay.freq_value_w, Color::Reset);
    let freq_gauge = element! {
        LineGauge(line_gauge: Some(RenderedLineGauge {
            ratio: f.freq_ratio,
            width: lay.freq_bar(f.freq_label.chars().count()),
            label: Some(f.freq_label.clone()),
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
        View(flex_direction: FlexDirection::Row) { #(vec![activity, frequency]) }
    }
    .into_any()
}

/// Build the bottom row: power (sparkline + value) | peak text.
fn bottom_row(f: &GpuFrame, lay: &GpuLayout, theme: Theme) -> AnyElement<'static> {
    let pow_histo = spark_slot(
        f.power_spark.clone(),
        f.power_spark_max,
        lay.power_spark_slot,
        theme,
    );
    let pow_val = text_col(f.power_value.clone(), lay.power_value_w, Color::Reset);
    let power = element! {
        View(flex_direction: FlexDirection::Row) { #(vec![pow_histo, pow_val]) }
    }
    .into_any();
    let peak = text_col(f.peak_text.clone(), lay.peak_w, Color::Reset);
    element! {
        View(flex_direction: FlexDirection::Row) { #(vec![power, peak]) }
    }
    .into_any()
}

/// Build the `Thermals` block body (`Pressure: {x}`, accent when nominal else
/// Yellow).
fn thermals_panel(t: &Thermals, width: usize, theme: Theme) -> AnyElement<'static> {
    let p_color = if t.is_nominal {
        theme.accent
    } else {
        Color::Yellow
    };
    let body = element! {
        MixedText(
            wrap: TextWrap::NoWrap,
            contents: vec![
                MixedTextContent::new("Pressure: "),
                MixedTextContent::new(t.pressure.clone()).color(p_color),
            ],
        )
    }
    .into_any();
    panel(" Thermals ", width, Color::Reset, body)
}

/// Render the full GPU tab at `width`.
pub(crate) fn gpu(f: &GpuFrame, width: usize, theme: Theme) -> AnyElement<'static> {
    let lay = GpuLayout::new(width);
    let gpu_body = element! {
        View(flex_direction: FlexDirection::Column) {
            #(vec![top_row(f, &lay, theme), bottom_row(f, &lay, theme)])
        }
    }
    .into_any();
    let gpu_block = panel("GPU: ", width, Color::Reset, gpu_body);

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(vec![
                gpu_block,
                thermals_panel(&f.thermals, width, theme),
                freq_table_panel(&f.freq_table, width),
            ])
        }
    }
    .into_any()
}
