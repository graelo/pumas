//! Reusable iocraft leaf components for the Pumas UI.
//!
//! Each component follows the gh-board owned-prop idiom: a single
//! `Option<Rendered*>` prop carrying fully-owned data (no borrows), taken with
//! `props.take()`, returning `impl Into<AnyElement<'static>>`. None of these
//! components compute history or formatting — they only turn already-prepared
//! values into glyphs (see `MIGRATION.md` §5/§7).

pub(crate) mod gauge;
pub(crate) mod line_gauge;
pub(crate) mod panel;
pub(crate) mod sparkline;

use iocraft::prelude::*;

/// A single rendered terminal cell: one character plus its fg/bg colors.
///
/// Components build a `Vec<Vec<Cell>>` (rows of cells) and hand it to
/// [`render_grid`], which merges same-colored runs into compact `Text` spans.
#[derive(Clone, Debug)]
pub(crate) struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
}

impl Cell {
    pub(crate) fn new(ch: char, fg: Color, bg: Color) -> Self {
        Self { ch, fg, bg }
    }
}

/// A run of consecutive cells sharing the same fg/bg, collapsed to one string.
struct Segment {
    text: String,
    fg: Color,
    bg: Color,
}

/// Collapse a row of cells into the minimal set of same-colored runs.
fn merge_runs(cells: &[Cell]) -> Vec<Segment> {
    let mut segments: Vec<Segment> = Vec::new();
    for cell in cells {
        match segments.last_mut() {
            Some(seg) if seg.fg == cell.fg && seg.bg == cell.bg => seg.text.push(cell.ch),
            _ => segments.push(Segment {
                text: cell.ch.to_string(),
                fg: cell.fg,
                bg: cell.bg,
            }),
        }
    }
    segments
}

/// Render a rectangular grid of cells as a `Column` of `Row`s, merging
/// same-colored runs within each row into single `Text` spans. The background
/// color of each run is painted via the wrapping `View` (iocraft `Text` has no
/// background prop).
pub(crate) fn render_grid(rows: Vec<Vec<Cell>>) -> AnyElement<'static> {
    let seg_rows: Vec<Vec<Segment>> = rows.iter().map(|r| merge_runs(r)).collect();
    element! {
        View(flex_direction: FlexDirection::Column) {
            #(seg_rows.into_iter().enumerate().map(|(ri, segs)| element! {
                View(key: ri, flex_direction: FlexDirection::Row) {
                    #(segs.into_iter().enumerate().map(|(si, seg)| element! {
                        View(key: si, background_color: seg.bg) {
                            Text(content: seg.text, color: seg.fg, wrap: TextWrap::NoWrap)
                        }
                    }))
                }
            }))
        }
    }
    .into_any()
}
