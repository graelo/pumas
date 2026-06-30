//! Memory tab (MIGRATION.md §9.6, the original `tab_memory.rs`).
//!
//! Two bordered text blocks inside a `margin(1)` inset: the Activity-Monitor-
//! compatible `VM Statistics` block (height 18) and the `Sysinfo Statistics`
//! block (height 8). Every line arrives pre-formatted and pre-colored as a
//! [`MemLine`] of [`MemSpan`]s; the view only maps each span's [`ColorRole`] to
//! a concrete theme color and pads each block to its fixed inner height.
//!
//! [`MemSpan`]: crate::backend::frame::MemSpan

use iocraft::prelude::*;

use crate::{
    backend::frame::{ColorRole, MemLine, MemoryFrame},
    ui::{components::panel::panel, theme::Theme},
};

/// Inner rows of the `VM Statistics` block (`18 - 2` borders).
const VM_INNER_ROWS: usize = 16;

/// Inner rows of the `Sysinfo Statistics` block (`8 - 2` borders).
const SYSINFO_INNER_ROWS: usize = 6;

/// Map a [`ColorRole`] to its concrete theme color (MIGRATION.md §6).
fn role_color(role: ColorRole, theme: Theme) -> Color {
    match role {
        ColorRole::Accent => theme.accent,
        ColorRole::GaugeFg => theme.gauge_fg,
        ColorRole::HistoryFg => theme.history_fg,
        ColorRole::Default => Color::Reset,
    }
}

/// Render one memory line: a blank line becomes a 1-row spacer; otherwise a
/// `MixedText` of the colored spans.
fn mem_line(line: &MemLine, theme: Theme) -> AnyElement<'static> {
    if line.spans.is_empty() {
        return element! { View(height: 1u32) }.into_any();
    }
    let contents: Vec<MixedTextContent> = line
        .spans
        .iter()
        .map(|s| MixedTextContent::new(s.text.clone()).color(role_color(s.role, theme)))
        .collect();
    element! { MixedText(wrap: TextWrap::NoWrap, contents: contents) }.into_any()
}

/// Build one bordered, height-fixed text block.
fn mem_block(
    title: &str,
    width: usize,
    lines: &[MemLine],
    inner_rows: usize,
    theme: Theme,
) -> AnyElement<'static> {
    let mut rows: Vec<AnyElement<'static>> = lines.iter().map(|l| mem_line(l, theme)).collect();
    while rows.len() < inner_rows {
        rows.push(element! { View(height: 1u32) }.into_any());
    }
    let body = element! {
        View(flex_direction: FlexDirection::Column) { #(rows) }
    }
    .into_any();
    panel(title, width, Color::Reset, body)
}

/// Render the full Memory tab at `width`.
pub(crate) fn memory(f: &MemoryFrame, width: usize, theme: Theme) -> AnyElement<'static> {
    // the original applies `margin(1)`: a 1-col left/right inset and a 1-row top
    // inset; the blocks therefore render at `width - 2`.
    let inner = width.saturating_sub(2);
    let vm = mem_block(
        " VM Statistics (Activity Monitor compatible) ",
        inner,
        &f.vm_lines,
        VM_INNER_ROWS,
        theme,
    );
    let sysinfo = mem_block(
        " Sysinfo Statistics ",
        inner,
        &f.sysinfo_lines,
        SYSINFO_INNER_ROWS,
        theme,
    );
    element! {
        View(flex_direction: FlexDirection::Column, padding_top: 1, padding_left: 1, padding_right: 1) {
            #(vec![vm, sysinfo])
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::frame::MemSpan;

    /// Color parity (D6): an `Accent`-role span renders with the theme accent
    /// foreground. Plain-text snapshots cannot see color, so we inspect the
    /// `Canvas` cell directly (MIGRATION.md §7.8).
    #[test]
    fn mem_span_role_maps_to_theme_color() {
        let theme = Theme::default();
        let line = MemLine {
            spans: vec![MemSpan {
                text: "RAM".to_string(),
                role: ColorRole::Accent,
            }],
        };
        let mut el = mem_line(&line, theme);
        let canvas = el.render(Some(10));
        let cell = canvas.cell(0, 0).expect("mem span cell (0,0)");
        assert_eq!(cell.text(), Some("R"), "first glyph of the accent span");
        assert_eq!(
            cell.text_style().and_then(|s| s.color),
            Some(theme.accent),
            "Accent-role span foreground must be the theme accent color",
        );
    }
}
