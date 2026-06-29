//! Root iocraft component: the one-directional data plane's frontend half.
//!
//! `PumasApp` owns the UI-side state (current [`Frame`], selected tab, exit
//! flag), drains the backend channel in a single `use_future`, and handles the
//! keyboard. Phase 1 renders only the title bar + a raw debug dump of the
//! incoming frame to prove the pipe; the per-tab views land in Phase 2
//! (MIGRATION.md §7.4–§7.7, §8).

use iocraft::prelude::*;
use smol::channel::Receiver;

use crate::{
    backend::frame::{Frame, RenderedHeader},
    ui::theme::Theme,
};

/// Number of tabs (Overview, CPU, GPU, Memory, SoC).
const NUM_TABS: usize = 5;

#[derive(Default, Props)]
pub(crate) struct PumasAppProps {
    /// Backend frame stream. Taken once into the draining `use_future`.
    pub rx: Option<Receiver<Frame>>,
    /// Session-static title-bar strings.
    pub header: Option<RenderedHeader>,
    /// Resolved theme colors.
    pub theme: Theme,
}

#[component]
pub(crate) fn PumasApp(
    mut hooks: Hooks,
    props: &mut PumasAppProps,
) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let theme = props.theme;

    let mut frame_state = hooks.use_state(|| Option::<Frame>::None);
    let mut tab = hooks.use_state(|| 0usize);
    let mut should_exit = hooks.use_state(|| false);
    let mut system = hooks.use_context_mut::<SystemContext>();

    // Header is session-static: take it once into state on first render.
    let header = props.header.take();
    let header = hooks.use_state(move || header.unwrap_or_default());

    // Drain the backend channel. `use_future` spawns exactly once, so taking
    // the receiver out of props here is safe (it is `Some` only on first
    // render). When the collector drops its sender (error or shutdown),
    // `recv()` errors, we fall through and request exit.
    let rx = props.rx.take();
    hooks.use_future(async move {
        if let Some(rx) = rx {
            while let Ok(frame) = rx.recv().await {
                frame_state.set(Some(frame));
            }
        }
        should_exit.set(true);
    });

    // Keyboard: quit + tab navigation (MIGRATION.md §7.5).
    hooks.use_terminal_events(move |event| {
        if let TerminalEvent::Key(KeyEvent {
            code,
            kind,
            modifiers,
            ..
        }) = event
        {
            if kind == KeyEventKind::Release {
                return;
            }
            match code {
                KeyCode::Char('q') | KeyCode::Char('x') | KeyCode::Esc => should_exit.set(true),
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    should_exit.set(true);
                }
                KeyCode::Left | KeyCode::BackTab => {
                    let i = tab.get();
                    tab.set(if i == 0 { NUM_TABS - 1 } else { i - 1 });
                }
                KeyCode::Right | KeyCode::Tab => {
                    tab.set((tab.get() + 1) % NUM_TABS);
                }
                _ => {}
            }
        }
    });

    // Exit at render time, never inside the event closure (MIGRATION.md §7.6).
    if should_exit.get() {
        system.exit();
    }

    let header = header.read();
    let program_name = header.program_name.clone();
    let machine_desc = header.machine_desc.clone();

    // Body: splash while no frame, else a raw debug dump proving the pipe.
    let body: AnyElement<'static> = match frame_state.read().as_ref() {
        None => element! { Text(content: "Starting up…", wrap: TextWrap::NoWrap) }.into_any(),
        Some(f) => {
            let first_e = f
                .overview
                .e_meters
                .first()
                .map_or_else(String::new, |m| m.title.clone());
            let dump = format!(
                "[tab {}] {}\n{}\n{}",
                tab.get(),
                f.overview.cpu_clusters_title,
                first_e,
                f.overview.package.title,
            );
            element! { Text(content: dump, wrap: TextWrap::NoWrap) }.into_any()
        }
    };

    element! {
        View(
            flex_direction: FlexDirection::Column,
            width: u32::from(width),
            height: u32::from(height),
        ) {
            View(
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
            ) {
                Text(content: program_name, wrap: TextWrap::NoWrap)
                Text(content: machine_desc, color: theme.accent, wrap: TextWrap::NoWrap)
            }
            #(vec![body])
        }
    }
}
