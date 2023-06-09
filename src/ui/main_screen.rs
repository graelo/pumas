//! Definition of the UI.

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::App;

use super::{tab_cpu, tab_gpu, tab_overview, tab_soc};

/// Draw the main UI.
pub(crate) fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(area);
    let title_area = chunks[0];
    let tabs_area = chunks[1];
    let main_area = chunks[2];

    //
    // Title line.
    //
    let program_name = format!("Pumas v{}", env!("CARGO_PKG_VERSION"));
    let app_paragraph = Paragraph::new(Span::from(program_name));
    f.render_widget(app_paragraph, title_area);

    let machine_desc = format!(
        " {} (cores: {}E+{}P+{}GPU) ",
        app.soc.cpu_brand_name,
        app.soc.num_efficiency_cores,
        app.soc.num_performance_cores,
        app.soc.num_gpu_cores
    );
    let machine_desc_par = Paragraph::new(Span::styled(
        machine_desc,
        Style::default().fg(Color::Green),
    ))
    .alignment(Alignment::Right);
    f.render_widget(machine_desc_par, title_area);

    //
    // Tab bar.
    //
    let tab_titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(*t, Style::default().fg(Color::LightYellow))))
        .collect();
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        // .title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .select(app.tabs.index);

    f.render_widget(tabs, tabs_area);

    //
    // Content.
    //
    match app.tabs.index {
        0 => tab_overview::draw_overview_tab(f, app, main_area),
        1 => tab_cpu::draw_cpu_tab(f, app, main_area),
        2 => tab_gpu::draw_gpu_tab(f, app, main_area),
        3 => tab_soc::draw_soc_tab(f, app, main_area),
        _ => {}
    };
}
