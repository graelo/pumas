//! Hold the application state.

use std::collections::HashMap;

use crate::{
    parser::{powermetrics::PowerMetrics, soc::Soc},
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

    /// Time of last update.
    pub(crate) last_update: std::time::Instant,

    /// Power metrics.
    pub(crate) metrics: Option<PowerMetrics>,

    /// System-on-chip information.
    pub(crate) soc: Soc,

    /// Store the history of all signals (u64 needed for `Sparkline`).
    pub(crate) history: History,
}

impl<'a> App<'a> {
    /// Returns a new `App`.
    pub fn new(soc_info: Soc) -> Self {
        Self {
            should_quit: false,
            tabs: TabsState::new(vec!["Overview", "CPU", "GPU", "SoC"]),
            last_update: std::time::Instant::now(),
            metrics: None,
            soc: soc_info,
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
    pub fn on_metrics(&mut self, metrics: PowerMetrics) {
        self.last_update = std::time::Instant::now();

        self.history
            .entry("package_w".to_string())
            .or_insert(signal::Signal::with_capacity(
                HISTORY_CAPACITY,
                /* max */ self.soc.max_package_w as f32,
            ))
            .push(metrics.package_w);

        for e_cluster in &metrics.e_clusters {
            let sig_name = format!("{}_active_ratio", e_cluster.name);
            self.history
                .entry(sig_name)
                .or_insert(signal::Signal::with_capacity(
                    HISTORY_CAPACITY,
                    /* max */ 100.0,
                ))
                .push(100.0 * e_cluster.active_ratio as f32);
        }

        for p_cluster in &metrics.p_clusters {
            let sig_name = format!("{}_active_ratio", p_cluster.name);
            self.history
                .entry(sig_name)
                .or_insert(signal::Signal::with_capacity(
                    HISTORY_CAPACITY,
                    /* max */ 100.0,
                ))
                .push(100.0 * p_cluster.active_ratio as f32);
        }

        self.history
            .entry("gpu_active_ratio".to_string())
            .or_insert(signal::Signal::with_capacity(
                HISTORY_CAPACITY,
                /* max */ 100.0,
            ))
            .push(100.0 * metrics.gpu.active_ratio as f32);

        self.history
            .entry("ane_active_ratio".to_string())
            .or_insert(signal::Signal::with_capacity(
                HISTORY_CAPACITY,
                /* max */ 100.0,
            ))
            .push(100.0 * metrics.ane_w / self.soc.max_ane_w as f32);

        self.metrics = Some(metrics);
    }
}
