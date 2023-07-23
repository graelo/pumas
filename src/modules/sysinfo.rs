//! Metrics obtained via the sysinfo crate.
//!
//! Currently, this provides CPU usage per core. This is more accurate than the CPU usage obtained
//! via powermetrics on M2 chips.

use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};

pub(crate) struct CpuMetrics {
    // pub(crate) id: u16,
    /// Activity ratio (0.0 - 1.0).
    pub(crate) active_ratio: f32,
}

pub(crate) struct Metrics {
    pub(crate) cpu_metrics: Vec<CpuMetrics>,
}

pub(crate) struct SystemState {
    system: System,
}

impl SystemState {
    pub(crate) fn new() -> Self {
        let mut system =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
        system.refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());
        Self { system }
    }

    pub(crate) fn latest_metrics(&mut self) -> Metrics {
        self.system
            .refresh_cpu_specifics(CpuRefreshKind::new().with_cpu_usage());

        let cpu_metrics = self
            .system
            .cpus()
            .iter()
            .map(|cpu| CpuMetrics {
                // id: cpu.name().parse::<u16>().unwrap() - 1_u16,
                active_ratio: cpu.cpu_usage() / 100.0_f32,
            })
            .collect();

        Metrics { cpu_metrics }
    }
}
