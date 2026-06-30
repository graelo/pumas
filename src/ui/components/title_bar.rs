//! Top title bar (MIGRATION.md §9.1, the original `main_screen.rs:31-50`).
//!
//! A single row: the program name (`Pumas v{version}`) on the left in the
//! default color, and the machine description (`{brand} (cores: …)`) on the
//! right in the accent color. The original overlays a left-aligned and a
//! right-aligned paragraph on the same row; iocraft expresses this as one
//! `Row` with `justify_content: SpaceBetween`.

use iocraft::prelude::*;

/// Render the title bar at an explicit `width`.
pub(crate) fn title_bar(
    program_name: String,
    machine_desc: String,
    accent: Color,
    width: usize,
) -> AnyElement<'static> {
    #[expect(clippy::cast_possible_truncation)]
    let w = width as u32;
    element! {
        View(
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            width: w,
        ) {
            Text(content: program_name, wrap: TextWrap::NoWrap)
            Text(content: machine_desc, color: accent, wrap: TextWrap::NoWrap)
        }
    }
    .into_any()
}
