//! SoC tab.
//!
//! A borderless 2-column table (label width 20, value width 16, the original default
//! `column_spacing = 1`), right column bold. The rows are session-static —
//! built once by [`render_soc_rows`](crate::backend::frame::render_soc_rows) and
//! threaded through as a one-time prop, not per-frame.

use iocraft::prelude::*;

use crate::{
    backend::frame::SocRows,
    ui::{theme::Theme, views::two_col_row},
};

/// SoC table label column width (20 columns).
const SOC_LABEL_WIDTH: usize = 20;

/// Render the SoC tab at `width`.
pub(crate) fn soc(rows: &SocRows, width: usize, _theme: Theme) -> AnyElement<'static> {
    #[expect(clippy::cast_possible_truncation)]
    let w = width as u32;
    let body: Vec<AnyElement<'static>> = rows
        .rows
        .iter()
        .map(|(l, r)| two_col_row(l, r, SOC_LABEL_WIDTH))
        .collect();
    element! {
        View(flex_direction: FlexDirection::Column, width: w) {
            #(body)
        }
    }
    .into_any()
}
