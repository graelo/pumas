///! Parse the plist output by `powermetrics`.
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Metrics {
    // pub(crate) hw_model: String,
    pub(crate) elapsed_ns: u64,
    pub(crate) processor: Processor,
    pub(crate) thermal_pressure: String,
    pub(crate) gpu: Gpu,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Processor {
    pub(crate) clusters: Vec<Cluster>,

    /// Energy consumed by the ANE in mJ.
    #[serde(rename = "ane_energy")]
    pub(crate) ane_mj: u16,
    /// Energy consumed by the CPU in mJ.
    #[serde(rename = "cpu_energy")]
    pub(crate) cpu_mj: u16,
    /// Energy consumed by the GPU in mJ.
    #[serde(rename = "gpu_energy")]
    pub(crate) gpu_mj: u16,
    /// Power consumed by the package in mW.
    #[serde(rename = "combined_power")]
    pub(crate) package_mw: f32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Cluster {
    pub(crate) name: String,
    pub(crate) freq_hz: f64,
    pub(crate) idle_ratio: f64,
    pub(crate) dvfm_states: Vec<DvfmState>,
    pub(crate) cpus: Vec<Cpu>,
}

impl Cluster {
    pub(crate) fn freq_mhz(&self) -> f64 {
        self.freq_hz / 1e6
    }
    pub(crate) fn active_ratio(&self) -> f64 {
        1.0 - self.idle_ratio
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Cpu {
    #[serde(rename = "cpu")]
    pub(crate) cpu_id: u16,
    pub(crate) freq_hz: f64,
    pub(crate) idle_ratio: f64,
    pub(crate) dvfm_states: Vec<DvfmState>,
}

impl Cpu {
    pub(crate) fn freq_mhz(&self) -> f64 {
        self.freq_hz / 1e6
    }
    pub(crate) fn active_ratio(&self) -> f64 {
        1.0 - self.idle_ratio
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Gpu {
    #[serde(rename = "freq_hz")]
    pub(crate) freq_mhz: f64,
    pub(crate) idle_ratio: f64,
    pub(crate) dvfm_states: Vec<DvfmState>,
}

impl Gpu {
    pub(crate) fn active_ratio(&self) -> f64 {
        1.0 - self.idle_ratio
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct DvfmState {
    #[serde(rename = "freq")]
    pub(crate) freq_mhz: u16,
    pub(crate) used_ratio: f64,
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
        assert_eq!(c0.active_ratio(), 1.0 - 0.772993);
        // cluster dvfm_states.
        assert_eq!(c0.dvfm_states[0].freq_mhz, 600);
        assert_eq!(c0.dvfm_states[0].used_ratio, 0.0);
        assert_eq!(c0.dvfm_states[1].freq_mhz, 972);
        assert_eq!(c0.dvfm_states[1].used_ratio, 0.919834);
        assert_eq!(c0.dvfm_states[2].freq_mhz, 1332);
        assert_eq!(c0.dvfm_states[2].used_ratio, 0.043774);
        assert_eq!(c0.dvfm_states[3].freq_mhz, 1704);
        assert_eq!(c0.dvfm_states[3].used_ratio, 0.0128986);
        assert_eq!(c0.dvfm_states[4].freq_mhz, 2064);
        assert_eq!(c0.dvfm_states[4].used_ratio, 0.0234935);

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
        assert_eq!(c0.cpus[0].dvfm_states[0].used_ratio, 0.0);
        assert_eq!(c0.cpus[0].dvfm_states[1].freq_mhz, 972);
        assert_eq!(c0.cpus[0].dvfm_states[1].used_ratio, 0.078834);
        assert_eq!(c0.cpus[0].dvfm_states[2].freq_mhz, 1332);
        assert_eq!(c0.cpus[0].dvfm_states[2].used_ratio, 0.00913338);

        let c1 = &pm.processor.clusters[1];
        assert_eq!(&c1.name[..], "P-Cluster");
        assert_eq!(c1.freq_mhz(), 618.173);
        assert_eq!(c1.active_ratio(), 1.0 - 0.983957);
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
        assert_eq!(pm.gpu.dvfm_states[0].used_ratio, 0.000265531);
        assert_eq!(pm.gpu.dvfm_states[1].freq_mhz, 528);
        assert_eq!(pm.gpu.dvfm_states[1].used_ratio, 0.0);
        assert_eq!(pm.gpu.dvfm_states[2].freq_mhz, 720);
        assert_eq!(pm.gpu.dvfm_states[2].used_ratio, 0.0163933);
        assert_eq!(pm.gpu.dvfm_states[3].freq_mhz, 924);
        assert_eq!(pm.gpu.dvfm_states[3].used_ratio, 0.0);
        assert_eq!(pm.gpu.dvfm_states[4].freq_mhz, 1128);
        assert_eq!(pm.gpu.dvfm_states[4].used_ratio, 0.0);
        assert_eq!(pm.gpu.dvfm_states[5].freq_mhz, 1278);
        assert_eq!(pm.gpu.dvfm_states[5].used_ratio, 0.0);
    }
}
