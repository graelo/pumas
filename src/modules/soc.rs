//! System-on-Chip (SoC) information.

use std::process;

use crate::{Result, error::Error};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SocInfo {
    /// Brand name of the CPU, e.g. "Apple M1".
    pub(crate) cpu_brand_name: String,

    /// Number of CPU cores.
    pub(crate) num_cpu_cores: u16,

    /// Number of Efficiency cores.
    pub(crate) num_efficiency_cores: u16,

    /// Number of Performance cores.
    pub(crate) num_performance_cores: u16,

    /// Number of GPU cores.
    pub(crate) num_gpu_cores: u16,

    /// Maximum CPU power consumption.
    pub(crate) max_cpu_w: f64,

    /// Maximum GPU power consumption.
    pub(crate) max_gpu_w: f64,

    /// Maximum ANE power consumption.
    pub(crate) max_ane_w: f64,

    /// Max Package power consumption.
    pub(crate) max_package_w: f64,
}

impl SocInfo {
    pub(crate) fn new() -> Result<SocInfo> {
        let (cpu_brand_name, num_cpu_cores, num_efficiency_cores, num_performance_cores) =
            cpu_info()?;

        let num_gpu_cores = gpu_info()?;

        // Approximate peak power ceilings (watts) for scaling gauges and sparklines — not
        // measured TDP. Calibrated loosely against public summaries (e.g. MacNerd SoC peak table
        // citing AnandTech / NotebookCheck / Apple; TechRadar on M4 Max CPU ~48 W) and split
        // across CPU / GPU / ANE for separate widgets; sums sit near those “chip peak” ballparks.
        //
        // M5: base chip CPU ceiling from user testing (~25 W). M5 Pro / M5 Max:
        // NotebookCheck CPU analysis (~75 W peak CPU for the shared 18-core complex; GPU analysis
        // ~38 W Pro / ~72 W Max). M5 Ultra: chip not public — values are extrapolation only.
        let (max_cpu_w, max_gpu_w, max_ane_w) = match cpu_brand_name.as_str() {
            "Apple M1" => (20.0, 20.0, 8.0),
            "Apple M1 Pro" => (22.0, 18.0, 8.0),
            "Apple M1 Max" => (32.0, 52.0, 8.0),
            "Apple M1 Ultra" => (70.0, 130.0, 15.0),
            "Apple M2" => (22.0, 22.0, 8.0),
            "Apple M2 Pro" => (38.0, 52.0, 10.0),
            "Apple M2 Max" => (45.0, 90.0, 10.0),
            "Apple M2 Ultra" => (75.0, 200.0, 20.0),
            "Apple M3" => (14.0, 14.0, 8.0),
            "Apple M3 Pro" => (16.0, 16.0, 8.0),
            "Apple M3 Max" => (42.0, 66.0, 10.0),
            "Apple M3 Ultra" => (65.0, 120.0, 15.0),
            "Apple M4" => (18.0, 17.0, 8.0),
            "Apple M4 Pro" => (35.0, 35.0, 10.0),
            "Apple M4 Max" => (52.0, 83.0, 10.0),
            "Apple M5" => (25.0, 22.0, 8.0),
            "Apple M5 Pro" => (78.0, 40.0, 10.0),
            "Apple M5 Max" => (78.0, 75.0, 12.0),
            "Apple M5 Ultra" => (95.0, 175.0, 18.0),
            _ => (20.0, 20.0, 8.0),
        };

        Ok(SocInfo {
            cpu_brand_name,
            num_cpu_cores,
            num_efficiency_cores,
            num_performance_cores,
            max_cpu_w,
            max_gpu_w,
            max_ane_w,
            max_package_w: max_cpu_w + max_gpu_w + max_ane_w,
            num_gpu_cores,
        })
    }
}

fn cpu_info() -> Result<(String, u16, u16, u16)> {
    let binary = "/usr/sbin/sysctl";
    let args = &[
        "-n",
        "machdep.cpu.brand_string",
        "machdep.cpu.core_count",
        "hw.perflevel0.logicalcpu",
        "hw.perflevel1.logicalcpu",
    ];

    let output = process::Command::new(binary).args(args).output()?;
    let buffer = String::from_utf8(output.stdout)?;

    parse_cpu_info(&buffer)
}

fn parse_cpu_info(buffer: &str) -> Result<(String, u16, u16, u16)> {
    let mut iter = buffer.split('\n');

    let cpu_brand_name = match iter.next() {
        Some(s) => s.to_string(),
        None => return Err(Error::SocInfoParsingError(buffer.to_string())),
    };

    let num_cpu_cores = match iter.next() {
        Some(s) => s.parse::<u16>()?,
        None => return Err(Error::SocInfoParsingError(buffer.to_string())),
    };

    let num_performance_cores = match iter.next() {
        Some(s) => s.parse::<u16>()?,
        None => return Err(Error::SocInfoParsingError(buffer.to_string())),
    };

    let num_efficiency_cores = match iter.next() {
        Some(s) => s.parse::<u16>()?,
        None => return Err(Error::SocInfoParsingError(buffer.to_string())),
    };

    Ok((
        cpu_brand_name,
        num_cpu_cores,
        num_efficiency_cores,
        num_performance_cores,
    ))
}

fn gpu_info() -> Result<u16> {
    let binary = "/usr/sbin/system_profiler";
    let args = &["-detailLevel", "basic", "SPDisplaysDataType"];

    let output = process::Command::new(binary).args(args).output()?;
    let buffer = String::from_utf8(output.stdout)?;

    parse_gpu_info(&buffer)
}

fn parse_gpu_info(buffer: &str) -> Result<u16> {
    let num_gpu_cores_line = buffer
        .lines()
        .find(|&line| line.trim_start().starts_with("Total Number of Cores:"));

    let num_gpu_cores = match num_gpu_cores_line {
        Some(s) => match s.split(": ").last() {
            Some(s) => s.parse::<u16>()?,
            None => return Err(Error::SocInfoParsingError(buffer.to_string())),
        },
        None => return Err(Error::SocInfoParsingError(buffer.to_string())),
    };

    Ok(num_gpu_cores)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cpu_info_ok() {
        let buffer = "Apple M1\n8\n4\n4\n";

        let actual = parse_cpu_info(buffer).expect("Parsing CPU Info should succeed");
        let expected = ("Apple M1".to_string(), 8, 4, 4);

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_gpu_info_ok() {
        let buffer = "Graphics/Displays:

    Apple M1:

      Chipset Model: Apple M1
      Type: GPU
      Bus: Built-In
      Total Number of Cores: 8
      Vendor: Apple (0x106b)
      Metal Support: Metal 3
      Displays:
        Color LCD:
          Display Type: Built-In Retina LCD
          Resolution: 2560 x 1600 Retina
          Main Display: Yes
          Mirror: Off
          Online: Yes
          Automatically Adjust Brightness: Yes
          Connection Type: Internal
    ";

        let actual = parse_gpu_info(buffer).expect("Parsing GPU Info should succeed");
        let expected = 8;

        assert_eq!(actual, expected);
    }
}
