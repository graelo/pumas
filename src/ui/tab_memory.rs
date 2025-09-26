//! Memory tab.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{app::App, modules::vm_stat::VmStats, units};

/// Draw the Memory tab.
pub(crate) fn draw_memory_tab(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(18), // VM Stats section (expanded for detailed breakdown)
            Constraint::Length(8),  // Sysinfo section
            Constraint::Min(0),     // Additional space
        ])
        .margin(1)
        .split(area);

    draw_vm_stats_section(f, app, main_chunks[0]);
    draw_sysinfo_section(f, app, main_chunks[1]);
}

/// Draw the VM statistics section.
fn draw_vm_stats_section(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" VM Statistics (Activity Monitor compatible) ")
        .borders(Borders::ALL);

    if let Ok(vm_stats) = VmStats::collect() {
        let page_to_gb =
            |pages: u64| (pages * vm_stats.page_size) as f64 / (1024.0 * 1024.0 * 1024.0);

        let total_gb = vm_stats.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
        let app_memory_gb = page_to_gb(vm_stats.pages_anonymous);
        let wired_gb = page_to_gb(vm_stats.pages_wired);
        let compressed_gb = page_to_gb(vm_stats.pages_compressed);
        let cached_gb = page_to_gb(vm_stats.pages_file_backed);
        let free_gb = page_to_gb(vm_stats.pages_free);
        let active_gb = page_to_gb(vm_stats.pages_active);
        let inactive_gb = page_to_gb(vm_stats.pages_inactive);

        let activity_monitor_used =
            vm_stats.activity_monitor_memory_used() as f64 / (1024.0 * 1024.0 * 1024.0);

        let content = vec![
            Line::from(vec![
                Span::styled(
                    "Physical Memory Total: ",
                    Style::default().fg(app.colors.accent()),
                ),
                Span::raw(format!("{:.2} GB", total_gb)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "═══ ACTIVITY MONITOR CALCULATION ═══",
                Style::default().fg(app.colors.accent()),
            )]),
            Line::from(vec![
                Span::styled(
                    "App Memory (Anonymous): ",
                    Style::default().fg(app.colors.gauge_fg()),
                ),
                Span::raw(format!("{:.2} GB", app_memory_gb)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Wired Memory:         + ",
                    Style::default().fg(app.colors.gauge_fg()),
                ),
                Span::raw(format!("{:.2} GB", wired_gb)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Compressed:           + ",
                    Style::default().fg(app.colors.gauge_fg()),
                ),
                Span::raw(format!("{:.2} GB", compressed_gb)),
            ]),
            Line::from(vec![Span::styled(
                "                      ─────────",
                Style::default().fg(app.colors.history_fg()),
            )]),
            Line::from(vec![
                Span::styled(
                    "Memory Used Total:      ",
                    Style::default().fg(app.colors.accent()),
                ),
                Span::raw(format!("{:.2} GB", activity_monitor_used)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "═══ OTHER MEMORY CATEGORIES ═══",
                Style::default().fg(app.colors.history_fg()),
            )]),
            Line::from(vec![
                Span::styled(
                    "Cached Files:         ",
                    Style::default().fg(app.colors.gauge_fg()),
                ),
                Span::raw(format!("{:.2} GB", cached_gb)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Free:                 ",
                    Style::default().fg(app.colors.gauge_fg()),
                ),
                Span::raw(format!("{:.2} GB", free_gb)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Active:               ",
                    Style::default().fg(app.colors.gauge_fg()),
                ),
                Span::raw(format!("{:.2} GB", active_gb)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Inactive:             ",
                    Style::default().fg(app.colors.gauge_fg()),
                ),
                Span::raw(format!("{:.2} GB", inactive_gb)),
            ]),
        ];

        let paragraph = Paragraph::new(content).block(block);
        f.render_widget(paragraph, area);
    } else {
        let error_content = vec![
            Line::from("Failed to collect VM statistics"),
            Line::from("vm_stat command may not be available"),
        ];
        let paragraph = Paragraph::new(error_content).block(block);
        f.render_widget(paragraph, area);
    }
}

/// Draw the sysinfo section.
fn draw_sysinfo_section(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Sysinfo Statistics ")
        .borders(Borders::ALL);

    if let Some(metrics) = &app.metrics {
        let mem = &metrics.memory;

        let content = vec![
            Line::from(vec![
                Span::styled("RAM Used: ", Style::default().fg(app.colors.accent())),
                Span::raw(format!(
                    "{} = {} / {} ({:.1}%)",
                    units::percent1(mem.ram_usage_ratio() * 100.0),
                    units::bibytes1(mem.ram_used as f64),
                    units::bibytes1(mem.ram_total as f64),
                    mem.ram_usage_ratio() * 100.0
                )),
            ]),
            Line::from(vec![
                Span::styled("Swap Used: ", Style::default().fg(app.colors.accent())),
                Span::raw(format!(
                    "{} = {} / {}",
                    units::percent1(mem.swap_usage_ratio() * 100.0),
                    units::bibytes1(mem.swap_used as f64),
                    units::bibytes1(mem.swap_total as f64)
                )),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Note: ", Style::default().fg(app.colors.history_fg())),
                Span::raw("RAM Used now uses vm_stat for Activity Monitor compatibility"),
            ]),
        ];

        let paragraph = Paragraph::new(content).block(block);
        f.render_widget(paragraph, area);
    } else {
        let error_content = vec![Line::from("No metrics available")];
        let paragraph = Paragraph::new(error_content).block(block);
        f.render_widget(paragraph, area);
    }
}
