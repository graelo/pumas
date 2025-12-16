//! Type-safe keys for metrics history.
//!
//! This module provides strongly-typed keys for accessing metrics in the history
//! HashMap, replacing stringly-typed keys with an enum for compile-time safety.

/// Identifies a CPU cluster by its kind and index.
///
/// Apple Silicon chips have efficiency (E) and performance (P) clusters.
/// Single-die chips (M1, M2, M3) have one of each, while multi-die chips
/// (M1 Ultra, M2 Ultra) have two of each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ClusterId {
    pub kind: ClusterKind,
    pub index: u8,
}

impl ClusterId {
    /// Create an efficiency cluster ID.
    pub const fn efficiency(index: u8) -> Self {
        Self {
            kind: ClusterKind::Efficiency,
            index,
        }
    }

    /// Create a performance cluster ID.
    pub const fn performance(index: u8) -> Self {
        Self {
            kind: ClusterKind::Performance,
            index,
        }
    }
}

/// The kind of CPU cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ClusterKind {
    /// Efficiency cores (E-cluster).
    Efficiency,
    /// Performance cores (P-cluster).
    Performance,
}

/// Type-safe key for accessing metrics in the history.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum MetricKey {
    // ─── Cluster metrics ───────────────────────────────────────────────────────
    /// Active ratio for a CPU cluster (0-100%).
    ClusterActivePercent(ClusterId),

    // ─── Per-CPU metrics ───────────────────────────────────────────────────────
    /// Active ratio for a specific CPU core (0-100%).
    CpuActivePercent(u16),
    /// Frequency ratio for a specific CPU core (0-100% of max freq).
    CpuFreqPercent(u16),

    // ─── GPU metrics ───────────────────────────────────────────────────────────
    /// GPU active ratio (0-100%).
    GpuActivePercent,
    /// GPU frequency ratio (0-100% of max freq).
    GpuFreqPercent,

    // ─── ANE metrics ───────────────────────────────────────────────────────────
    /// Apple Neural Engine active ratio (0-100%).
    AneActivePercent,

    // ─── Power consumption ─────────────────────────────────────────────────────
    /// CPU power consumption in watts.
    CpuPowerW,
    /// GPU power consumption in watts.
    GpuPowerW,
    /// ANE power consumption in watts.
    AnePowerW,
    /// Total package power (CPU + GPU + ANE) in watts.
    PackagePowerW,

    // ─── Memory ────────────────────────────────────────────────────────────────
    /// RAM usage in bytes.
    RamUsageBytes,
    /// Swap usage in bytes.
    SwapUsageBytes,
}
