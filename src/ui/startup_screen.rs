//! Startup screen.

use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text,
    widgets::{Paragraph, Wrap},
    Frame,
};

/// Draw the startup screen.
pub(crate) fn draw<B: Backend>(f: &mut Frame<B>) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(17 + 2 + 6 + 2), Constraint::Length(1)])
        // .constraints([Constraint::Min(17), Constraint::Length(1)].as_ref())
        .split(f.size());
    let logo_area = vertical_chunks[0];
    let message_area = vertical_chunks[1];

    draw_logo(f, logo_area);

    let message = text::Text::from("Starting up...".to_string());

    let par = Paragraph::new(message)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(par, message_area);
}

/// Draw the logo.
fn draw_logo<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((f.size().width - 37) / 2), // to center
            Constraint::Length(37),
            // Constraint::Min(0),
        ])
        .split(area);
    let logo_area = horizontal_chunks[1];

    let logo_vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Length(6),
        ])
        .split(logo_area);
    let logo_top_area = logo_vertical_chunks[0];
    let logo_bot_area = logo_vertical_chunks[1];
    let pumas_text_area = logo_vertical_chunks[3];

    let logo_horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(14), Constraint::Length(23)])
        .split(logo_top_area);

    let logo_top_left =
        Paragraph::new(text::Text::from(LOGO_17_TOP_LEFT)).style(Style::default().fg(Color::Blue));
    let logo_top_right = Paragraph::new(text::Text::from(LOGO_17_TOP_RIGHT))
        .style(Style::default().fg(Color::Green));
    let logo_bottom =
        Paragraph::new(text::Text::from(LOGO_17_BOTTOM)).style(Style::default().fg(Color::Magenta));
    let pumas_text = Paragraph::new(text::Text::from(PUMAS));

    f.render_widget(logo_top_left, logo_horizontal_chunks[0]);
    f.render_widget(logo_top_right, logo_horizontal_chunks[1]);
    f.render_widget(logo_bottom, logo_bot_area);
    f.render_widget(pumas_text, pumas_text_area);
}

/// Top-left logo, height: 10 lines.
const LOGO_17_TOP_LEFT: &str = "
  .!~    :?^
  ^G5.   ?&J
  :G5    7&J
.~7P5!~!!YB5!^
~P55555GGGGGG5
~555555PGGGGGY
:Y55555PGGGGG7
 :?5555PGGG5!
   :~!J5?!^.";

/// Top-right logo, height: 10 lines.
const LOGO_17_TOP_RIGHT: &str = "
          ............
       ....:::::..::~~.
     ..::.......:^~~~~.
     .:......::^~~~~~~.
    .:....:^77~~~~~~~~.
    .:..:~7?7!~~~~~~~^
    ..:~7?7!~~~~~~~~^
    :!??7!~~~~~~~~^.
   ^?7~^^^^^^^::..";

/// Bottom logo, height: 8 lines.
const LOGO_17_BOTTOM: &str = "      7?:       :?7.
      7?:       :?7.
      7?:        ~?7:.
      7?:         :~777!!!!!!!!!!~.
      7?:            .::::::::::^7?~
      ~J~                        .??.
       ~?7~:::::::::::::::::::::^!?~
        .^!!!!!!!!!!!!!!!!!!!!!!!~:";

// const LOGO_17_HEIGHT: &str = "
//   .!~    :?^            ............
//   ^G5.   ?&J         ....:::::..::~~.
//   :G5    7&J       ..::.......:^~~~~.
// .~7P5!~!!YB5!^     .:......::^~~~~~~.
// ~P55555GGGGGG5    .:....:^77~~~~~~~~.
// ~555555PGGGGGY    .:..:~7?7!~~~~~~~^
// :Y55555PGGGGG7    ..:~7?7!~~~~~~~~^
//  :?5555PGGG5!     :!??7!~~~~~~~~^.
//    :~!J5?!^.     ^?7~^^^^^^^::..
//       7?:       :?7.
//       7?:       :?7.
//       7?:        ~?7:.
//       7?:         :~777!!!!!!!!!!~.
//       7?:            .::::::::::^7?~
//       ~J~                        .??.
//        ~?7~:::::::::::::::::::::^!?~
//         .^!!!!!!!!!!!!!!!!!!!!!!!~:
// ";

const PUMAS: &str = r#"
 _ __  _   _ _ __ ___   __ _ ___
| '_ \| | | | '_ ` _ \ / _` / __|
| |_) | |_| | | | | | | (_| \__ \
| .__/ \__,_|_| |_| |_|\__,_|___/
|_|"#;
