//! Power and Usage metrics coming from the macOS `powermetrics` tool.
//!
//! These metrics are represented a bit differently than at the parsing stage (in `plist_parsing`)
//! in order to simplify computations and simplify access from the UI.

use std::str::FromStr;

use super::plist_parsing;
use crate::error::Error;

/// Reformulated metrics from the output of the `powermetrics` tool.
///
/// # Note
///
/// - Mx chips have a single E cluster and a single P cluster.
/// - Mx Pro chips have one E cluster and two P clusters.
/// - Mx Max chips have one E cluster and two P clusters.
/// - Mx Ultra chips have multiple E clusters and multiple P clusters.
///
pub(crate) struct Metrics {
    /// Efficiency Cluster metrics.
    pub(crate) e_clusters: Vec<ClusterMetrics>,
    /// Performance Cluster metrics.
    pub(crate) p_clusters: Vec<ClusterMetrics>,
    /// GPU metrics.
    pub(crate) gpu: GpuMetrics,
    /// CPU power consumption in W.
    pub(crate) cpu_w: f32,
    /// GPU power consumption in W.
    pub(crate) gpu_w: f32,
    /// Apple Neural Engine power consumption in W.
    pub(crate) ane_w: f32,
    /// Package power consumption in W.
    pub(crate) package_w: f32,
    /// Thermal pressure.
    pub(crate) thermal_pressure: String,
}

impl FromStr for Metrics {
    type Err = Error;

    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let pm: plist_parsing::Metrics = plist::from_bytes(content.as_bytes())
            .map_err(|e| Error::PlistParsingError(e.to_string()))?;
        Ok(Self::from(pm))
    }
}

impl Metrics {
    pub(crate) fn from_bytes(content: &[u8]) -> Result<Self, Error> {
        let pm: plist_parsing::Metrics =
            plist::from_bytes(content).map_err(|e| Error::PlistParsingError(e.to_string()))?;
        Ok(Self::from(pm))
    }

    /// Total number of CPUs on the chip.
    pub(crate) fn num_cpus(&self) -> usize {
        let mut total = 0;
        self.e_clusters.iter().for_each(|c| total += c.cpus.len());
        self.p_clusters.iter().for_each(|c| total += c.cpus.len());
        total
    }

    /// Override the CPU active ratio with the values provided by sysinfo.
    ///
    /// Yes this is ugly, but it's the only way to get the correct active ratio given that the
    /// powermetrics tool reports incorrect values on M2 chips.
    ///
    /// # Note
    ///
    /// This can be improved by aligning the CPU ids between the two tools.
    /// TODO: align CPU ids between powermetrics and sysinfo.
    ///
    pub(crate) fn set_cpus_active_ratio(mut self, active_ratios: &[f32]) -> Self {
        assert_eq!(
            self.num_cpus(),
            active_ratios.len(),
            "The number of active ratios (provided by sysinfo) must match the number of cpus."
        );

        let mut active_ratios = active_ratios.iter();
        for e_cluster in &mut self.e_clusters {
            for cpu in &mut e_cluster.cpus {
                cpu.active_ratio = *active_ratios.next().unwrap() as f64;
            }
        }
        for p_cluster in &mut self.p_clusters {
            for cpu in &mut p_cluster.cpus {
                cpu.active_ratio = *active_ratios.next().unwrap() as f64;
            }
        }

        self
    }
}

impl From<plist_parsing::Metrics> for Metrics {
    /// Create a new `Metrics` instance from the given `plist_parsing::Metrics` instance, and
    /// a time interval in milliseconds.
    ///
    /// Some CPUs (M1 Ultra) have multiple E clusters and multiple P clusters, so we create an
    /// aggregated E cluster which has the max frequency of all E clusters, and mean used ratio of
    /// all E clusters. Same applies for P clusters.
    ///
    fn from(value: plist_parsing::Metrics) -> Self {
        let interval_sec = value.elapsed_ns as f64 / 1e9;

        // Collect all E clusters.
        let e_clusters = value
            .processor
            .clusters
            .iter()
            .filter(|c| c.name.starts_with('E'))
            .map(ClusterMetrics::from)
            .collect::<Vec<_>>();

        // Collect all P clusters.
        let p_clusters = value
            .processor
            .clusters
            .iter()
            .filter(|c| c.name.starts_with('P'))
            .map(ClusterMetrics::from)
            .collect::<Vec<_>>();

        let gpu = GpuMetrics::from(&value.gpu);

        let cpu_w = (value.processor.cpu_mj as f64 / interval_sec / 1e3) as f32;
        let gpu_w = (value.processor.gpu_mj as f64 / interval_sec / 1e3) as f32;
        let ane_w = (value.processor.ane_mj as f64 / interval_sec / 1e3) as f32;
        let package_w = value.processor.package_mw / 1e3;

        Self {
            e_clusters,
            p_clusters,
            gpu,
            cpu_w,
            gpu_w,
            ane_w,
            package_w,
            thermal_pressure: value.thermal_pressure,
        }
    }
}

/// Metrics for a single cluster.
pub(crate) struct ClusterMetrics {
    /// Cluster name: e.g. "E-Cluster" or "P-Cluster", or "P0-Cluster", "P1-Cluster", etc.
    pub(crate) name: String,
    /// Cluster frequency (max of all CPUs) in MHz.
    pub(crate) freq_mhz: f64,
    /// Cluster dvfm states.
    pub(crate) dvfm_states: Vec<DvfmState>,
    /// Individual CPU metrics.
    pub(crate) cpus: Vec<CpuMetrics>,
}

impl ClusterMetrics {
    /// Cluster active ratio (mean of all CPU active ratios).
    pub(crate) fn active_ratio(&self) -> f32 {
        self.cpus.iter().map(|c| c.active_ratio as f32).sum::<f32>() / self.cpus.len() as f32
    }
}

impl From<&plist_parsing::ClusterMetrics> for ClusterMetrics {
    fn from(value: &plist_parsing::ClusterMetrics) -> Self {
        Self {
            name: value.name.clone(),
            freq_mhz: value.freq_mhz(),
            dvfm_states: value.dvfm_states.iter().map(DvfmState::from).collect(),
            cpus: value.cpus.iter().map(CpuMetrics::from).collect(),
        }
    }
}

/// Metrics for a single CPU.
pub(crate) struct CpuMetrics {
    /// CPU ID.
    pub(crate) id: u16,
    /// CPU frequency in MHz.
    pub(crate) freq_mhz: f64,
    /// CPU active ratio.
    pub(crate) active_ratio: f64,
    /// CPU dvfm states.
    pub(crate) dvfm_states: Vec<DvfmState>,
}

impl From<&plist_parsing::Cpu> for CpuMetrics {
    fn from(value: &plist_parsing::Cpu) -> Self {
        Self {
            id: value.cpu_id,
            freq_mhz: value.freq_mhz(),
            active_ratio: value.active_ratio(),
            dvfm_states: value.dvfm_states.iter().map(DvfmState::from).collect(),
        }
    }
}

/// Metrics for the GPU.
pub(crate) struct GpuMetrics {
    /// GPU frequency in MHz.
    pub(crate) freq_mhz: f64,
    /// GPU active ratio.
    pub(crate) active_ratio: f64,
    /// DVFM states.
    pub(crate) dvfm_states: Vec<DvfmState>,
}

impl From<&plist_parsing::GpuMetrics> for GpuMetrics {
    fn from(value: &plist_parsing::GpuMetrics) -> Self {
        Self {
            freq_mhz: value.freq_mhz,
            active_ratio: value.active_ratio(),
            dvfm_states: value.dvfm_states.iter().map(DvfmState::from).collect(),
        }
    }
}

/// Frequency ratios (from dynamic voltage and frequency management).
#[derive(Debug, PartialEq)]
pub(crate) struct DvfmState {
    pub(crate) freq_mhz: u16,
    pub(crate) active_ratio: f64,
}

impl From<&plist_parsing::DvfmState> for DvfmState {
    fn from(value: &plist_parsing::DvfmState) -> Self {
        Self {
            freq_mhz: value.freq_mhz,
            active_ratio: value.active_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powermetrics() {
        let content = std::fs::read_to_string("./tests/data/powermetrics-output-m1.xml")
            .expect("failed to read the file");
        let powermetrics = Metrics::from_str(&content).unwrap();
        // let pm: Metrics = plist::from_bytes(content.as_bytes()).expect("failed to parse the plist");

        // let metrics: plist_parsing::Metrics =
        //     plist::from_file("tests/data/powermetrics-output-m1.xml").unwrap();
        // let powermetrics = Metrics::from(metrics);

        // E cluster 0.
        assert_eq!(powermetrics.e_clusters[0].freq_mhz, 1022.87);
        // assert_eq!(powermetrics.e_clusters[0].active_ratio, 1.0 - 0.772993);

        // E cluster 0 DVFM states.
        assert_eq!(powermetrics.e_clusters[0].dvfm_states[0].freq_mhz, 600);
        assert_eq!(powermetrics.e_clusters[0].dvfm_states[0].active_ratio, 0.0);
        assert_eq!(powermetrics.e_clusters[0].dvfm_states[1].freq_mhz, 972);
        assert_eq!(
            powermetrics.e_clusters[0].dvfm_states[1].active_ratio,
            0.919834
        );

        // E cluster 0 CPUs.
        let cpus = &powermetrics.e_clusters[0].cpus;
        assert_eq!(cpus.len(), 4);
        assert_eq!(cpus[0].id, 0);
        assert_eq!(cpus[0].freq_mhz, 1046.15);
        assert_eq!(cpus[0].active_ratio, 1.0 - 0.907821);
        assert_eq!(cpus[0].dvfm_states[0].freq_mhz, 600);
        assert_eq!(cpus[0].dvfm_states[0].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[1].freq_mhz, 972);
        assert_eq!(cpus[0].dvfm_states[1].active_ratio, 0.078834);
        assert_eq!(cpus[0].dvfm_states[2].freq_mhz, 1332);
        assert_eq!(cpus[0].dvfm_states[2].active_ratio, 0.00913338);
        assert_eq!(cpus[0].dvfm_states[3].freq_mhz, 1704);
        assert_eq!(cpus[0].dvfm_states[3].active_ratio, 0.00292666);
        assert_eq!(cpus[0].dvfm_states[4].freq_mhz, 2064);
        assert_eq!(cpus[0].dvfm_states[4].active_ratio, 0.00128528);
        assert_eq!(cpus[1].id, 1);
        assert_eq!(cpus[1].freq_mhz, 1057.48);
        assert_eq!(cpus[1].active_ratio, 1.0 - 0.907626);
        assert_eq!(cpus[2].id, 2);
        assert_eq!(cpus[2].freq_mhz, 1084.65);
        assert_eq!(cpus[2].active_ratio, 1.0 - 0.906645);
        assert_eq!(cpus[3].id, 3);
        assert_eq!(cpus[3].freq_mhz, 1010.65);
        assert_eq!(cpus[3].active_ratio, 1.0 - 0.946967);

        // P cluster 0.
        assert_eq!(powermetrics.p_clusters[0].freq_mhz, 618.173);
        // assert_eq!(powermetrics.p_clusters[0].active_ratio, 1.0 - 0.983957);

        // P cluster 0 DVFM states.
        assert_eq!(powermetrics.p_clusters[0].dvfm_states[0].freq_mhz, 600);

        // P cluster 0 CPUs.
        let cpus = &powermetrics.p_clusters[0].cpus;
        assert_eq!(cpus.len(), 4);
        assert_eq!(cpus[0].id, 4);
        assert_eq!(cpus[0].freq_mhz, 1026.43);
        assert_eq!(cpus[0].active_ratio, 1.0 - 0.988368);
        assert_eq!(cpus[0].dvfm_states[0].freq_mhz, 600);
        assert_eq!(cpus[0].dvfm_states[0].active_ratio, 0.000163299);
        assert_eq!(cpus[0].dvfm_states[1].freq_mhz, 828);
        assert_eq!(cpus[0].dvfm_states[1].active_ratio, 0.00255751);
        assert_eq!(cpus[0].dvfm_states[2].freq_mhz, 1056);
        assert_eq!(cpus[0].dvfm_states[2].active_ratio, 0.00753595);
        assert_eq!(cpus[0].dvfm_states[3].freq_mhz, 1284);
        assert_eq!(cpus[0].dvfm_states[3].active_ratio, 0.00137491);
        assert_eq!(cpus[0].dvfm_states[4].freq_mhz, 1500);
        assert_eq!(cpus[0].dvfm_states[4].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[5].freq_mhz, 1728);
        assert_eq!(cpus[0].dvfm_states[5].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[6].freq_mhz, 1956);
        assert_eq!(cpus[0].dvfm_states[6].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[7].freq_mhz, 2184);
        assert_eq!(cpus[0].dvfm_states[7].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[8].freq_mhz, 2388);
        assert_eq!(cpus[0].dvfm_states[8].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[9].freq_mhz, 2592);
        assert_eq!(cpus[0].dvfm_states[9].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[10].freq_mhz, 2772);
        assert_eq!(cpus[0].dvfm_states[10].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[11].freq_mhz, 2988);
        assert_eq!(cpus[0].dvfm_states[11].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[12].freq_mhz, 3096);
        assert_eq!(cpus[0].dvfm_states[12].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[13].freq_mhz, 3144);
        assert_eq!(cpus[0].dvfm_states[13].active_ratio, 0.0);
        assert_eq!(cpus[0].dvfm_states[14].freq_mhz, 3204);
        assert_eq!(cpus[0].dvfm_states[14].active_ratio, 0.0);
        assert_eq!(cpus[1].id, 5);
        assert_eq!(cpus[1].freq_mhz, 1030.07);
        assert_eq!(cpus[1].active_ratio, 1.0 - 0.989273);
    }
}
