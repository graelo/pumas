//! CPU cores tab.

use ratatui::{
    backend::Backend,
    layout::Rect,
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// Draw the per-core usage, and per-core frequency distribution.
pub(crate) fn draw_cpu_tab<B>(f: &mut Frame<B>, _app: &App, area: Rect)
where
    B: Backend,
{
    let text = Text::from("Coming soon: CPU Power and frequency distribution.");
    let par = Paragraph::new(text).block(Block::default().title("Paragraph").borders(Borders::ALL));
    f.render_widget(par, area);

    // let chunks = Layout::default()
    //     .constraints(
    //         [
    //             Constraint::Length(2),
    //             Constraint::Length(3),
    //             Constraint::Length(1),
    //         ]
    //         .as_ref(),
    //     )
    //     .margin(1)
    //     .split(area);

    // let block = Block::default().title(app.title).borders(Borders::ALL);
    // f.render_widget(block, area);

    // if let Some(metrics) = &app.metrics {
    //     let text_cpu_power =
    //         Text::from(format!("Per-core Power: {}", units::watts2(metrics.cpu_w)));
    //     let par = Paragraph::new(text_cpu_power)
    //         .block(Block::default().title("Paragraph").borders(Borders::ALL))
    //         // .style(Style::default().fg(Color::White).bg(Color::Black))
    //         // .alignment(Alignment::Center)
    //         .wrap(Wrap { trim: true });
    //     f.render_widget(par, chunks[1]);
    // }

    // let label = format!("{:.2}%", app.progress * 100.0);
    // let gauge = Gauge::default()
    //     .block(Block::default().title("Gauge:"))
    //     .gauge_style(
    //         Style::default()
    //             .fg(Color::Magenta)
    //             .bg(Color::LightCyan)
    //             .add_modifier(Modifier::ITALIC | Modifier::BOLD),
    //     )
    //     .label(label)
    //     .ratio(app.progress);
    // f.render_widget(gauge, chunks[0]);
}
