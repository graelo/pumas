//! Tab bar.
//!
//! A `View` bordered on all edges (3 rows) whose single inner row holds the
//! five tab labels. Each tab is
//! `padding_left` +
//! title + `padding_right` (both a single space), tabs separated by the
//! `│` (U+2502) divider. The active title is accent + bold;
//! the surrounding padding/divider stay default. The net inner string is
//! `" Overview │ CPU │ GPU │ Memory │ SoC "`.

use iocraft::prelude::*;

/// The five tab titles, in order.
pub(crate) const TAB_TITLES: [&str; 5] = ["Overview", "CPU", "GPU", "Memory", "SoC"];

/// Render one tab label, accent + bold when it is the active tab.
fn tab_label(title: &'static str, active: bool, accent: Color) -> AnyElement<'static> {
    if active {
        element! {
            Text(content: title, color: accent, weight: Weight::Bold, wrap: TextWrap::NoWrap)
        }
        .into_any()
    } else {
        element! { Text(content: title, wrap: TextWrap::NoWrap) }.into_any()
    }
}

/// Render the bordered tab bar at an explicit `width`, highlighting `active`.
pub(crate) fn tab_bar(active: usize, accent: Color, width: usize) -> AnyElement<'static> {
    #[expect(clippy::cast_possible_truncation)]
    let w = width as u32;

    // Interleave: leading pad, then `title (│ )` per tab, trailing pad.
    let last = TAB_TITLES.len() - 1;
    let mut segments: Vec<AnyElement<'static>> = Vec::new();
    segments.push(element! { Text(content: " ", wrap: TextWrap::NoWrap) }.into_any());
    for (i, title) in TAB_TITLES.iter().enumerate() {
        segments.push(tab_label(title, i == active, accent));
        let sep = if i == last { " " } else { " │ " };
        segments.push(element! { Text(content: sep, wrap: TextWrap::NoWrap) }.into_any());
    }

    element! {
        View(
            width: w,
            border_style: BorderStyle::Single,
            border_edges: Edges::all(),
        ) {
            View(flex_direction: FlexDirection::Row) {
                #(segments)
            }
        }
    }
    .into_any()
}
