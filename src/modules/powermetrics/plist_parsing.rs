//! Parse the plist output by `powermetrics`.

use serde::Deserialize;

/// The top-level struct of the plist reports metrics for the CPU, GPU, ANE and thermal pressure.
///
/// The fields correspond to the order of appearance in the plist.
#[derive(Debug, Deserialize)]
pub(super) struct Metrics {
    // pub(super) hw_model: String,
    /// Sampling period in nanoseconds.
    pub(super) elapsed_ns: u64,
    /// Metrics for the CPU, and energy consumption of the ANE, CPU and GPU (weird grouping
    /// indeed).
    pub(super) processor: ProcessorMetrics,
    /// Thermal pressure, one of "Nominal", "Light", "Moderate", "Heavy" or "Critical".
    /// These enum variants are handled at a higher level in `powermetrics.rs`.
    pub(super) thermal_pressure: String,
    /// Basic metrics for the GPU.
    pub(super) gpu: GpuMetrics,
}

/// Processor metrics, including energy consumption of the ANE, CPU and GPU.
///
/// # Note
///
/// The energy consumption of the ANE is the only way to estimate its activity.
#[derive(Debug, Deserialize)]
pub(super) struct ProcessorMetrics {
    pub(super) clusters: Vec<ClusterMetrics>,

    /// Energy consumed by the ANE in mJ over the sampling period.
    #[serde(rename = "ane_energy")]
    pub(super) ane_mj: u16,
    /// Energy consumed by the CPU in mJ over the sampling period.
    #[serde(rename = "cpu_energy")]
    pub(super) cpu_mj: u16,
    /// Energy consumed by the GPU in mJ over the sampling period.
    #[serde(rename = "gpu_energy")]
    pub(super) gpu_mj: u16,
    /// Average power consumed by the package in mW over the sampling period.
    #[serde(rename = "combined_power")]
    pub(super) package_mw: f32,
}

/// Metrics for a single CPU cluster. The metrics are averaged over the sampling period and all
/// CPUs of the cluster.
#[derive(Debug, Deserialize)]
pub(super) struct ClusterMetrics {
    /// Name of the cluster, usually "E-Cluster" or "P-Cluster".
    pub(super) name: String,
    /// Average frequency of the cluster in Hz.
    pub(super) freq_hz: f64,
    // /// Average idle ratio of the cluster.
    // On M2 chips, powermetrics reports incorrect values.
    // pub(super) idle_ratio: f64,
    /// Average frequency states of the cluster.
    pub(super) dvfm_states: Vec<DvfmState>,
    /// Per-CPU metrics.
    pub(super) cpus: Vec<Cpu>,
}

impl ClusterMetrics {
    /// Average frequency of the cluster in MHz.
    pub(super) fn freq_mhz(&self) -> f64 {
        self.freq_hz / 1e6
    }
    // /// Average active ratio of the cluster.
    // On M2 chips, powermetrics reports incorrect values.
    // pub(super) fn active_ratio(&self) -> f64 {
    //     1.0 - self.idle_ratio
    // }
}

/// Metrics for a single CPU. The metrics are averaged over the sampling period.
#[derive(Debug, Deserialize)]
pub(super) struct Cpu {
    /// ID of the CPU: from 0 to n-1.
    #[serde(rename = "cpu")]
    pub(super) cpu_id: u16,
    /// Average frequency of the CPU in Hz.
    pub(super) freq_hz: f64,
    /// Average idle ratio of the CPU.
    pub(super) idle_ratio: f64,
    /// Average frequency states of the CPU.
    pub(super) dvfm_states: Vec<DvfmState>,
}

impl Cpu {
    /// Average frequency of the CPU in MHz.
    pub(super) fn freq_mhz(&self) -> f64 {
        self.freq_hz / 1e6
    }
    /// Average active ratio of the CPU.
    pub(super) fn active_ratio(&self) -> f64 {
        1.0 - self.idle_ratio
    }
}

/// Metrics for the GPU. The metrics are averaged over the sampling period and all the GPU cores.
#[derive(Debug, Deserialize)]
pub(super) struct GpuMetrics {
    /// Average frequency of the GPU in Hz.
    #[serde(rename = "freq_hz")]
    pub(super) freq_mhz: f64,
    /// Average idle ratio of the GPU.
    pub(super) idle_ratio: f64,
    /// Average frequency states of the GPU.
    pub(super) dvfm_states: Vec<DvfmState>,
}

impl GpuMetrics {
    /// Average frequency of the GPU in MHz.
    pub(super) fn active_ratio(&self) -> f64 {
        1.0 - self.idle_ratio
    }
}

/// Frequency state of a CPU or GPU. The metrics are averaged over the sampling period.
#[derive(Debug, Deserialize)]
pub(super) struct DvfmState {
    /// Average frequency of the state in MHz.
    #[serde(rename = "freq")]
    pub(super) freq_mhz: u16,
    /// Average active ratio of the state.
    #[serde(rename = "used_ratio")]
    pub(super) active_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn read_file() {
        // Read the file.
        let content = std::fs::read_to_string("./tests/data/powermetrics-output-m1.xml")
            .expect("failed to read the file");
        let pm: Metrics = plist::from_bytes(content.as_bytes()).expect("failed to parse the plist");

        // assert_eq!(&pm.hw_model[..], "MacBookPro17,1");

        let c0 = &pm.processor.clusters[0];
        assert_eq!(&c0.name[..], "E-Cluster");
        assert_eq!(c0.freq_mhz(), 1022.87);
        // assert_eq!(c0.active_ratio(), 1.0 - 0.772993);

        // cluster dvfm_states.
        assert_eq!(c0.dvfm_states[0].freq_mhz, 600);
        assert_eq!(c0.dvfm_states[0].active_ratio, 0.0);
        assert_eq!(c0.dvfm_states[1].freq_mhz, 972);
        assert_eq!(c0.dvfm_states[1].active_ratio, 0.919834);
        assert_eq!(c0.dvfm_states[2].freq_mhz, 1332);
        assert_eq!(c0.dvfm_states[2].active_ratio, 0.043774);
        assert_eq!(c0.dvfm_states[3].freq_mhz, 1704);
        assert_eq!(c0.dvfm_states[3].active_ratio, 0.0128986);
        assert_eq!(c0.dvfm_states[4].freq_mhz, 2064);
        assert_eq!(c0.dvfm_states[4].active_ratio, 0.0234935);

        assert_eq!(c0.cpus[0].cpu_id, 0);
        assert_eq!(c0.cpus[1].cpu_id, 1);
        assert_eq!(c0.cpus[2].cpu_id, 2);
        assert_eq!(c0.cpus[3].cpu_id, 3);
        assert_eq!(c0.cpus[0].freq_mhz(), 1046.15);
        assert_eq!(c0.cpus[1].freq_mhz(), 1057.48);
        assert_eq!(c0.cpus[2].freq_mhz(), 1084.65);
        assert_eq!(c0.cpus[3].freq_mhz(), 1010.65);
        assert_eq!(c0.cpus[0].active_ratio(), 1.0 - 0.907821);
        assert_eq!(c0.cpus[1].active_ratio(), 1.0 - 0.907626);
        assert_eq!(c0.cpus[2].active_ratio(), 1.0 - 0.906645);
        assert_eq!(c0.cpus[3].active_ratio(), 1.0 - 0.946967);
        // cpu dvfm_states.
        assert_eq!(c0.cpus[0].dvfm_states[0].freq_mhz, 600);
        assert_eq!(c0.cpus[0].dvfm_states[0].active_ratio, 0.0);
        assert_eq!(c0.cpus[0].dvfm_states[1].freq_mhz, 972);
        assert_eq!(c0.cpus[0].dvfm_states[1].active_ratio, 0.078834);
        assert_eq!(c0.cpus[0].dvfm_states[2].freq_mhz, 1332);
        assert_eq!(c0.cpus[0].dvfm_states[2].active_ratio, 0.00913338);

        let c1 = &pm.processor.clusters[1];
        assert_eq!(&c1.name[..], "P-Cluster");
        assert_eq!(c1.freq_mhz(), 618.173);
        // assert_eq!(c1.active_ratio(), 1.0 - 0.983957);

        // cluster dvfm_states.
        assert_eq!(c1.cpus[0].cpu_id, 4);
        assert_eq!(c1.cpus[1].cpu_id, 5);
        assert_eq!(c1.cpus[2].cpu_id, 6);
        assert_eq!(c1.cpus[3].cpu_id, 7);
        assert_eq!(c1.cpus[0].freq_mhz(), 1026.43);
        assert_eq!(c1.cpus[1].freq_mhz(), 1030.07);
        assert_eq!(c1.cpus[2].freq_mhz(), 1033.73);
        assert_eq!(c1.cpus[3].freq_mhz(), 1015.09);

        assert_eq!(pm.processor.ane_mj, 0);
        assert_eq!(pm.processor.cpu_mj, 89);
        assert_eq!(pm.processor.gpu_mj, 31);
        assert_eq!(pm.processor.package_mw, 59.4301);

        assert_eq!(&pm.thermal_pressure[..], "Nominal");

        assert_eq!(pm.gpu.freq_mhz, 714.836);
        assert_approx_eq!(pm.gpu.active_ratio(), 1_f64 - 0.983341, 1e-5_f64);
        // cpu dvfm_states.
        assert_eq!(pm.gpu.dvfm_states[0].freq_mhz, 396);
        assert_eq!(pm.gpu.dvfm_states[0].active_ratio, 0.000265531);
        assert_eq!(pm.gpu.dvfm_states[1].freq_mhz, 528);
        assert_eq!(pm.gpu.dvfm_states[1].active_ratio, 0.0);
        assert_eq!(pm.gpu.dvfm_states[2].freq_mhz, 720);
        assert_eq!(pm.gpu.dvfm_states[2].active_ratio, 0.0163933);
        assert_eq!(pm.gpu.dvfm_states[3].freq_mhz, 924);
        assert_eq!(pm.gpu.dvfm_states[3].active_ratio, 0.0);
        assert_eq!(pm.gpu.dvfm_states[4].freq_mhz, 1128);
        assert_eq!(pm.gpu.dvfm_states[4].active_ratio, 0.0);
        assert_eq!(pm.gpu.dvfm_states[5].freq_mhz, 1278);
        assert_eq!(pm.gpu.dvfm_states[5].active_ratio, 0.0);
    }
}
