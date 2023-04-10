//! Overview tab.

use std::iter::zip;

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};

use crate::{
    app::{App, History},
    parser::powermetrics::{ClusterMetrics, PowerMetrics},
    parser::soc::Soc,
    units,
};

const SPARKLINE_HEIGHT: u16 = 3;
const SPARKLINE_MAX_OVERSHOOT: f32 = 1.05;
const GAUGE_HEIGHT: u16 = 2;
const PKG_TEXT_HEIGHT: u16 = 1;
const THR_TEXT_HEIGHT: u16 = 1;

/// Draw the Overview tab.
///
pub(crate) fn draw_overview_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let metrics = match &app.metrics {
        Some(metrics) => metrics,
        None => return,
    };

    let num_clusters = metrics.e_clusters.len();
    let cpu_block_height = GAUGE_HEIGHT + SPARKLINE_HEIGHT;
    let gpu_block_height = GAUGE_HEIGHT + SPARKLINE_HEIGHT;
    let pkg_block_height = PKG_TEXT_HEIGHT + SPARKLINE_HEIGHT;
    let thr_block_height = THR_TEXT_HEIGHT;

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2 + cpu_block_height * num_clusters as u16), // Borders & CPU clusters blocks
                Constraint::Length(2 + gpu_block_height), // Borders & GPU ANE block
                Constraint::Length(2 + pkg_block_height), // Borders & Package block
                Constraint::Length(2 + thr_block_height), // Borders & Thermal block
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);
    let cpu_area = vertical_chunks[0];
    let gpu_area = vertical_chunks[1];
    let pkg_area = vertical_chunks[2];
    let thr_area = vertical_chunks[3];

    draw_cpu_clusters_usage_block(f, metrics, &app.history, cpu_area);
    draw_gpu_ane_usage_block(f, metrics, &app.soc, &app.history, gpu_area);
    draw_package_power_block(f, metrics, &app.history, pkg_area);
    draw_thermal_pressure_block(f, metrics, thr_area);
}

/// Draw the CPU clusters usage block.
///
/// On Apple Silicon, a CPU has at least one CPU efficiency cluster (the efficiency cores) and at
/// least one CPU performance cluster (the perf. cores). M1, M1 Pro, M1 Max, M2, M2 Pro and M2 Max
/// have one CPU cluster of each, while M1 Ultra has two of these pairs.
///
/// In this block, for each CPU, we draw both the efficiency cluster metrics and the performance
/// cluster metrics.
fn draw_cpu_clusters_usage_block<B>(
    f: &mut Frame<B>,
    metrics: &PowerMetrics,
    history: &History,
    area: Rect,
) where
    B: Backend,
{
    let num_cluster_pairs = metrics.e_clusters.len();

    let title = if num_cluster_pairs > 1 { "CPUs" } else { "CPU" };
    let title_with_power = format!(" {title}: {} ", units::watts2(metrics.cpu_w));
    let block = Block::default()
        .title(title_with_power)
        .borders(Borders::ALL);
    f.render_widget(block, area);

    let constraints = (0..num_cluster_pairs)
        .map(|_| Constraint::Length(GAUGE_HEIGHT + SPARKLINE_HEIGHT)) // block height
        .collect::<Vec<_>>();
    let cpu_cluster_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .margin(1)
        .split(area);

    for (clu_area, (e_cluster, p_cluster)) in zip(
        &*cpu_cluster_chunks,
        zip(&metrics.e_clusters, &metrics.p_clusters),
    ) {
        draw_cluster_pair_overall_metrics(f, e_cluster, p_cluster, history, *clu_area);
    }
}

/// Draw the overall metrics for a CPU cluster pair.
///
/// The efficiency cluster is on the left, the performance cluster on the right.
fn draw_cluster_pair_overall_metrics<B>(
    f: &mut Frame<B>,
    e_cluster: &ClusterMetrics,
    p_cluster: &ClusterMetrics,
    history: &History,
    area: Rect,
) where
    B: Backend,
{
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(1, 2),
                Constraint::Length(2), // space
                Constraint::Ratio(1, 2),
            ]
            .as_ref(),
        )
        // .horizontal_margin(1)
        .split(area);
    let left_area = horizontal_chunks[0];
    let right_area = horizontal_chunks[2];

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(GAUGE_HEIGHT),
                Constraint::Length(SPARKLINE_HEIGHT),
            ]
            .as_ref(),
        )
        .split(left_area);
    let top_left_area = left_chunks[0];
    let bottom_left_area = left_chunks[1];

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(GAUGE_HEIGHT),
                Constraint::Length(SPARKLINE_HEIGHT),
            ]
            .as_ref(),
        )
        .split(right_area);
    let top_right_area = right_chunks[0];
    let bottom_right_area = right_chunks[1];

    // Efficiency cores Usage Gauge.
    let title = format!(
        "{}: {} @ {}",
        e_cluster.name,
        units::percent1(e_cluster.active_ratio * 100.0),
        units::mhz(e_cluster.freq_mhz),
    );
    let gauge = Gauge::default()
        .block(Block::default().title(title))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Gray))
        .ratio(e_cluster.active_ratio);

    f.render_widget(gauge, top_left_area);

    // Efficiency cores Sparklines.
    let sig_name = format!("{}_active_ratio", e_cluster.name);
    let sig = history.get(&sig_name).unwrap();
    let sparkline = Sparkline::default()
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(bottom_left_area.width as usize))
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(sparkline, bottom_left_area);

    // Performance cores Usage Gauge.
    let title = format!(
        "{}: {} @ {}",
        p_cluster.name,
        units::percent1(p_cluster.active_ratio * 100.0),
        units::mhz(p_cluster.freq_mhz),
    );
    let gauge = Gauge::default()
        .block(Block::default().title(title))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Gray))
        .ratio(p_cluster.active_ratio);
    f.render_widget(gauge, top_right_area);

    // Performance cores Sparklines.
    let sig_name = format!("{}_active_ratio", p_cluster.name);
    let sig = history.get(&sig_name).unwrap();
    let sparkline = Sparkline::default()
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(bottom_right_area.width as usize))
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(sparkline, bottom_right_area);
}

/// Draw the GPU & ANE usage block.
///
/// ┌ GPU & ANE ────────────────────────────────────────────────────────────────────┐
/// │GPU Usage: 4.5 % @ 711 MHz ⚡️12.84 mW    ANE Usage: 0.0 % ⚡️0.00 W             │
/// │----------------- 4% -----------------   ----------------- 0% -----------------│
/// │                                                                               │
/// │                                                                               │
/// │                  ▁          ▄▅▄▅▄▅▂ ▁                                         │
/// └───────────────────────────────────────────────────────────────────────────────┘

fn draw_gpu_ane_usage_block<B>(
    f: &mut Frame<B>,
    metrics: &PowerMetrics,
    soc: &Soc,
    history: &History,
    area: Rect,
) where
    B: Backend,
{
    let block = Block::default().title(" GPU & ANE ").borders(Borders::ALL);
    f.render_widget(block, area);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(1, 2),
                Constraint::Length(2), // space
                Constraint::Ratio(1, 2),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(area);
    let left_area = horizontal_chunks[0];
    let right_area = horizontal_chunks[2];

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(9)].as_ref())
        .split(left_area);
    let top_left_area = left_chunks[0];
    let bottom_left_area = left_chunks[1];

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(9)].as_ref())
        .split(right_area);
    let top_right_area = right_chunks[0];
    let bottom_right_area = right_chunks[1];

    // left: GPU.
    let gpu = &metrics.gpu;
    let title = format!(
        "GPU Usage: {} @ {} ⚡️{}",
        units::percent1(gpu.active_ratio * 100.0),
        units::mhz(gpu.freq_mhz),
        units::watts2(metrics.gpu_w)
    );
    let gauge = Gauge::default()
        .block(Block::default().title(title))
        .gauge_style(
            Style::default().fg(Color::Green).bg(Color::Gray),
            // .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        )
        .ratio(gpu.active_ratio);

    f.render_widget(gauge, top_left_area);

    // GPU Usage Sparklines.
    // let sig_name = format!("{}_active_ratio", p_cluster.name);
    let sig = history.get("gpu_active_ratio").unwrap();
    let sparkline = Sparkline::default()
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(bottom_left_area.width as usize))
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(sparkline, bottom_left_area);

    // Right: ANE.
    let ane_active_ratio = metrics.ane_w as f64 / soc.max_ane_w;
    let title = format!(
        "ANE Usage: {} ⚡️{}",
        units::percent1(ane_active_ratio * 100.0),
        units::watts2(metrics.ane_w)
    );
    let gauge = Gauge::default()
        .block(Block::default().title(title))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Gray))
        .ratio(ane_active_ratio);

    f.render_widget(gauge, top_right_area);

    // ANE Usage Sparklines.
    let sig = history.get("ane_active_ratio").unwrap();
    let sparkline = Sparkline::default()
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(bottom_right_area.width as usize))
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(sparkline, bottom_right_area);
}

/// Draw the Package power block.
fn draw_package_power_block<B>(
    f: &mut Frame<B>,
    metrics: &PowerMetrics,
    history: &History,
    area: Rect,
) where
    B: Backend,
{
    let block = Block::default().title(" Package ").borders(Borders::ALL);
    f.render_widget(block, area);

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(SPARKLINE_HEIGHT)].as_ref())
        .margin(1)
        .split(area);
    let title_area = vertical_chunks[0];
    let sparkline_area = vertical_chunks[1];

    let sig = history.get("package_w").unwrap();
    let title = format!(
        "CPU+GPU+ANE: ⚡️{} (peak: {})",
        units::watts2(metrics.package_w),
        units::watts2(sig.peak)
    );
    let text = Paragraph::new(Text::from(title));
    f.render_widget(text, title_area);

    // Package Power Usage Sparklines.
    let sparkline = Sparkline::default()
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(sparkline_area.width as usize))
        .max(sig.max as u64);
    f.render_widget(sparkline, sparkline_area);
}

/// Draw the Thermal Pressure block.
fn draw_thermal_pressure_block<B>(f: &mut Frame<B>, metrics: &PowerMetrics, area: Rect)
where
    B: Backend,
{
    let color = match metrics.thermal_pressure.as_str() {
        "Nominal" => Color::Green,
        _ => Color::Yellow,
    };
    let text = Spans::from(vec![
        Span::raw("Thermal Pressure: "),
        Span::styled(&metrics.thermal_pressure, Style::default().fg(color)),
    ]);
    let paragraph =
        Paragraph::new(text).block(Block::default().title(" Thermals ").borders(Borders::ALL));
    f.render_widget(paragraph, area);
}
