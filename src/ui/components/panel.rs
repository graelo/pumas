//! Bordered panel with a ratatui-style inline border title (`┌ Title ──┐`).
//!
//! iocraft `View` has native border props but no `Block::title` equivalent, so
//! the title is composed manually (MIGRATION.md §7.2): the panel is a `Column`
//! whose first row is the top-border line (`┌` + title + `─`-fill + `┐`) and
//! whose second child is a `View` with native borders on every edge *except*
//! Top. The left/right `│` and the bottom `└──┘` are drawn natively; only the
//! titled top line is hand-composed. This is NOT a border drawer (DO-NOT #1) —
//! all box-drawing except the title line comes from `View`'s own border props.
//!
//! `width` is the total outer width including the two corner columns; the body
//! must be sized to `width - 2` by the caller.

use iocraft::prelude::*;

/// Compose the titled top-border line, exactly `width` columns wide:
/// `┌` + title (truncated to fit) + `─` filler + `┐`.
fn top_border_line(title: &str, width: usize) -> String {
    let inner = width.saturating_sub(2); // columns between the two corners
    let title_w = title.chars().count().min(inner);
    let mut s = String::with_capacity(width * 3);
    s.push('┌');
    s.extend(title.chars().take(title_w));
    for _ in 0..(inner - title_w) {
        s.push('─');
    }
    s.push('┐');
    s
}

/// Render a bordered panel with an inline title and the given body.
///
/// The body is any owned `'static` element (typically a `Column` of gauges and
/// sparklines). `border_color` colors the box-drawing (and, for parity with
/// ratatui's default styling, the title text — both are `Color::Reset` today).
pub(crate) fn panel(
    title: &str,
    width: usize,
    border_color: Color,
    body: AnyElement<'static>,
) -> AnyElement<'static> {
    let top = top_border_line(title, width);
    #[expect(clippy::cast_possible_truncation)]
    let w = width as u32;
    element! {
        View(flex_direction: FlexDirection::Column, width: w) {
            Text(content: top, color: border_color, wrap: TextWrap::NoWrap)
            View(
                flex_direction: FlexDirection::Column,
                width: w,
                border_style: BorderStyle::Single,
                border_edges: Edges::Left | Edges::Right | Edges::Bottom,
                border_color: border_color,
            ) {
                #(vec![body])
            }
        }
    }
    .into_any()
}
