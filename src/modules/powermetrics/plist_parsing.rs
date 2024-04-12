//! Parse the plist output by `powermetrics`.

use serde::Deserialize;

/// The top-level struct of the plist reports metrics for the CPU, GPU, ANE and thermal pressure.
///
/// The fields correspond to the order of appearance in the plist.
#[derive(Debug, Deserialize)]
pub(crate) struct Metrics {
    // pub(crate) hw_model: String,
    /// Sampling period in nanoseconds.
    pub(crate) elapsed_ns: u64,
    /// Metrics for the CPU, and energy consumption of the ANE, CPU and GPU (weird grouping
    /// indeed).
    pub(crate) processor: ProcessorMetrics,
    /// Thermal pressure, one of "Nominal", "Light", "Moderate", "Heavy" or "Critical".
    /// These enum variants are handled at a higher level in `powermetrics.rs`.
    pub(crate) thermal_pressure: String,
    /// Basic metrics for the GPU.
    pub(crate) gpu: GpuMetrics,
}

/// Processor metrics, including energy consumption of the ANE, CPU and GPU.
///
/// # Note
///
/// The energy consumption of the ANE is the only way to estimate its activity.
#[derive(Debug, Deserialize)]
pub(crate) struct ProcessorMetrics {
    pub(crate) clusters: Vec<ClusterMetrics>,

    /// Energy consumed by the ANE in mJ over the sampling period.
    #[serde(rename = "ane_energy")]
    pub(crate) ane_mj: u16,
    /// Energy consumed by the CPU in mJ over the sampling period.
    #[serde(rename = "cpu_energy")]
    pub(crate) cpu_mj: u32,
    /// Energy consumed by the GPU in mJ over the sampling period.
    #[serde(rename = "gpu_energy")]
    pub(crate) gpu_mj: u32,
    /// Average power consumed by the package in mW over the sampling period.
    #[serde(rename = "combined_power")]
    pub(crate) package_mw: f32,
}

/// Metrics for a single CPU cluster. The metrics are averaged over the sampling period and all
/// CPUs of the cluster.
#[derive(Debug, Deserialize)]
pub(crate) struct ClusterMetrics {
    /// Name of the cluster, usually "E-Cluster" or "P-Cluster".
    pub(crate) name: String,
    /// Average frequency of the cluster in Hz.
    pub(crate) freq_hz: f64,
    // /// Average idle ratio of the cluster.
    // On M2 chips, powermetrics reports incorrect values.
    // pub(crate) idle_ratio: f64,
    /// Average frequency states of the cluster.
    pub(crate) dvfm_states: Vec<DvfmState>,
    /// Per-CPU metrics.
    pub(crate) cpus: Vec<Cpu>,
}

impl ClusterMetrics {
    /// Average frequency of the cluster in MHz.
    pub(crate) fn freq_mhz(&self) -> f64 {
        self.freq_hz / 1e6
    }
    // /// Average active ratio of the cluster.
    // On M2 chips, powermetrics reports incorrect values.
    // pub(crate) fn active_ratio(&self) -> f64 {
    //     1.0 - self.idle_ratio
    // }
}

/// Metrics for a single CPU. The metrics are averaged over the sampling period.
#[derive(Debug, Deserialize)]
pub(crate) struct Cpu {
    /// ID of the CPU: from 0 to n-1.
    #[serde(rename = "cpu")]
    pub(crate) cpu_id: u16,
    /// Average frequency of the CPU in Hz.
    pub(crate) freq_hz: f64,
    /// Average idle ratio of the CPU.
    pub(crate) idle_ratio: f64,
    /// Average frequency states of the CPU.
    pub(crate) dvfm_states: Vec<DvfmState>,
}

impl Cpu {
    /// Average frequency of the CPU in MHz.
    pub(crate) fn freq_mhz(&self) -> f64 {
        self.freq_hz / 1e6
    }
    /// Average active ratio of the CPU.
    pub(crate) fn active_ratio(&self) -> f64 {
        1.0 - self.idle_ratio
    }
}

/// Metrics for the GPU. The metrics are averaged over the sampling period and all the GPU cores.
#[derive(Debug, Deserialize)]
pub(crate) struct GpuMetrics {
    /// Average frequency of the GPU in Hz.
    #[serde(rename = "freq_hz")]
    pub(crate) freq_mhz: f64,
    /// Average idle ratio of the GPU.
    pub(crate) idle_ratio: f64,
    /// Average frequency states of the GPU.
    pub(crate) dvfm_states: Vec<DvfmState>,
}

impl GpuMetrics {
    /// Average frequency of the GPU in MHz.
    pub(crate) fn active_ratio(&self) -> f64 {
        1.0 - self.idle_ratio
    }
}

/// Frequency state of a CPU or GPU. The metrics are averaged over the sampling period.
#[derive(Debug, Deserialize)]
pub(crate) struct DvfmState {
    /// Average frequency of the state in MHz.
    #[serde(rename = "freq")]
    pub(crate) freq_mhz: u16,
    /// Average active ratio of the state.
    #[serde(rename = "used_ratio")]
    pub(crate) active_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn read_file_m1() {
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

    #[test]
    fn read_file_m2ultra() {
        // Read the file.
        let content = std::fs::read_to_string("./tests/data/powermetrics-output-m2ultra.xml")
            .expect("failed to read the file");
        let pm: Metrics = plist::from_bytes(content.as_bytes()).expect("failed to parse the plist");

        // assert_eq!(&pm.hw_model[..], "MacBookPro17,1");

        let c0 = &pm.processor.clusters[0];
        assert_eq!(&c0.name[..], "E0-Cluster");
        assert_eq!(c0.freq_mhz(), 990.818);
        // assert_eq!(c0.active_ratio(), 1.0 - 0.772993);

        // cluster dvfm_states.
        assert_eq!(c0.dvfm_states[0].freq_mhz, 912);
        assert_eq!(c0.dvfm_states[0].active_ratio, 0.88102);
        assert_eq!(c0.dvfm_states[1].freq_mhz, 1284);
        assert_eq!(c0.dvfm_states[1].active_ratio, 0.0153791);
        assert_eq!(c0.dvfm_states[2].freq_mhz, 1752);
        assert_eq!(c0.dvfm_states[2].active_ratio, 0.0324065);
        assert_eq!(c0.dvfm_states[3].freq_mhz, 2004);
        assert_eq!(c0.dvfm_states[3].active_ratio, 0.00133004);
        assert_eq!(c0.dvfm_states[4].freq_mhz, 2256);
        assert_eq!(c0.dvfm_states[4].active_ratio, 0.012331);

        assert_eq!(c0.cpus[0].cpu_id, 0);
        assert_eq!(c0.cpus[1].cpu_id, 1);
        assert_eq!(c0.cpus[2].cpu_id, 2);
        assert_eq!(c0.cpus[3].cpu_id, 3);
        assert_eq!(c0.cpus[0].freq_mhz(), 979.723);
        assert_eq!(c0.cpus[1].freq_mhz(), 1041.27);
        assert_eq!(c0.cpus[2].freq_mhz(), 1028.45);
        assert_eq!(c0.cpus[3].freq_mhz(), 958.188);
        assert_eq!(c0.cpus[0].active_ratio(), 1.0 - 0.440453);
        assert_eq!(c0.cpus[1].active_ratio(), 1.0 - 0.56774);
        assert_eq!(c0.cpus[2].active_ratio(), 1.0 - 0.695484);
        // cpu dvfm_states.
        assert_eq!(c0.cpus[0].dvfm_states[0].freq_mhz, 912);
        assert_eq!(c0.cpus[0].dvfm_states[0].active_ratio, 0.516857);
        assert_eq!(c0.cpus[0].dvfm_states[1].freq_mhz, 1284);
        assert_eq!(c0.cpus[0].dvfm_states[1].active_ratio, 0.0105143);
        assert_eq!(c0.cpus[0].dvfm_states[2].freq_mhz, 1752);
        assert_eq!(c0.cpus[0].dvfm_states[2].active_ratio, 0.0192085);

        let c1 = &pm.processor.clusters[1];
        assert_eq!(&c1.name[..], "P0-Cluster");
        assert_eq!(c1.freq_mhz(), 2775.02);
        // assert_eq!(c1.active_ratio(), 1.0 - 0.983957);

        // cluster dvfm_states.
        assert_eq!(c1.cpus[0].cpu_id, 4);
        assert_eq!(c1.cpus[1].cpu_id, 5);
        assert_eq!(c1.cpus[2].cpu_id, 6);
        assert_eq!(c1.cpus[3].cpu_id, 7);
        assert_eq!(c1.cpus[0].freq_mhz(), 2189.62);
        assert_eq!(c1.cpus[1].freq_mhz(), 3470.68);
        assert_eq!(c1.cpus[2].freq_mhz(), 3084.83);
        assert_eq!(c1.cpus[3].freq_mhz(), 3160.17);

        assert_eq!(pm.processor.ane_mj, 0);
        assert_eq!(pm.processor.cpu_mj, 855);
        assert_eq!(pm.processor.gpu_mj, 72607);
        assert_eq!(pm.processor.package_mw, 71759.8);

        assert_eq!(&pm.thermal_pressure[..], "Nominal");

        assert_eq!(pm.gpu.freq_mhz, 1360.85);
        assert_approx_eq!(pm.gpu.active_ratio(), 1_f64 - 0.0536782, 1e-5_f64);
        // cpu dvfm_states.
        assert_eq!(pm.gpu.dvfm_states[0].freq_mhz, 444);
        assert_eq!(pm.gpu.dvfm_states[0].active_ratio, 0.000297563);
        assert_eq!(pm.gpu.dvfm_states[1].freq_mhz, 612);
        assert_eq!(pm.gpu.dvfm_states[1].active_ratio, 0.0);
        assert_eq!(pm.gpu.dvfm_states[2].freq_mhz, 808);
        assert_eq!(pm.gpu.dvfm_states[2].active_ratio, 0.0);
        assert_eq!(pm.gpu.dvfm_states[3].freq_mhz, 968);
        assert_eq!(pm.gpu.dvfm_states[3].active_ratio, 0.0);
        assert_eq!(pm.gpu.dvfm_states[4].freq_mhz, 1110);
        assert_eq!(pm.gpu.dvfm_states[4].active_ratio, 0.0);
        assert_eq!(pm.gpu.dvfm_states[5].freq_mhz, 1236);
        assert_eq!(pm.gpu.dvfm_states[5].active_ratio, 0.00113613);
    }
}
