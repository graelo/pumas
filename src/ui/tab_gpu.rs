//! GPU tab.

use ratatui::{
    backend::Backend,
    layout::Rect,
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// Draw the GPU tab.
pub(crate) fn draw_gpu_tab<B>(f: &mut Frame<B>, _app: &App, area: Rect)
where
    B: Backend,
{
    let text = Text::from("Coming soon: GPU Power and frequency distribution.");
    let par = Paragraph::new(text).block(Block::default().title("Paragraph").borders(Borders::ALL));
    f.render_widget(par, area);
}
