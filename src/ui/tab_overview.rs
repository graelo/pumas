//! Overview tab.

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
    modules::powermetrics,
    modules::soc::SocInfo,
    units,
};

const SPARKLINE_HEIGHT: u16 = 3;
const SPARKLINE_MAX_OVERSHOOT: f32 = 1.05;
const GAUGE_HEIGHT: u16 = 2;
const PKG_TEXT_HEIGHT: u16 = 1;
const THR_TEXT_HEIGHT: u16 = 1;

/// Draw the Overview tab.
///
/// Pumas v0.0.3                                                  Apple M1 (cores: 4E+4P+8GPU)
/// ┌─────────────────────────────────────────────────────────────────────────────────────────┐
/// │ Overview │ CPU │ GPU │ SoC                                                              │
/// └─────────────────────────────────────────────────────────────────────────────────────────┘
/// ┌ CPU: 160.13 mW ─────────────────────────────────────────────────────────────────────────┐
/// │E-Cluster: 28.9 % @ 1043 MHz                  P-Cluster: 8.7 % @ 1891 MHz                │
/// │                    29%                                           9%                     │
/// │▆▃▅▆▆▅▂▄  ▆▂▅▂▄▄▃▄▄▅ ▄▄▄▅  ▄▄ ▅▃                                                         │
/// │████████▆▃██████████▆████▇███▇██▂▁                                           ▆           │
/// │██████████████████████████████████▅▅▄▄▄▆▇▆▆▆  ▃▃▂▂▃▂▂▁▂▂▅▃▃▂▃▃▂▃▃▄▃▁▂▃▁▁▁▂▃▃▁█▅▁▂▂▁▂▂▃▂▂▁│
/// └─────────────────────────────────────────────────────────────────────────────────────────┘
/// ┌ GPU & ANE ──────────────────────────────────────────────────────────────────────────────┐
/// │GPU Usage: 1.6 % @ 717 MHz ⚡️16.80 mW         ANE Usage: 0.0 % ⚡️0.00 W                  │
/// │                     2%                                           0%                     │
/// │                                                                                         │
/// │                                                                                         │
/// │▂▂▃        ▂▁▁ ▃    ▄    ▁      ▁                                                        │
/// └─────────────────────────────────────────────────────────────────────────────────────────┘
/// ┌ Package ────────────────────────────────────────────────────────────────────────────────┐
/// │CPU+GPU+ANE: ⚡️176.93 mW (peak: 1.58 W)                                                  │
/// │                                                                                         │
/// │                                                                                         │
/// │                                                                                         │
/// └─────────────────────────────────────────────────────────────────────────────────────────┘
/// ┌ Thermals ───────────────────────────────────────────────────────────────────────────────┐
/// │Thermal Pressure: Nominal                                                                │
/// └─────────────────────────────────────────────────────────────────────────────────────────┘
///
pub(crate) fn draw_overview_tab<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let metrics = match &app.metrics {
        Some(metrics) => metrics,
        None => return,
    };

    // Number of horizontal blocks for the CPU clusters.
    let num_clusters_blocks =
        num_blocks_for(metrics.e_clusters.len()) + num_blocks_for(metrics.p_clusters.len());

    let cpu_block_height = GAUGE_HEIGHT + SPARKLINE_HEIGHT;
    let gpu_block_height = GAUGE_HEIGHT + SPARKLINE_HEIGHT;
    let pkg_block_height = PKG_TEXT_HEIGHT + SPARKLINE_HEIGHT;
    let thr_block_height = THR_TEXT_HEIGHT;

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2 + cpu_block_height * num_clusters_blocks as u16), // Borders & CPU clusters blocks
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
    draw_gpu_ane_usage_block(f, metrics, &app.soc_info, &app.history, gpu_area);
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
///
/// ┌ CPU: 124.32 mW ─────────────────────────────────────────────────────────────────────────┐
/// │E0-Cluster: 25.6 % @ 1027 MHz                 E1-Cluster: 7.0 % @ 1729 MHz               │
/// │------------------- 26% --------------------  ------------------- 7% --------------------│
/// │   ▄▃▅▆▂▁ ▆▇▇▃▄▅▅▅▆  ▆▃                                                                  │
/// │▁▂▄██████▇█████████▇▃███▄▂▁█   █                                                         │
/// │████████████████████████████▇▅▆█▆▄▅▅▆▄▆▅▅▇▇▅  ▂▄▃▂▄▃▂▂▁▃▃▁▂▁▂▂▂▃▂ ▂▁▃▂▂▂▂▁ ▂▁▁▁  ▁▁▁▁▁▁▁▁│
/// │P0-Cluster: 25.6 % @ 1027 MHz                 P1-Cluster: 7.0 % @ 1729 MHz               │
/// │------------------- 26% --------------------  ------------------- 7% --------------------│
/// │   ▄▃▅▆▂▁ ▆▇▇▃▄▅▅▅▆  ▆▃                                                                  │
/// │▁▂▄██████▇█████████▇▃███▄▂▁█   █                                                         │
/// │████████████████████████████▇▅▆█▆▄▅▅▆▄▆▅▅▇▇▅  ▂▄▃▂▄▃▂▂▁▃▃▁▂▁▂▂▂▃▂ ▂▁▃▂▂▂▂▁ ▂▁▁▁  ▁▁▁▁▁▁▁▁│
/// └─────────────────────────────────────────────────────────────────────────────────────────┘
///
fn draw_cpu_clusters_usage_block<B>(
    f: &mut Frame<B>,
    metrics: &powermetrics::Metrics,
    history: &History,
    area: Rect,
) where
    B: Backend,
{
    let num_cluster_blocks =
        num_blocks_for(metrics.e_clusters.len()) + num_blocks_for(metrics.p_clusters.len());

    let title = "CPU Clusters";
    let title_with_power = format!(" {title}: {} ", units::watts2(metrics.cpu_w));
    let block = Block::default()
        .title(title_with_power)
        .borders(Borders::ALL);
    f.render_widget(block, area);

    let constraints = (0..num_cluster_blocks)
        .map(|_| Constraint::Length(GAUGE_HEIGHT + SPARKLINE_HEIGHT)) // block height
        .collect::<Vec<_>>();
    let cpu_cluster_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .margin(1)
        .split(area);

    let mut clu_area_iter = cpu_cluster_chunks.iter();

    // TODO: refactor this.
    // Draw the metrics for the Efficiency cluster (or clusters).
    let area = clu_area_iter.next().unwrap();
    if metrics.e_clusters.len() == 1 {
        draw_cluster_overall_metrics(f, &metrics.e_clusters[0], history, *area);
    } else {
        draw_cluster_pair_overall_metrics(
            f,
            &metrics.e_clusters[0],
            &metrics.e_clusters[1],
            history,
            *area,
        );
    }

    // Draw the metrics for the Performance cluster (or clusters).
    let area = clu_area_iter.next().unwrap();
    if metrics.p_clusters.len() == 1 {
        draw_cluster_overall_metrics(f, &metrics.p_clusters[0], history, *area);
    } else {
        draw_cluster_pair_overall_metrics(
            f,
            &metrics.p_clusters[0],
            &metrics.p_clusters[1],
            history,
            *area,
        );
    }
    // for (clu_area, (e_cluster, p_cluster)) in zip(
    //     &*cpu_cluster_chunks,
    //     zip(&metrics.e_clusters, &metrics.p_clusters),
    // ) {
    //     draw_cluster_pair_overall_metrics(f, e_cluster, p_cluster, history, *clu_area);
    // }
}

/// Draw the overall metrics for a single CPU cluster.
///
/// E0-Cluster: 26.3 % @ 1009 MHz
/// ------------------------------------------- 26% -----------------------------------------
///
///  ▁ ▄▅▄ ▁    ▂   ▃▃                                          ▃   ▅
/// ██▅█████▄▆▄▅█▄▆███▆▇▅▇▅▅▅█▆█▃▅▃▅▄▅▅▅▅█▄▅▃▅▅▆█▅▃▄▁▂▁▄▃▁▇▃▂▃▁▆█▂▃▆█▂▂▂▂▂▃▁▂▁▁ ▂▂▂▂▂▃▂▁▁▂▂▃▂
///
fn draw_cluster_overall_metrics<B>(
    f: &mut Frame<B>,
    cluster: &powermetrics::ClusterMetrics,
    history: &History,
    area: Rect,
) where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(GAUGE_HEIGHT),
                Constraint::Length(SPARKLINE_HEIGHT),
            ]
            .as_ref(),
        )
        .split(area);
    let top_area = chunks[0];
    let bottom_area = chunks[1];

    // Cluster cores Usage Gauge.
    let title = format!(
        "{}: {} @ {}",
        cluster.name,
        units::percent1(cluster.active_ratio() * 100.0),
        units::mhz(cluster.freq_mhz),
    );
    let gauge = Gauge::default()
        .block(Block::default().title(title))
        .gauge_style(Style::default().fg(Color::Green).bg(Color::Gray))
        .ratio(cluster.active_ratio() as f64);

    f.render_widget(gauge, top_area);

    // Cluster cores Sparklines.
    let sig_name = format!("{}_active_ratio", cluster.name);
    let sig = history.get(&sig_name).unwrap();
    let sparkline = Sparkline::default()
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(bottom_area.width as usize))
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(sparkline, bottom_area);
}

/// Draw the overall metrics for a pair of CPU clusters.
///
/// For instance:
///
/// E0-Cluster: 26.3 % @ 1009 MHz                 E1-Cluster: 12.1 % @ 1873 MHz
/// ------------------- 26% --------------------  ------------------- 12% -------------------
///
///  ▁ ▄▅▄ ▁    ▂   ▃▃                                          ▃   ▅
/// ██▅█████▄▆▄▅█▄▆███▆▇▅▇▅▅▅█▆█▃▅▃▅▄▅▅▅▅█▄▅▃▅▅▆  ▃▄▁▂▁▄▃▁▇▃▂▃▁▆█▂▃▆█▂▂▂▂▂▃▁▂▁▁ ▂▂▂▂▂▃▂▁▁▂▂▃▂
///
fn draw_cluster_pair_overall_metrics<B>(
    f: &mut Frame<B>,
    left_cluster: &powermetrics::ClusterMetrics,
    right_cluster: &powermetrics::ClusterMetrics,
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

    draw_cluster_overall_metrics(f, left_cluster, history, left_area);
    draw_cluster_overall_metrics(f, right_cluster, history, right_area);
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
///
fn draw_gpu_ane_usage_block<B>(
    f: &mut Frame<B>,
    metrics: &powermetrics::Metrics,
    soc_info: &SocInfo,
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
    let ane_active_ratio = metrics.ane_w as f64 / soc_info.max_ane_w;
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

    // Sparklines for the ANE usage.
    let sig = history.get("ane_active_ratio").unwrap();
    let sparkline = Sparkline::default()
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(bottom_right_area.width as usize))
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(sparkline, bottom_right_area);
}

/// Draw the Package power block.
///
/// ┌ Package ────────────────────────────────────────────────────────────────────────────────┐
/// │CPU+GPU+ANE: ⚡️95.48 mW (peak: 3.17 W)                                                   │
/// │                                                                                         │
/// │                                                                                         │
/// │                                    ▁                                           ▁▁▁▁▁▁▁  │
/// └─────────────────────────────────────────────────────────────────────────────────────────┘
///
fn draw_package_power_block<B>(
    f: &mut Frame<B>,
    metrics: &powermetrics::Metrics,
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

    // Sparklines for the Package total usage.
    let sparkline = Sparkline::default()
        .style(Style::default().fg(Color::Green))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(sparkline_area.width as usize))
        .max(sig.max as u64);
    f.render_widget(sparkline, sparkline_area);
}

/// Draw the Thermal Pressure block.
///
/// ┌ Thermals ───────────────────────────────────────────────────────────────────────────────┐
/// │Thermal Pressure: Nominal                                                                │
/// └─────────────────────────────────────────────────────────────────────────────────────────┘
///
fn draw_thermal_pressure_block<B>(f: &mut Frame<B>, metrics: &powermetrics::Metrics, area: Rect)
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

/// Compute the number of blocks for a given cluster.
fn num_blocks_for(count: usize) -> usize {
    (count as f32 / 2.0).ceil() as usize
}
