//! Overview tab.

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};

use crate::{
    app::{App, History},
    metrics,
    modules::soc::SocInfo,
    units,
};

const CLUSTER_SPACING: u16 = 1; // Space between CPU blocks.
const SPARKLINE_HEIGHT: u16 = 3;
const SPARKLINE_MAX_OVERSHOOT: f32 = 1.05; // Prevent sparklines from touching gauges.
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
    let num_clusters_blocks = (num_blocks_for(metrics.e_clusters.len())
        + num_blocks_for(metrics.p_clusters.len())) as u16;

    let cls_block_height = GAUGE_HEIGHT + SPARKLINE_HEIGHT;
    let cpu_block_height =
        cls_block_height * num_clusters_blocks + (num_clusters_blocks - 1) * CLUSTER_SPACING;
    let gpu_block_height = GAUGE_HEIGHT + SPARKLINE_HEIGHT;
    let pkg_block_height = PKG_TEXT_HEIGHT + SPARKLINE_HEIGHT;
    let thr_block_height = THR_TEXT_HEIGHT;

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2 + cpu_block_height), // Borders & CPU clusters blocks
            Constraint::Length(2 + gpu_block_height), // Borders & GPU ANE block
            Constraint::Length(2 + pkg_block_height), // Borders & Package block
            Constraint::Length(2 + thr_block_height), // Borders & Thermal block
            Constraint::Min(0),
        ])
        .split(area);
    let cpu_area = vertical_chunks[0];
    let gpu_area = vertical_chunks[1];
    let pkg_area = vertical_chunks[2];
    let thr_area = vertical_chunks[3];

    draw_cpu_clusters_usage_block(
        f,
        metrics,
        &app.history,
        app.accent_color(),
        app.gauge_bg_color(),
        cpu_area,
    );
    draw_gpu_ane_usage_block(
        f,
        metrics,
        &app.soc_info,
        &app.history,
        app.accent_color(),
        app.gauge_bg_color(),
        gpu_area,
    );
    draw_package_power_block(f, metrics, &app.history, app.accent_color(), pkg_area);
    draw_thermal_pressure_block(f, metrics, app.accent_color(), thr_area);
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
/// │                                                                                         │
/// │P0-Cluster: 25.6 % @ 1027 MHz                 P1-Cluster: 7.0 % @ 1729 MHz               │
/// │------------------- 26% --------------------  ------------------- 7% --------------------│
/// │   ▄▃▅▆▂▁ ▆▇▇▃▄▅▅▅▆  ▆▃                                                                  │
/// │▁▂▄██████▇█████████▇▃███▄▂▁█   █                                                         │
/// │████████████████████████████▇▅▆█▆▄▅▅▆▄▆▅▅▇▇▅  ▂▄▃▂▄▃▂▂▁▃▃▁▂▁▂▂▂▃▂ ▂▁▃▂▂▂▂▁ ▂▁▁▁  ▁▁▁▁▁▁▁▁│
/// └─────────────────────────────────────────────────────────────────────────────────────────┘
///
fn draw_cpu_clusters_usage_block<B>(
    f: &mut Frame<B>,
    metrics: &metrics::Metrics,
    history: &History,
    accent_color: Color,
    gauge_bg_color: Color,
    area: Rect,
) where
    B: Backend,
{
    let num_cluster_blocks =
        num_blocks_for(metrics.e_clusters.len()) + num_blocks_for(metrics.p_clusters.len());

    let sig = history.get("cpu_w").unwrap();
    let title = "CPU Clusters";
    let title_with_power = format!(
        " {title}: 󱐋 {} (peak: {})",
        units::watts2(metrics.consumption.cpu_w),
        units::watts2(sig.peak)
    );
    let block = Block::default()
        .title(title_with_power)
        .borders(Borders::ALL);
    f.render_widget(block, area);

    let constraints = (0..num_cluster_blocks)
        .map(|k| {
            Constraint::Length(
                GAUGE_HEIGHT
                    + SPARKLINE_HEIGHT
                    + if k < num_cluster_blocks - 1 {
                        CLUSTER_SPACING
                    } else {
                        0
                    },
            )
        }) // block height
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
        draw_cluster_overall_metrics(
            f,
            &metrics.e_clusters[0],
            history,
            accent_color,
            gauge_bg_color,
            *area,
        );
    } else {
        draw_cluster_pair_overall_metrics(
            f,
            &metrics.e_clusters[0],
            &metrics.e_clusters[1],
            history,
            accent_color,
            gauge_bg_color,
            *area,
        );
    }

    // Draw the metrics for the Performance cluster (or clusters).
    let area = clu_area_iter.next().unwrap();
    if metrics.p_clusters.len() == 1 {
        draw_cluster_overall_metrics(
            f,
            &metrics.p_clusters[0],
            history,
            accent_color,
            gauge_bg_color,
            *area,
        );
    } else {
        draw_cluster_pair_overall_metrics(
            f,
            &metrics.p_clusters[0],
            &metrics.p_clusters[1],
            history,
            accent_color,
            gauge_bg_color,
            *area,
        );
    }
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
    cluster: &metrics::ClusterMetrics,
    history: &History,
    accent_color: Color,
    gauge_bg_color: Color,
    area: Rect,
) where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(GAUGE_HEIGHT),
            Constraint::Length(SPARKLINE_HEIGHT),
            Constraint::Max(CLUSTER_SPACING),
        ])
        .split(area);
    let top_area = chunks[0];
    let bottom_area = chunks[1];

    // Cluster cores Usage Gauge.
    let sig_name = format!("{}_active_ratio", cluster.name);
    let sig = history.get(&sig_name).unwrap();
    let title = format!(
        "{}: {} @ {} (peak: {})",
        cluster.name,
        units::percent1(cluster.active_ratio() * 100.0),
        units::mhz(cluster.freq_mhz),
        units::percent1(sig.peak)
    );
    let gauge = Gauge::default()
        .block(Block::default().title(title))
        .gauge_style(Style::default().fg(accent_color).bg(gauge_bg_color))
        .ratio(cluster.active_ratio() as f64);

    f.render_widget(gauge, top_area);

    // Cluster cores Sparklines.
    let sparkline = Sparkline::default()
        .style(Style::default().fg(accent_color))
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
    left_cluster: &metrics::ClusterMetrics,
    right_cluster: &metrics::ClusterMetrics,
    history: &History,
    accent_color: Color,
    gauge_bg_color: Color,
    area: Rect,
) where
    B: Backend,
{
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 2),
            Constraint::Length(2), // space
            Constraint::Ratio(1, 2),
        ])
        // .horizontal_margin(1)
        .split(area);
    let left_area = horizontal_chunks[0];
    let right_area = horizontal_chunks[2];

    draw_cluster_overall_metrics(
        f,
        left_cluster,
        history,
        accent_color,
        gauge_bg_color,
        left_area,
    );
    draw_cluster_overall_metrics(
        f,
        right_cluster,
        history,
        accent_color,
        gauge_bg_color,
        right_area,
    );
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
    metrics: &metrics::Metrics,
    soc_info: &SocInfo,
    history: &History,
    accent_color: Color,
    gauge_bg_color: Color,
    area: Rect,
) where
    B: Backend,
{
    let block = Block::default().title(" GPU & ANE ").borders(Borders::ALL);
    f.render_widget(block, area);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 2),
            Constraint::Length(2), // space
            Constraint::Ratio(1, 2),
        ])
        .margin(1)
        .split(area);
    let left_area = horizontal_chunks[0];
    let right_area = horizontal_chunks[2];

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(9)])
        .split(left_area);
    let top_left_area = left_chunks[0];
    let bottom_left_area = left_chunks[1];

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(9)])
        .split(right_area);
    let top_right_area = right_chunks[0];
    let bottom_right_area = right_chunks[1];

    // left: GPU.
    let gpu = &metrics.gpu;
    let sig = history.get("gpu_active_ratio").unwrap();
    let sig_gpu_power = history.get("gpu_w").unwrap();
    let title = format!(
        "GPU Usage: {} @ {} 󱐋 {} (peak {} 󱐋 {})",
        units::percent1(gpu.active_ratio * 100.0),
        units::mhz(gpu.freq_mhz),
        units::watts2(metrics.consumption.gpu_w),
        units::percent1(sig.peak),
        units::watts2(sig_gpu_power.peak)
    );
    let gauge = Gauge::default()
        .block(Block::default().title(title))
        .gauge_style(
            Style::default().fg(accent_color).bg(gauge_bg_color),
            // .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        )
        .ratio(gpu.active_ratio);

    f.render_widget(gauge, top_left_area);

    // GPU Usage Sparklines.
    let sparkline = Sparkline::default()
        .style(Style::default().fg(accent_color))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(bottom_left_area.width as usize))
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(sparkline, bottom_left_area);

    // Right: ANE.
    let ane_active_ratio = metrics.consumption.ane_w as f64 / soc_info.max_ane_w;
    let sig = history.get("ane_active_ratio").unwrap();
    let sig_ane_power = history.get("ane_w").unwrap();
    let title = format!(
        "ANE Usage: {} 󱐋 {} (peak {} 󱐋 {})",
        units::percent1(ane_active_ratio * 100.0),
        units::watts2(metrics.consumption.ane_w),
        units::percent1(sig.peak),
        units::watts2(sig_ane_power.peak)
    );
    let gauge = Gauge::default()
        .block(Block::default().title(title))
        .gauge_style(Style::default().fg(accent_color).bg(gauge_bg_color))
        .ratio(ane_active_ratio);

    f.render_widget(gauge, top_right_area);

    // Sparklines for the ANE usage.
    let sparkline = Sparkline::default()
        .style(Style::default().fg(accent_color))
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
    metrics: &metrics::Metrics,
    history: &History,
    accent_color: Color,
    area: Rect,
) where
    B: Backend,
{
    let block = Block::default().title(" Package ").borders(Borders::ALL);
    f.render_widget(block, area);

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(SPARKLINE_HEIGHT)])
        .margin(1)
        .split(area);
    let title_area = vertical_chunks[0];
    let sparkline_area = vertical_chunks[1];

    let sig = history.get("package_w").unwrap();
    let title = format!(
        "CPU+GPU+ANE: 󱐋 {} (peak: {})",
        units::watts2(metrics.consumption.package_w),
        units::watts2(sig.peak)
    );
    let text = Paragraph::new(Text::from(title));
    f.render_widget(text, title_area);

    // Sparklines for the Package total usage.
    let sparkline = Sparkline::default()
        .style(Style::default().fg(accent_color))
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
fn draw_thermal_pressure_block<B>(
    f: &mut Frame<B>,
    metrics: &metrics::Metrics,
    accent_color: Color,
    area: Rect,
) where
    B: Backend,
{
    let color = match metrics.thermal_pressure.as_str() {
        "Nominal" => accent_color,
        _ => Color::Yellow,
    };
    let text = Line::from(vec![
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
