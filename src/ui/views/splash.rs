//! Startup splash.
//!
//! Shown while the backend has not yet shipped a `Frame`. The 40×17 logo is
//! built from three colored regions — top-left (blue, 15w), top-right (green,
//! 25w), bottom (magenta) — plus the default-colored `pumas` ASCII wordmark,
//! then a spacer and a centered "Starting up..." message. The whole thing is
//! centered horizontally and vertically.
//!
//! Colors are pinned to the ANSI indices the original UI used
//! (blue = 4, green = 2, magenta = 5) via [`Color::AnsiValue`]. Do NOT use the
//! crossterm-named `Color::Blue/Green/Magenta`: those map to the *bright* indices
//! 12/10/13, which themes like Solarized repurpose as greys/violet.
//!
//! The ASCII literals are copied verbatim from the original logo art.
//! Colors are not captured by the plain-text snapshot; they
//! are verified by the live smoke check.

use iocraft::prelude::*;

const LOGO2_TOP_LEFT_WIDTH: u32 = 15;
const LOGO2_TOP_RIGHT_WIDTH: u32 = 25;

/// Top-left logo, height: 9 lines (rendered blue).
const LOGO2_TOP_LEFT: &str = "   ▓▓     ▓▓
   ██     ██
   ██     ██
▄▄▄▓▓▄▄▄▄▄██▄▄▄
▓▓▓▓▓▓▓▓███████
▓▓▓▓▓▓▓▓███████
▐▓▓▓▓▓▓▓██████▌
 ▀▓▓▓▓▓▓█████▀
   ▀▀▓▓▓██▀▀";

/// Top-right logo, height: 9 lines (rendered green).
const LOGO2_TOP_RIGHT: &str = "             ░░░░░░░░░░░▒
         ░░░░░░░░░░░░░▒▒▒
       ░░░░░░░░░░░░░▒▒▒▒▒
      ░░░░░░░░░░░▒▒▒▒▒▒▒▒
     ░░░░░░░░▒▓▒▒▒▒▒▒▒▒▒▒
     ░░░░░░▒▓▒▒▒▒▒▒▒▒▒▒▒▌
     ░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
     ░▒▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒
     ▓▒▒▒▒▒▒▒▒▒▒▒▒▀";

/// Bottom logo, height: 8 lines (rendered magenta).
const LOGO2_BOTTOM: &str = "      ▐▒▌          ▒▒
      ▐▒▌         ▐▒▌
      ▐▒▌          ▐▒▓▄
      ▐▒▌            ▀▀▒▓▓▓▓▓▓▓▓▓▓▓▓▒▄
      ▐▒▌                           ▀▓▓▄
       ▒▓                             ▒▒
       ▐▒▓▄                          ▓▒▌
         ▀▀▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▀";

/// The `pumas` ASCII wordmark (rendered in the default color).
const PUMAS: &str = r"
    _ __  _   _ _ __ ___   __ _ ___
   | '_ \| | | | '_ ` _ \ / _` / __|
   | |_) | |_| | | | | | | (_| \__ \
   | .__/ \__,_|_| |_| |_|\__,_|___/
   |_|";

/// An empty row used as a vertical spacer.
fn blank(height: u32) -> AnyElement<'static> {
    element! { View(height: height) }.into_any()
}

/// Render the splash, centered within `width` × `height`.
pub(crate) fn splash(width: usize, height: usize) -> AnyElement<'static> {
    #[expect(clippy::cast_possible_truncation)]
    let (w, h) = (width as u32, height as u32);
    element! {
        View(
            width: w,
            height: h,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        ) {
            // Logo: top (blue | green), bottom (magenta), blank, wordmark.
            View(flex_direction: FlexDirection::Column, align_items: AlignItems::Center) {
                View(flex_direction: FlexDirection::Row) {
                    View(width: LOGO2_TOP_LEFT_WIDTH) {
                        Text(content: LOGO2_TOP_LEFT, color: Color::AnsiValue(4), wrap: TextWrap::NoWrap)
                    }
                    View(width: LOGO2_TOP_RIGHT_WIDTH) {
                        Text(content: LOGO2_TOP_RIGHT, color: Color::AnsiValue(2), wrap: TextWrap::NoWrap)
                    }
                }
                Text(content: LOGO2_BOTTOM, color: Color::AnsiValue(5), wrap: TextWrap::NoWrap)
                #(blank(1))
                Text(content: PUMAS, wrap: TextWrap::NoWrap)
            }
            #(blank(2))
            Text(content: "Starting up...", wrap: TextWrap::NoWrap)
        }
    }
    .into_any()
}
