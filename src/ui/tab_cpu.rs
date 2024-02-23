//! CPU cores tab.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Block, Borders, Cell, LineGauge, Paragraph, Row, Sparkline, Table},
    Frame,
};

use crate::{
    app::{App, History},
    metrics::{ClusterMetrics, CpuMetrics, Metrics},
    units,
};

const CPU_BLOCK_HEIGHT: u16 = 1;
const SPARKLINE_MAX_OVERSHOOT: f32 = 1.05;
const ACTIVITY_HISTORY_LENGTH: u16 = 8;
const FREQUENCY_LABEL_WIDTH: u16 = 6; // "freq: "
const FREQUENCY_VALUE_WIDTH: u16 = 10; // "1070 MHz "
const FREQUENCY_HISTORY_LENGTH: u16 = 8;
const FREQUENCY_TABLE_HEIGHT: u16 = 4;

/// Draw the per-core usage, and per-core frequency distribution.
///
/// Pumas v0.0.10                                                             Apple M2 Max (cores: 4E+8P+38GPU)
/// ┌──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
/// │ Overview │ CPU │ GPU │ SoC                                                                               │
/// └──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
/// ┌ E-Cluster: ──────────────────────────────────────────────────────────────────────────────────────────────┐
/// │ 0 -          6.9% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq:   ▁ ▁    1085 MHz  11% ━━━━━━━━━━━━━━━━━━━━━━│
/// │ 1 -          6.9% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq:     ▁    1009 MHz  6% ━━━━━━━━━━━━━━━━━━━━━━━│
/// │ 2 -          6.9% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq:   ▁ ▁▁   1047 MHz  9% ━━━━━━━━━━━━━━━━━━━━━━━│
/// │ 3 -          3.0% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq: ▁ ▁▁▁    1011 MHz  7% ━━━━━━━━━━━━━━━━━━━━━━━│
/// └──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
/// ┌ P0-Cluster: ─────────────────────────────────────────────────────────────────────────────────────────────┐
/// │ 4 - ▇▇▇      2.0% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq: ▇▇▇▇▁    734 MHz   1% ━━━━━━━━━━━━━━━━━━━━━━━│
/// │ 5 -          2.0% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq: ▇▇▇▇     724 MHz   1% ━━━━━━━━━━━━━━━━━━━━━━━│
/// │ 6 -          0.0% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq: ▇▇▇▇     711 MHz   0% ━━━━━━━━━━━━━━━━━━━━━━━│
/// │ 7 -          0.0% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq: ▇▇▇▇     706 MHz   0% ━━━━━━━━━━━━━━━━━━━━━━━│
/// └──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
/// ┌ P1-Cluster: ─────────────────────────────────────────────────────────────────────────────────────────────┐
/// │ 8 -          0.0% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq:  ▇       708 MHz   0% ━━━━━━━━━━━━━━━━━━━━━━━│
/// │ 9 -          0.0% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq:          703 MHz   0% ━━━━━━━━━━━━━━━━━━━━━━━│
/// │10 -          1.0% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq:          702 MHz   0% ━━━━━━━━━━━━━━━━━━━━━━━│
/// │11 -          0.0% ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━freq:          702 MHz   0% ━━━━━━━━━━━━━━━━━━━━━━━│
/// └──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
/// ┌Frequencies───────────────────────────────────────────────────────────────────────────────────────────────┐
/// │E-Cluster:  912 1284 1752 2004 2256 2424                                                                  │
/// │P-Cluster:  702  948 1188 1452 1704 1968 2208 2400 2568 2724 2868 3000 3132 3264 3360 3408 3504 3528 3696 │
/// │                                                                                                          │
/// │Note:      Hardware-wise, CPUs quickly shift between the above frequencies.                               │
/// └──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
///
pub(crate) fn draw_cpu_tab(f: &mut Frame, app: &App, area: Rect) {
    let metrics = match &app.metrics {
        Some(metrics) => metrics,
        None => return,
    };

    let accent_color = app.accent_color();
    let gauge_bg_color = app.gauge_bg_color();

    let constraints = metrics
        // E-Clusters
        .e_clusters
        .iter()
        .map(|cl| Constraint::Length(2 + CPU_BLOCK_HEIGHT * cl.cpus.len() as u16))
        // P-Clusters
        .chain(
            metrics
                .p_clusters
                .iter()
                .map(|cl| Constraint::Length(2 + CPU_BLOCK_HEIGHT * cl.cpus.len() as u16)),
        )
        // Frequency table
        .chain(std::iter::once(Constraint::Length(
            2 + FREQUENCY_TABLE_HEIGHT,
        )))
        // Spacer
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

    let freq_table_area = clu_area_iter.next().unwrap();
    draw_freq_table(f, metrics, *freq_table_area);
}

fn draw_cpu_cluster(
    f: &mut Frame,
    cluster: &ClusterMetrics,
    history: &History,
    accent_color: Color,
    gauge_bg_color: Color,
    area: Rect,
) {
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

fn draw_cpu(
    f: &mut Frame,
    cpu: &CpuMetrics,
    history: &History,
    accent_color: Color,
    gauge_bg_color: Color,
    area: Rect,
) {
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

    let sig_name = format!("{}_active_percent", cpu.id);
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

fn draw_freq_table(f: &mut Frame, metrics: &Metrics, area: Rect) {
    let e_cluster_frequencies = metrics.e_clusters[0].cpus[0].frequencies_mhz();
    let p_cluster_frequencies = metrics.p_clusters[0].cpus[0].frequencies_mhz();

    let e_clus = e_cluster_frequencies
        .iter()
        .map(|f| format!("{:4}", *f))
        .collect::<Vec<_>>()
        .join(" ");
    let p_clus = p_cluster_frequencies
        .iter()
        .map(|f| format!("{:4}", *f))
        .collect::<Vec<_>>()
        .join(" ");
    let row_content = vec![
        ("E-Cluster:", e_clus),
        ("P-Cluster:", p_clus),
        ("", "".into()),
        (
            "Note:",
            "Hardware-wise, CPUs quickly shift between the above frequencies.".into(),
        ),
    ];

    let rows = row_content.iter().map(|(left, ref right)| {
        Row::new(vec![
            Cell::from(Span::from(*left)),
            Cell::from(Span::styled(
                right.as_str(),
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ])
    });

    let label_width = 10;
    let array_width = area.width - label_width - 2;
    let constraints = [
        Constraint::Length(label_width),
        Constraint::Length(array_width),
    ];
    let table = Table::new(rows, constraints)
        .block(Block::default().borders(Borders::ALL).title("Frequencies"));

    f.render_widget(table, area);
}
