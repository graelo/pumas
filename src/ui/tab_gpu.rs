//! GPU tab.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, LineGauge, Paragraph, Row, Sparkline, Table},
    Frame,
};

use crate::{
    app::{App, AppColors, History, HistoryExt},
    metric_key::MetricKey,
    metrics::{GpuMetrics, Metrics},
    units,
};

const SPARKLINE_MAX_OVERSHOOT: f32 = 1.05;
const ACTIVITY_HISTORY_LENGTH: u16 = 8;
const FREQUENCY_LABEL_WIDTH: u16 = 6; // "freq: "
const FREQUENCY_VALUE_WIDTH: u16 = 10; // "1070 MHz "
const FREQUENCY_HISTORY_LENGTH: u16 = 8;
const POWER_HISTORY_LENGTH: u16 = 8;

/// Draw the GPU tab.
pub(crate) fn draw_gpu_tab(f: &mut Frame, app: &App, area: Rect) {
    let metrics = match &app.metrics {
        Some(metrics) => metrics,
        None => return,
    };

    let gpu_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // GPU activity + frequency + power
            Constraint::Length(3), // Thermal pressure
            Constraint::Length(5), // Frequency table
            Constraint::Min(0),    // Remaining space
        ])
        .split(area);
    let gpu_area = gpu_chunks[0];
    let thermal_area = gpu_chunks[1];
    let freq_table_area = gpu_chunks[2];

    draw_gpu(f, metrics, &app.history, &app.colors, gpu_area);
    draw_thermal_pressure(f, metrics, &app.colors, thermal_area);
    draw_freq_table(f, &metrics.gpu, freq_table_area);
}

fn draw_gpu(f: &mut Frame, metrics: &Metrics, history: &History, colors: &AppColors, area: Rect) {
    let block = Block::default().title("GPU: ").borders(Borders::ALL);
    f.render_widget(block, area);

    let gpu = &metrics.gpu;

    // Split into two rows: activity+frequency and power+peak.
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .margin(1)
        .split(area);
    let top_row = vertical_chunks[0];
    let bottom_row = vertical_chunks[1];

    //
    // Top row: GPU activity + frequency.
    //

    let activity_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(top_row);

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

    let sig = history.get_or_default(&MetricKey::GpuActivePercent);
    let activity_history_sparkline = Sparkline::default()
        .style(
            Style::default()
                .fg(colors.history_fg())
                .bg(colors.history_bg()),
        )
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(ACTIVITY_HISTORY_LENGTH as usize))
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(activity_history_sparkline, acti_histo_area);

    let active_ratio = gpu.active_ratio;
    let label = format!("{:.1}%", active_ratio * 100.0);
    let gauge = LineGauge::default()
        .filled_style(Style::default().fg(colors.gauge_fg()).bg(colors.gauge_bg()))
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

    let sig = history.get_or_default(&MetricKey::GpuFreqPercent);
    let freq_history_sparkline = Sparkline::default()
        .style(
            Style::default()
                .fg(colors.history_fg())
                .bg(colors.history_bg()),
        )
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(FREQUENCY_HISTORY_LENGTH as usize))
        // .data(&[1, 4, 3, 4, 2, 3, 8, 4])
        // .max(10);
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(freq_history_sparkline, freq_hist_area);

    let freq_value_text = units::mhz(gpu.freq_mhz);
    let par = Paragraph::new(Span::from(freq_value_text));
    f.render_widget(par, freq_value_area);

    let gauge = LineGauge::default()
        .filled_style(Style::default().fg(colors.gauge_fg()).bg(colors.gauge_bg()))
        .line_set(symbols::line::THICK)
        // .label(label)
        .ratio(gpu.freq_ratio());
    f.render_widget(gauge, freq_gauge_area);

    //
    // Bottom row: GPU power + peak values.
    //

    let power_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(bottom_row);
    let power_area = power_chunks[0];
    let peak_area = power_chunks[1];

    // GPU power with sparkline.
    let power_inner_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(POWER_HISTORY_LENGTH + 1),
            Constraint::Min(0),
        ])
        .split(power_area);
    let power_hist_area = power_inner_chunks[0];
    let power_value_area = power_inner_chunks[1];

    let sig = history.get_or_default(&MetricKey::GpuPowerW);
    let power_history_sparkline = Sparkline::default()
        .style(
            Style::default()
                .fg(colors.history_fg())
                .bg(colors.history_bg()),
        )
        .bar_set(symbols::bar::NINE_LEVELS)
        .data(sig.as_slice_last_n(POWER_HISTORY_LENGTH as usize))
        .max((SPARKLINE_MAX_OVERSHOOT * sig.max) as u64);
    f.render_widget(power_history_sparkline, power_hist_area);

    let power_value_text = units::watts2(metrics.consumption.gpu_w);
    let par = Paragraph::new(Span::from(power_value_text));
    f.render_widget(par, power_value_area);

    // Peak values display.
    let sig_activity = history.get_or_default(&MetricKey::GpuActivePercent);
    let sig_power = history.get_or_default(&MetricKey::GpuPowerW);
    let peak_text = format!(
        "Peak: {} | {}",
        units::percent1(sig_activity.peak),
        units::watts2(sig_power.peak)
    );
    let par = Paragraph::new(Span::from(peak_text));
    f.render_widget(par, peak_area);
}

/// Draw thermal pressure indicator with color coding.
fn draw_thermal_pressure(f: &mut Frame, metrics: &Metrics, colors: &AppColors, area: Rect) {
    let color = match metrics.thermal_pressure.as_str() {
        "Nominal" => colors.accent(),
        _ => Color::Yellow,
    };
    let text = Line::from(vec![
        Span::raw("Pressure: "),
        Span::styled(&metrics.thermal_pressure, Style::default().fg(color)),
    ]);
    let paragraph =
        Paragraph::new(text).block(Block::default().title(" Thermals ").borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

fn draw_freq_table(f: &mut Frame, gpu_metrics: &GpuMetrics, area: Rect) {
    let gpu_freq_values = gpu_metrics
        .frequencies_mhz()
        .iter()
        .map(|f| format!("{:4}", *f))
        .collect::<Vec<_>>()
        .join(" ");
    let row_content = [
        ("GPU:", gpu_freq_values),
        ("", "".into()),
        (
            "Note:",
            "Hardware-wise, GPUs quickly shift between the above frequencies.".into(),
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
