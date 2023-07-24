//! Hold the application state.

use std::collections::HashMap;

use crate::{
    modules::{powermetrics::Metrics, soc::SocInfo},
    signal,
};

const HISTORY_CAPACITY: usize = 100;

pub(crate) type History = HashMap<String, signal::Signal<f32>>;

pub(crate) struct TabsState<'a> {
    pub(crate) titles: Vec<&'a str>,
    pub(crate) index: usize,
}

impl<'a> TabsState<'a> {
    fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

/// The App structure.
pub(crate) struct App<'a> {
    /// Indicates the app should quit.
    pub(crate) should_quit: bool,

    /// Tabs and currently selected tab.
    pub(crate) tabs: TabsState<'a>,

    /// Accent color, default: 2 (green).
    accent_color_: u8,

    /// Gauge background color, default: 7 (white).
    gauge_bg_color_: u8,

    /// Time of last update.
    pub(crate) last_update: std::time::Instant,

    /// Power and usage metrics.
    pub(crate) metrics: Option<Metrics>,

    /// System-on-chip information.
    pub(crate) soc_info: SocInfo,

    /// Store the history of all signals (u64 needed for `Sparkline`).
    pub(crate) history: History,
}

impl<'a> App<'a> {
    /// Returns a new `App`.
    pub fn new(soc_info: SocInfo, accent_color: u8, gauge_bg_color: u8) -> Self {
        Self {
            should_quit: false,
            tabs: TabsState::new(vec!["Overview", "CPU", "GPU", "SoC"]),
            accent_color_: accent_color,
            gauge_bg_color_: gauge_bg_color,
            last_update: std::time::Instant::now(),
            metrics: None,
            soc_info,
            history: HashMap::new(),
        }
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            'x' => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }
    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    //     /// Update the app state.
    //     pub fn on_tick(&mut self) {
    //         // Update progress by 1%, resetting if 100%.
    //         self.progress = (self.progress + 0.01) % 1.0;
    //     }

    /// Update the app state.
    pub fn on_metrics(&mut self, metrics: Metrics) {
        self.last_update = std::time::Instant::now();

        self.history
            .entry("package_w".to_string())
            .or_insert(signal::Signal::with_capacity(
                HISTORY_CAPACITY,
                /* max */ self.soc_info.max_package_w as f32,
            ))
            .push(metrics.package_w);

        self.history
            .entry("cpu_w".to_string())
            .or_insert(signal::Signal::with_capacity(
                HISTORY_CAPACITY,
                /* max */ self.soc_info.max_cpu_w as f32,
            ))
            .push(metrics.cpu_w);

        for e_cluster in &metrics.e_clusters {
            let sig_name = format!("{}_active_ratio", e_cluster.name);
            self.history
                .entry(sig_name)
                .or_insert(signal::Signal::with_capacity(
                    HISTORY_CAPACITY,
                    /* max */ 100.0,
                ))
                .push(100.0 * e_cluster.active_ratio());
        }

        for p_cluster in &metrics.p_clusters {
            let sig_name = format!("{}_active_ratio", p_cluster.name);
            self.history
                .entry(sig_name)
                .or_insert(signal::Signal::with_capacity(
                    HISTORY_CAPACITY,
                    /* max */ 100.0,
                ))
                .push(100.0 * p_cluster.active_ratio());
        }

        self.history
            .entry("gpu_active_ratio".to_string())
            .or_insert(signal::Signal::with_capacity(
                HISTORY_CAPACITY,
                /* max */ 100.0,
            ))
            .push(100.0 * metrics.gpu.active_ratio as f32);

        self.history
            .entry("gpu_w".to_string())
            .or_insert(signal::Signal::with_capacity(
                HISTORY_CAPACITY,
                /* max */ 100.0,
            ))
            .push(metrics.gpu_w);

        self.history
            .entry("ane_active_ratio".to_string())
            .or_insert(signal::Signal::with_capacity(
                HISTORY_CAPACITY,
                /* max */ 100.0,
            ))
            .push(100.0 * metrics.ane_w / self.soc_info.max_ane_w as f32);

        self.history
            .entry("ane_w".to_string())
            .or_insert(signal::Signal::with_capacity(
                HISTORY_CAPACITY,
                /* max */ 100.0,
            ))
            .push(metrics.ane_w);

        self.metrics = Some(metrics);
    }

    fn color(code: u8) -> ratatui::style::Color {
        ratatui::style::Color::Indexed(code)
    }

    pub(crate) fn accent_color(&self) -> ratatui::style::Color {
        Self::color(self.accent_color_)
    }

    pub fn gauge_bg_color(&self) -> ratatui::style::Color {
        Self::color(self.gauge_bg_color_)
    }
}
