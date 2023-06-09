//! SoC tab.

use ratatui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Cell, Row, Table},
    Frame,
};

use crate::{app::App, units};

/// Draw the SoC tab.
///
/// A simple table with the SoC's name, number of cores, etc.
pub(crate) fn draw_soc_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let row_content = vec![
        ("SoC brand name:", app.soc.cpu_brand_name.clone()),
        ("CPU cores:", format!("{}", app.soc.num_cpu_cores)),
        (
            "- Efficiency cores:",
            format!("{}", app.soc.num_efficiency_cores),
        ),
        (
            "- Performance cores:",
            format!("{}", app.soc.num_performance_cores),
        ),
        ("GPU cores:", format!("{}", app.soc.num_gpu_cores)),
        ("Max CPU power:", units::watts(app.soc.max_cpu_w)),
        ("Max GPU power:", units::watts(app.soc.max_gpu_w)),
        ("Max ANE power:", units::watts(app.soc.max_ane_w)),
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

    let table = Table::new(rows).widths(&[Constraint::Length(20), Constraint::Length(16)]);

    f.render_widget(table, area);
}
