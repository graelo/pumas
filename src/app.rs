//! Hold the application state.

use crate::parser::{powermetrics::PowerMetrics, soc::Soc};

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
        self.metrics = Some(metrics);
        self.last_update = std::time::Instant::now();
        // println!("on_metrics: {:?}", metrics.e_clusters.len());
    }
}
