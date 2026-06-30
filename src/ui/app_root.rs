//! Root iocraft component: the one-directional data plane's frontend half.
//!
//! `PumasApp` owns the UI-side state (current [`Frame`], selected tab, exit
//! flag), drains the backend channel in a single `use_future`, and handles the
//! keyboard. While no frame has arrived it shows the splash; once frames flow it
//! renders the title bar, the tab bar, and the selected tab's view
//! (MIGRATION.md §7.4–§7.7, §8). All five tabs (Overview, CPU, GPU, Memory,
//! SoC) are wired to their real views.

use iocraft::prelude::*;
use smol::channel::Receiver;

use crate::{
    backend::frame::{Frame, RenderedHeader, SocRows},
    ui::{
        components::{tab_bar::tab_bar, title_bar::title_bar},
        theme::Theme,
        views::{cpu::cpu, gpu::gpu, memory::memory, overview::overview, soc::soc, splash::splash},
    },
};

/// Number of tabs (Overview, CPU, GPU, Memory, SoC).
const NUM_TABS: usize = 5;

#[derive(Default, Props)]
pub(crate) struct PumasAppProps {
    /// Backend frame stream. Taken once into the draining `use_future`.
    pub rx: Option<Receiver<Frame>>,
    /// Session-static title-bar strings.
    pub header: Option<RenderedHeader>,
    /// Session-static SoC-tab rows (built once, taken into state on first render).
    pub soc_rows: Option<SocRows>,
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

    // SoC rows are session-static too: take once into state.
    let soc_rows = props.soc_rows.take();
    let soc_rows = hooks.use_state(move || soc_rows.unwrap_or_default());

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
    let w = usize::from(width);

    // Splash full-screen until the first frame arrives (mirrors ratatui's
    // startup screen, which replaces the whole UI — no title/tab bar).
    let Some(frame) = frame_state.read().clone() else {
        return element! {
            View(width: u32::from(width), height: u32::from(height)) {
                #(vec![splash(w, usize::from(height))])
            }
        }
        .into_any();
    };

    let active = tab.get();
    let body: AnyElement<'static> = match active {
        0 => overview(&frame.overview, w, theme),
        1 => cpu(&frame.cpu, w, theme),
        2 => gpu(&frame.gpu, w, theme),
        3 => memory(&frame.memory, w, theme),
        4 => soc(&soc_rows.read(), w, theme),
        _ => overview(&frame.overview, w, theme),
    };

    let chrome = vec![
        title_bar(program_name, machine_desc, theme.accent, w),
        tab_bar(active, theme.accent, w),
        body,
    ];

    element! {
        View(
            flex_direction: FlexDirection::Column,
            width: u32::from(width),
            height: u32::from(height),
        ) {
            #(chrome)
        }
    }
    .into_any()
}
