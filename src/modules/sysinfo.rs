//! Metrics obtained via the sysinfo crate.
//!
//! Currently, this provides:
//! - Memory usage
//! - CPU usage per core, which is more accurate than the CPU usage obtained
//!   via powermetrics on M2 chips.

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, System};

pub(crate) struct CpuMetrics {
    /// CPU ID (0 - ...)
    pub(crate) id: u16,
    /// Activity ratio (0.0 - 1.0).
    pub(crate) active_ratio: f32,
}

pub(crate) struct MemoryMetrics {
    pub(crate) ram_total: u64,
    pub(crate) ram_used: u64,
    pub(crate) swap_total: u64,
    pub(crate) swap_used: u64,
}

pub(crate) struct Metrics {
    pub(crate) cpu_metrics: Vec<CpuMetrics>,
    pub(crate) memory_metrics: MemoryMetrics,
}

pub(crate) struct SystemState {
    system: System,
}

impl SystemState {
    pub(crate) fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_specifics(CpuRefreshKind::default().with_cpu_usage());
        system.refresh_memory_specifics(MemoryRefreshKind::everything());
        Self { system }
    }

    pub(crate) fn latest_metrics(&mut self) -> Metrics {
        self.system
            .refresh_cpu_specifics(CpuRefreshKind::default().with_cpu_usage());
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::default().with_ram());
        self.system
            .refresh_memory_specifics(MemoryRefreshKind::default().with_swap());

        let cpu_metrics = self
            .system
            .cpus()
            .iter()
            .map(|cpu| CpuMetrics {
                id: cpu.name().parse::<u16>().unwrap_or(0).saturating_sub(1),
                active_ratio: cpu.cpu_usage() / 100.0_f32,
            })
            .collect();

        // Use vm_stat for better memory accounting on macOS, fallback to sysinfo if it fails
        let memory_metrics = if let Ok(vm_stats) = super::vm_stat::VmStats::collect() {
            MemoryMetrics {
                ram_total: self.system.total_memory(), // Use sysinfo for total memory (more reliable)
                ram_used: vm_stats.activity_monitor_memory_used(),
                swap_total: self.system.total_swap(),
                swap_used: self.system.used_swap(),
            }
        } else {
            // Fallback to sysinfo if vm_stat fails
            MemoryMetrics {
                ram_total: self.system.total_memory(),
                ram_used: self.system.used_memory(),
                swap_total: self.system.total_swap(),
                swap_used: self.system.used_swap(),
            }
        };

        Metrics {
            cpu_metrics,
            memory_metrics,
        }
    }
}
