//! CPU cores tab.

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Block, Borders, LineGauge, Paragraph, Sparkline},
    Frame,
};

use crate::{
    app::{App, History},
    metrics::{ClusterMetrics, CpuMetrics},
    units,
};

const CPU_BLOCK_HEIGHT: u16 = 1;
const SPARKLINE_MAX_OVERSHOOT: f32 = 1.05;
const ACTIVITY_HISTORY_LENGTH: u16 = 8;
const FREQUENCY_LABEL_WIDTH: u16 = 6; // "freq: "
const FREQUENCY_VALUE_WIDTH: u16 = 10; // "1070 MHz "
const FREQUENCY_HISTORY_LENGTH: u16 = 8;

/// Draw the per-core usage, and per-core frequency distribution.
pub(crate) fn draw_cpu_tab<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    // let text = Text::from("Coming soon: CPU Power and frequency distribution.");
    // let par = Paragraph::new(text).block(Block::default().title("Paragraph").borders(Borders::ALL));
    // f.render_widget(par, area);

    let metrics = match &app.metrics {
        Some(metrics) => metrics,
        None => return,
    };

    let accent_color = app.accent_color();
    let gauge_bg_color = app.gauge_bg_color();

    // let chunks = Layout::default()
    //     .direction(Direction::Vertical)
    //     .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)].as_ref())
    //     // .margin(1)
    //     .split(area);
    // let top_area = chunks[0];
    // let bot_area = chunks[1];

    // let bar_chart = BarChart::default()
    //     .block(Block::default().title("BarChart").borders(Borders::ALL))
    //     .bar_width(3)
    //     .bar_gap(1)
    //     .group_gap(3)
    //     .bar_style(Style::new().fg(accent_color).bg(gauge_bg_color))
    //     .value_style(Style::new().fg(gauge_bg_color).bg(accent_color))
    //     // .label_style(Style::new().black())
    //     // .data(&[("B0", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
    //     .data(BarGroup::default().label("E-Cluster".into()).bars(&[
    //         Bar::default().label("8".into()).value(7),
    //         Bar::default().label("9".into()).value(8),
    //         Bar::default().label("10".into()).value(9),
    //         Bar::default().label("11".into()).value(10),
    //     ]))
    //     .data(BarGroup::default().label("P0-Cluster".into()).bars(&[
    //         Bar::default().label("8".into()).value(7),
    //         Bar::default().label("9".into()).value(8),
    //         Bar::default().label("10".into()).value(9),
    //         Bar::default().label("11".into()).value(10),
    //     ]))
    //     .data(BarGroup::default().label("P1-Cluster".into()).bars(&[
    //         Bar::default().label("8".into()).value(7),
    //         Bar::default().label("9".into()).value(8),
    //         Bar::default().label("10".into()).value(9),
    //         Bar::default().label("11".into()).value(10),
    //     ]))
    //     // .data(BarGroup::default().bars(&[Bar::default().value(6), Bar::default().value(8)]))
    //     .max(14);
    // f.render_widget(bar_chart, top_area);

    //
    //

    // let num_cluster_blocks = metrics.e_clusters.len() + metrics.p_clusters.len();
    // let constraints = (0..num_cluster_blocks)
    //     .map(|_| Constraint::Max(8)) // block height
    //     .collect::<Vec<_>>();

    let constraints = metrics
        .e_clusters
        .iter()
        .map(|cl| Constraint::Length(2 + CPU_BLOCK_HEIGHT * cl.cpus.len() as u16))
        .chain(
            metrics
                .p_clusters
                .iter()
                .map(|cl| Constraint::Length(2 + CPU_BLOCK_HEIGHT * cl.cpus.len() as u16)),
        )
        .chain(std::iter::once(Constraint::Min(0)))
        .collect::<Vec<_>>();

    let cpu_cluster_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    let mut clu_area_iter = cpu_cluster_chunks.iter();

    for cluster in metrics.e_clusters.iter() {
        let cluster_area = clu_area_iter.next().unwrap();
        draw_cpu_cluster(
            f,
            cluster,
            &app.history,
            accent_color,
            gauge_bg_color,
            *cluster_area,
        );
    }
    for cluster in metrics.p_clusters.iter() {
        let cluster_area = clu_area_iter.next().unwrap();
        draw_cpu_cluster(
            f,
            cluster,
            &app.history,
            accent_color,
            gauge_bg_color,
            *cluster_area,
        );
    }
}

fn draw_cpu_cluster<B>(
    f: &mut Frame<B>,
    cluster: &ClusterMetrics,
    history: &History,
    accent_color: Color,
    gauge_bg_color: Color,
    area: Rect,
) where
    B: Backend,
{
    let cluster_name = format!(" {}: ", cluster.name);
    let block = Block::default().title(cluster_name).borders(Borders::ALL);
    f.render_widget(block, area);

    let constraints = (0..cluster.cpus.len())
        .map(|_| Constraint::Length(CPU_BLOCK_HEIGHT))
        .collect::<Vec<_>>();
    let cpu_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .margin(1)
        .split(area);
    let mut cpu_area_iter = cpu_chunks.iter();

    for cpu in cluster.cpus.iter() {
        let cpu_area = cpu_area_iter.next().unwrap();
        draw_cpu(f, cpu, history, accent_color, gauge_bg_color, *cpu_area);
    }
}

fn draw_cpu<B>(
    f: &mut Frame<B>,
    cpu: &CpuMetrics,
    history: &History,
    accent_color: Color,
    gauge_bg_color: Color,
    area: Rect,
) where
    B: Backend,
{
    let horiz_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);
    let cpu_id_area = horiz_chunks[0];
    let other_area = horiz_chunks[1];

    //
    // CPU ID.
    //

    let cpu_id_text = format!("{:2} -", cpu.id);
    let par = Paragraph::new(Span::styled(cpu_id_text, Style::default().fg(accent_color)));
    f.render_widget(par, cpu_id_area);

    //
    // CPU activity.
    //

    let activity_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(other_area);

    let activity_area = activity_chunks[0];
    let frequency_area = activity_chunks[1];

    let activity_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(ACTIVITY_HISTORY_LENGTH + 1),
            Constraint::Min(0),
        ])
        .split(activity_area);
    let acti_histo_area = activity_chunks[0];
    let acti_gauge_area = activity_chunks[1];

    let sig_name = format!("{}_active_ratio", cpu.id);
    let sig = history.get(&sig_name).unwrap();
    let activity_history_sparkline = Sparkline::default()
        .style(Style::default().fg(accent_color))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(ACTIVITY_HISTORY_LENGTH as usize))
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(activity_history_sparkline, acti_histo_area);

    let active_ratio = cpu.active_ratio;
    let label = format!("{:.1}%", active_ratio * 100.0);
    let gauge = LineGauge::default()
        .gauge_style(Style::default().fg(accent_color).bg(gauge_bg_color))
        .line_set(symbols::line::THICK)
        .label(label)
        .ratio(active_ratio);
    f.render_widget(gauge, acti_gauge_area);

    //
    // Frequency distribution.
    //

    let frequency_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(FREQUENCY_LABEL_WIDTH),
            Constraint::Length(FREQUENCY_HISTORY_LENGTH + 1),
            Constraint::Length(FREQUENCY_VALUE_WIDTH),
            Constraint::Min(0),
        ])
        .split(frequency_area);
    let freq_label_area = frequency_chunks[0];
    let freq_hist_area = frequency_chunks[1];
    let freq_value_area = frequency_chunks[2];
    let freq_gauge_area = frequency_chunks[3];

    let freq_label_text = "freq:";
    let par = Paragraph::new(Span::from(freq_label_text));
    f.render_widget(par, freq_label_area);

    let sig_name = format!("{}_freq_percent", cpu.id);
    let sig = history.get(&sig_name).unwrap();
    let freq_history_sparkline = Sparkline::default()
        .style(Style::default().fg(accent_color))
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(FREQUENCY_HISTORY_LENGTH as usize))
        // .data(&[1, 4, 3, 4, 2, 3, 8, 4])
        // .max(10);
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(freq_history_sparkline, freq_hist_area);

    let freq_value_text = units::mhz(cpu.freq_mhz);
    let par = Paragraph::new(Span::from(freq_value_text));
    f.render_widget(par, freq_value_area);

    let gauge = LineGauge::default()
        .gauge_style(Style::default().fg(accent_color).bg(gauge_bg_color))
        .line_set(symbols::line::THICK)
        // .label(label)
        .ratio(cpu.freq_ratio());
    f.render_widget(gauge, freq_gauge_area);
}
