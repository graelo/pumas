//! Backend collector thread + data plane.
//!
//! A single OS thread owns the `powermetrics` subprocess, the sysinfo merge and
//! all signal [`history`], and ships an owned [`Frame`] per sample over a
//! `smol::channel`. The frontend never drives the backend.
//!
//! The same streaming loop powers `run --json`: [`run_exporter`] prints one JSON
//! line per sample instead of building a `Frame`, byte-identical to the previous
//! implementation.

pub(crate) mod frame;
pub(crate) mod history;

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    ops::ControlFlow,
    process::{self, Stdio},
    time::Duration,
};

use smol::channel::Sender;

use crate::{
    Result,
    config::RunConfig,
    error::Error as CrateError,
    metric_key::{ClusterId, MetricKey},
    metrics::{ClusterMetrics, CpuMetrics, Metrics},
    modules::{powermetrics, soc::SocInfo, sysinfo, vm_stat::VmStats},
    units,
};

use history::{History, HistoryExt};

use frame::ColorRole::{Accent, Default as Def, GaugeFg, HistoryFg};
use frame::{
    CpuCluster, CpuFrame, CpuRow, Frame, FreqTable, GpuFrame, MemLine, MemSpan, MemoryFrame, Meter,
    OverviewFrame, SparkText, Thermals,
};

/// Overshoot keeping sparkline bars from touching the gauge above
///. Applied everywhere except the Overview Package block.
const SPARKLINE_MAX_OVERSHOOT: f32 = 1.05;

/// Fixed sparkline window (in samples) for the per-core CPU/GPU rows.
const HISTORY_LENGTH: usize = 8;

// ─── Public entry points ────────────────────────────────────────────────────

/// Run the collector loop, shipping one [`Frame`] per sample over `tx`.
///
/// Owns the `History`. On send error (the UI is gone) the powermetrics
/// subprocess is killed and the loop returns.
pub(crate) fn run_collector(soc: SocInfo, run_config: RunConfig, tx: Sender<Frame>) -> Result<()> {
    let tick_rate = Duration::from_millis(u64::from(run_config.sample_rate_ms));
    let history_size = run_config.history_size;
    let mut history: History = HashMap::new();

    stream(tick_rate, |metrics| {
        update_history(&mut history, &soc, history_size, metrics);
        let frame = build_frame(metrics, &soc, &history);
        if tx.send_blocking(frame).is_err() {
            // UI dropped the receiver: stop streaming.
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
}

/// Run the JSON exporter loop: one `{"soc":…,"metrics":…}` line per sample.
pub(crate) fn run_exporter(soc: SocInfo, run_config: RunConfig) -> Result<()> {
    let tick_rate = Duration::from_millis(u64::from(run_config.sample_rate_ms));
    stream(tick_rate, |metrics| {
        println!("{}", export_line(&soc, metrics));
        ControlFlow::Continue(())
    })
}

/// Serialize one sample exactly as `run --json` prints it (Display of the
/// compact `serde_json::Value`).
fn export_line(soc: &SocInfo, metrics: &Metrics) -> String {
    serde_json::json!({ "soc": soc, "metrics": metrics }).to_string()
}

// ─── powermetrics streaming ─────────────────────────────────────────────────

/// Stream metrics from `powermetrics`, invoking `on_sample` for each completed
/// sample. Ported from the former `monitor::stream_metrics`; the only change is
/// the per-sample callback in place of channel sends.
///
/// Powermetrics outputs plist messages; we fix them up and parse, then merge in
/// the sysinfo CPU/memory data (more accurate per-core usage on M2).
fn stream<F>(tick_rate: Duration, mut on_sample: F) -> Result<()>
where
    F: FnMut(&Metrics) -> ControlFlow<()>,
{
    let sample_rate_ms = format!("{}", tick_rate.as_millis());

    let binary = "/usr/bin/powermetrics";
    let args = vec![
        "--sample-rate",
        sample_rate_ms.as_str(),
        "--samplers",
        "cpu_power,gpu_power,thermal",
        "-f",
        "plist",
    ];

    let mut cmd = process::Command::new(binary)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(CrateError::PowermetricsSpawn)?;

    let stdout = cmd.stdout.as_mut().ok_or(CrateError::PowermetricsStdout)?;
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    let mut buffer = powermetrics::Buffer::new();
    let mut system_state = sysinfo::SystemState::new();

    for line in stdout_lines.map_while(std::result::Result::<String, std::io::Error>::ok) {
        if line != "</plist>" {
            buffer.append_line(line);
        } else {
            buffer.append_last_line(line);
            let text = buffer.finalize();

            let power_metrics = match Metrics::from_bytes(text.as_bytes()) {
                Ok(metrics) => metrics,
                Err(err) => {
                    eprintln!("{err}");
                    cmd.kill().map_err(CrateError::PowermetricsKill)?;
                    break;
                }
            };

            let sysinfo_metrics = system_state.latest_metrics();

            let metrics = match power_metrics.merge_sysinfo_metrics(sysinfo_metrics) {
                Ok(metrics) => metrics,
                Err(err) => {
                    eprintln!("{err}");
                    cmd.kill().map_err(CrateError::PowermetricsKill)?;
                    break;
                }
            };

            if on_sample(&metrics).is_break() {
                cmd.kill().map_err(CrateError::PowermetricsKill)?;
                break;
            }
        }
    }

    let status = cmd.wait()?;
    if !status.success() && status.code().is_some() {
        let mut err_msg = String::new();
        if let Some(mut stderr) = cmd.stderr.take() {
            stderr.read_to_string(&mut err_msg).ok();
        }
        return Err(CrateError::PowermetricsNonZeroExit(
            status,
            err_msg.trim().to_string(),
        ));
    }

    Ok(())
}

// ─── History (ported from app::update_history) ──────────────────────────────

/// Push the current sample into every signal, creating signals on first sight.
/// Power signals scale to the SoC ceilings; memory to the reported totals; all
/// others to 100%. Ported verbatim from `app::update_history`.
fn update_history(history: &mut History, soc: &SocInfo, history_size: usize, metrics: &Metrics) {
    use history::Signal;

    // Active ratios.
    for (idx, e_cluster) in metrics.e_clusters.iter().enumerate() {
        let key = MetricKey::ClusterActivePercent(ClusterId::efficiency(idx as u8));
        history
            .entry(key)
            .or_insert(Signal::with_capacity(history_size, 100.0))
            .push(100.0 * e_cluster.active_ratio());

        for cpu in &e_cluster.cpus {
            push_cpu(history, history_size, cpu);
        }
    }

    for (idx, p_cluster) in metrics.p_clusters.iter().enumerate() {
        let key = MetricKey::ClusterActivePercent(ClusterId::performance(idx as u8));
        history
            .entry(key)
            .or_insert(Signal::with_capacity(history_size, 100.0))
            .push(100.0 * p_cluster.active_ratio());

        for cpu in &p_cluster.cpus {
            push_cpu(history, history_size, cpu);
        }
    }

    for (idx, s_cluster) in metrics.s_clusters.iter().enumerate() {
        let key = MetricKey::ClusterActivePercent(ClusterId::super_core(idx as u8));
        history
            .entry(key)
            .or_insert(Signal::with_capacity(history_size, 100.0))
            .push(100.0 * s_cluster.active_ratio());

        for cpu in &s_cluster.cpus {
            push_cpu(history, history_size, cpu);
        }
    }

    history
        .entry(MetricKey::GpuActivePercent)
        .or_insert(Signal::with_capacity(history_size, 100.0))
        .push(100.0 * metrics.gpu.active_ratio as f32);

    history
        .entry(MetricKey::GpuFreqPercent)
        .or_insert(Signal::with_capacity(history_size, 100.0))
        .push(100.0 * metrics.gpu.freq_ratio() as f32);

    history
        .entry(MetricKey::AneActivePercent)
        .or_insert(Signal::with_capacity(history_size, 100.0))
        .push(100.0 * metrics.consumption.ane_w / soc.max_ane_w as f32);

    // Power consumption.
    history
        .entry(MetricKey::CpuPowerW)
        .or_insert(Signal::with_capacity(history_size, soc.max_cpu_w as f32))
        .push(metrics.consumption.cpu_w);

    history
        .entry(MetricKey::GpuPowerW)
        .or_insert(Signal::with_capacity(history_size, soc.max_gpu_w as f32))
        .push(metrics.consumption.gpu_w);

    history
        .entry(MetricKey::AnePowerW)
        .or_insert(Signal::with_capacity(history_size, soc.max_ane_w as f32))
        .push(metrics.consumption.ane_w);

    history
        .entry(MetricKey::PackagePowerW)
        .or_insert(Signal::with_capacity(
            history_size,
            soc.max_package_w as f32,
        ))
        .push(metrics.consumption.package_w);

    // Memory usage.
    history
        .entry(MetricKey::RamUsageBytes)
        .or_insert(Signal::with_capacity(
            history_size,
            metrics.memory.ram_total as f32,
        ))
        .push(metrics.memory.ram_used as f32);

    history
        .entry(MetricKey::SwapUsageBytes)
        .or_insert(Signal::with_capacity(
            history_size,
            metrics.memory.swap_total as f32,
        ))
        .push(metrics.memory.swap_used as f32);
}

/// Push a single CPU core's activity + frequency ratios into the history.
fn push_cpu(history: &mut History, history_size: usize, cpu: &CpuMetrics) {
    use history::Signal;

    history
        .entry(MetricKey::CpuActivePercent(cpu.id))
        .or_insert(Signal::with_capacity(history_size, 100.0))
        .push(100.0 * cpu.active_ratio as f32);

    history
        .entry(MetricKey::CpuFreqPercent(cpu.id))
        .or_insert(Signal::with_capacity(history_size, 100.0))
        .push(100.0 * cpu.freq_ratio() as f32);
}

// ─── Frame builder ──────────────────────────────────────────────────────────

/// Build an owned [`Frame`] from the current `Metrics` + signal history. Every
/// title/label string is formatted here (reusing [`units`]); the frontend does
/// no formatting.
fn build_frame(metrics: &Metrics, soc: &SocInfo, history: &History) -> Frame {
    Frame {
        overview: build_overview(metrics, soc, history),
        cpu: build_cpu(metrics, history),
        gpu: build_gpu(metrics, history),
        memory: build_memory(metrics),
    }
}

fn build_overview(metrics: &Metrics, soc: &SocInfo, history: &History) -> OverviewFrame {
    let cpu_pow = history.get_or_default(&MetricKey::CpuPowerW);
    let cpu_clusters_title = format!(
        " CPU Clusters: {} (peak: {}) ",
        units::watts2(metrics.consumption.cpu_w),
        units::watts2(cpu_pow.peak)
    );

    let e_meters = metrics
        .e_clusters
        .iter()
        .enumerate()
        .map(|(idx, c)| cluster_meter(c, ClusterId::efficiency(idx as u8), history))
        .collect();
    let p_meters = metrics
        .p_clusters
        .iter()
        .enumerate()
        .map(|(idx, c)| cluster_meter(c, ClusterId::performance(idx as u8), history))
        .collect();
    let s_meters = metrics
        .s_clusters
        .iter()
        .enumerate()
        .map(|(idx, c)| cluster_meter(c, ClusterId::super_core(idx as u8), history))
        .collect();

    // GPU.
    let gpu = &metrics.gpu;
    let gpu_act = history.get_or_default(&MetricKey::GpuActivePercent);
    let gpu_pow = history.get_or_default(&MetricKey::GpuPowerW);
    let gpu_meter = Meter {
        title: format!(
            "GPU: {} @ {} | {} (peak: {} | {})",
            units::percent1(gpu.active_ratio * 100.0),
            units::mhz(gpu.freq_mhz),
            units::watts2(metrics.consumption.gpu_w),
            units::percent1(gpu_act.peak),
            units::watts2(gpu_pow.peak)
        ),
        ratio: gpu.active_ratio,
        spark: gpu_act.as_slice().to_vec(),
        spark_max: (SPARKLINE_MAX_OVERSHOOT * gpu_act.max) as u64,
    };

    // ANE.
    let ane_ratio = metrics.consumption.ane_w as f64 / soc.max_ane_w;
    let ane_act = history.get_or_default(&MetricKey::AneActivePercent);
    let ane_pow = history.get_or_default(&MetricKey::AnePowerW);
    let ane_meter = Meter {
        title: format!(
            "ANE: {} | {} (peak: {} | {})",
            units::percent1(ane_ratio * 100.0),
            units::watts2(metrics.consumption.ane_w),
            units::percent1(ane_act.peak),
            units::watts2(ane_pow.peak)
        ),
        ratio: ane_ratio,
        spark: ane_act.as_slice().to_vec(),
        spark_max: (SPARKLINE_MAX_OVERSHOOT * ane_act.max) as u64,
    };

    // Package (no overshoot).
    let pkg = history.get_or_default(&MetricKey::PackagePowerW);
    let package = SparkText {
        title: format!(
            "CPU+GPU+ANE: {} (peak: {})",
            units::watts2(metrics.consumption.package_w),
            units::watts2(pkg.peak)
        ),
        spark: pkg.as_slice().to_vec(),
        spark_max: pkg.max as u64,
    };

    let thermals = build_thermals(metrics);

    // Memory & swap.
    let mem = &metrics.memory;
    let ram_sig = history.get_or_default(&MetricKey::RamUsageBytes);
    let ram_ratio = mem.ram_usage_ratio();
    let ram = Meter {
        title: format!(
            "Memory Used: {} = {} / {} (peak: {} = {})",
            units::percent1(ram_ratio * 100.0),
            units::bibytes1(mem.ram_used as f64),
            units::bibytes1(mem.ram_total as f64),
            units::percent1(ram_sig.peak / mem.ram_total as f32 * 100.0),
            units::bibytes1(ram_sig.peak),
        ),
        ratio: ram_ratio,
        spark: ram_sig.as_slice().to_vec(),
        spark_max: (SPARKLINE_MAX_OVERSHOOT * ram_sig.max) as u64,
    };

    let swap_sig = history.get_or_default(&MetricKey::SwapUsageBytes);
    let swap_ratio = mem.swap_usage_ratio();
    let swap = Meter {
        title: format!(
            "SWAP: {} = {} / {} (peak: {})",
            units::percent1(swap_ratio * 100.0),
            units::bibytes1(mem.swap_used as f64),
            units::bibytes1(mem.swap_total as f64),
            units::bibytes1(swap_sig.peak),
        ),
        ratio: swap_ratio,
        spark: swap_sig.as_slice().to_vec(),
        spark_max: (SPARKLINE_MAX_OVERSHOOT * swap_sig.max) as u64,
    };

    OverviewFrame {
        cpu_clusters_title,
        e_meters,
        p_meters,
        s_meters,
        gpu: gpu_meter,
        ane: ane_meter,
        package,
        thermals,
        ram,
        swap,
    }
}

/// Build an Overview cluster meter (full-history sparkline, 1.05 overshoot).
fn cluster_meter(cluster: &ClusterMetrics, id: ClusterId, history: &History) -> Meter {
    let sig = history.get_or_default(&MetricKey::ClusterActivePercent(id));
    Meter {
        title: format!(
            "{}: {} @ {} (peak: {})",
            cluster.name,
            units::percent1(cluster.active_ratio() * 100.0),
            units::mhz(cluster.freq_mhz),
            units::percent1(sig.peak)
        ),
        ratio: cluster.active_ratio() as f64,
        spark: sig.as_slice().to_vec(),
        spark_max: (SPARKLINE_MAX_OVERSHOOT * sig.max) as u64,
    }
}

fn build_thermals(metrics: &Metrics) -> Thermals {
    Thermals {
        pressure: metrics.thermal_pressure.clone(),
        is_nominal: metrics.thermal_pressure == "Nominal",
    }
}

fn build_cpu(metrics: &Metrics, history: &History) -> CpuFrame {
    let clusters = metrics
        .e_clusters
        .iter()
        .chain(metrics.p_clusters.iter())
        .chain(metrics.s_clusters.iter())
        .map(|c| cpu_cluster(c, history))
        .collect();

    CpuFrame {
        clusters,
        freq_table: cpu_freq_table(metrics),
    }
}

fn cpu_cluster(cluster: &ClusterMetrics, history: &History) -> CpuCluster {
    CpuCluster {
        title: format!(" {}: ", cluster.name),
        cpus: cluster.cpus.iter().map(|c| cpu_row(c, history)).collect(),
    }
}

fn cpu_row(cpu: &CpuMetrics, history: &History) -> CpuRow {
    let act = history.get_or_default(&MetricKey::CpuActivePercent(cpu.id));
    let freq = history.get_or_default(&MetricKey::CpuFreqPercent(cpu.id));
    CpuRow {
        id_label: format!("{:2} -", cpu.id),
        act_ratio: cpu.active_ratio,
        act_label: format!("{:.1}%", cpu.active_ratio * 100.0),
        act_spark: act.as_slice_last_n(HISTORY_LENGTH).to_vec(),
        act_spark_max: (SPARKLINE_MAX_OVERSHOOT * act.max) as u64,
        freq_value: units::mhz(cpu.freq_mhz),
        freq_ratio: cpu.freq_ratio(),
        freq_label: format!("{:3.0}%", cpu.freq_ratio() * 100.0),
        freq_spark: freq.as_slice_last_n(HISTORY_LENGTH).to_vec(),
        freq_spark_max: (SPARKLINE_MAX_OVERSHOOT * freq.max) as u64,
    }
}

/// CPU DVFM frequency table.
fn cpu_freq_table(metrics: &Metrics) -> FreqTable {
    let e = first_cluster_freqs(&metrics.e_clusters);
    let p = first_cluster_freqs(&metrics.p_clusters);
    let s = first_cluster_freqs(&metrics.s_clusters);

    let mut rows: Vec<(String, String)> = Vec::new();
    if !e.is_empty() {
        rows.push(("E-Cluster:".into(), e));
    }
    if !p.is_empty() {
        rows.push(("P-Cluster:".into(), p));
    }
    if !s.is_empty() {
        rows.push(("S-Cluster:".into(), s));
    }
    rows.push((String::new(), String::new()));
    rows.push((
        "Note:".into(),
        "Hardware-wise, CPUs quickly shift between the above frequencies.".into(),
    ));
    FreqTable { rows }
}

/// Space-joined `{:4}` DVFM frequencies of the first cluster's first CPU.
fn first_cluster_freqs(clusters: &[ClusterMetrics]) -> String {
    clusters
        .first()
        .and_then(|c| c.cpus.first())
        .map(|c| c.frequencies_mhz())
        .unwrap_or_default()
        .iter()
        .map(|f| format!("{f:4}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_gpu(metrics: &Metrics, history: &History) -> GpuFrame {
    let gpu = &metrics.gpu;
    let act = history.get_or_default(&MetricKey::GpuActivePercent);
    let freq = history.get_or_default(&MetricKey::GpuFreqPercent);
    let pow = history.get_or_default(&MetricKey::GpuPowerW);

    let gpu_freqs = gpu
        .frequencies_mhz()
        .iter()
        .map(|f| format!("{f:4}"))
        .collect::<Vec<_>>()
        .join(" ");
    let freq_table = FreqTable {
        rows: vec![
            ("GPU:".into(), gpu_freqs),
            (String::new(), String::new()),
            (
                "Note:".into(),
                "Hardware-wise, GPUs quickly shift between the above frequencies.".into(),
            ),
        ],
    };

    GpuFrame {
        act_ratio: gpu.active_ratio,
        act_label: format!("{:.1}%", gpu.active_ratio * 100.0),
        act_spark: act.as_slice_last_n(HISTORY_LENGTH).to_vec(),
        act_spark_max: (SPARKLINE_MAX_OVERSHOOT * act.max) as u64,
        freq_value: units::mhz(gpu.freq_mhz),
        freq_ratio: gpu.freq_ratio(),
        freq_label: format!("{:3.0}%", gpu.freq_ratio() * 100.0),
        freq_spark: freq.as_slice_last_n(HISTORY_LENGTH).to_vec(),
        freq_spark_max: (SPARKLINE_MAX_OVERSHOOT * freq.max) as u64,
        power_value: units::watts2(metrics.consumption.gpu_w),
        power_spark: pow.as_slice_last_n(HISTORY_LENGTH).to_vec(),
        power_spark_max: (SPARKLINE_MAX_OVERSHOOT * pow.max) as u64,
        peak_text: format!(
            "Peak: {} | {}",
            units::percent1(act.peak),
            units::watts2(pow.peak)
        ),
        thermals: build_thermals(metrics),
        freq_table,
    }
}

/// Build the Memory tab lines. `vm_stat` is collected here, not on the UI
/// thread.
fn build_memory(metrics: &Metrics) -> MemoryFrame {
    let vm_lines = match VmStats::collect() {
        Ok(vm) => {
            let page_to_gb =
                |pages: u64| (pages * vm.page_size) as f64 / (1024.0 * 1024.0 * 1024.0);
            let total_gb = vm.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = vm.activity_monitor_memory_used() as f64 / (1024.0 * 1024.0 * 1024.0);

            vec![
                line(vec![
                    span("Physical Memory Total: ", Accent),
                    span(format!("{total_gb:.2} GB"), Def),
                ]),
                blank(),
                line(vec![span("═══ ACTIVITY MONITOR CALCULATION ═══", Accent)]),
                line(vec![
                    span("App Memory (Anonymous): ", GaugeFg),
                    span(format!("{:.2} GB", page_to_gb(vm.pages_anonymous)), Def),
                ]),
                line(vec![
                    span("Wired Memory:         + ", GaugeFg),
                    span(format!("{:.2} GB", page_to_gb(vm.pages_wired)), Def),
                ]),
                line(vec![
                    span("Compressed:           + ", GaugeFg),
                    span(format!("{:.2} GB", page_to_gb(vm.pages_compressed)), Def),
                ]),
                line(vec![span("                      ─────────", HistoryFg)]),
                line(vec![
                    span("Memory Used Total:      ", Accent),
                    span(format!("{used_gb:.2} GB"), Def),
                ]),
                blank(),
                line(vec![span("═══ OTHER MEMORY CATEGORIES ═══", HistoryFg)]),
                line(vec![
                    span("Cached Files:         ", GaugeFg),
                    span(format!("{:.2} GB", page_to_gb(vm.pages_file_backed)), Def),
                ]),
                line(vec![
                    span("Free:                 ", GaugeFg),
                    span(format!("{:.2} GB", page_to_gb(vm.pages_free)), Def),
                ]),
                line(vec![
                    span("Active:               ", GaugeFg),
                    span(format!("{:.2} GB", page_to_gb(vm.pages_active)), Def),
                ]),
                line(vec![
                    span("Inactive:             ", GaugeFg),
                    span(format!("{:.2} GB", page_to_gb(vm.pages_inactive)), Def),
                ]),
            ]
        }
        Err(_) => vec![
            line(vec![span("Failed to collect VM statistics", Def)]),
            line(vec![span("vm_stat command may not be available", Def)]),
        ],
    };

    let mem = &metrics.memory;
    let sysinfo_lines = vec![
        line(vec![
            span("RAM Used: ", Accent),
            span(
                format!(
                    "{} = {} / {} ({:.1}%)",
                    units::percent1(mem.ram_usage_ratio() * 100.0),
                    units::bibytes1(mem.ram_used as f64),
                    units::bibytes1(mem.ram_total as f64),
                    mem.ram_usage_ratio() * 100.0
                ),
                Def,
            ),
        ]),
        line(vec![
            span("Swap Used: ", Accent),
            span(
                format!(
                    "{} = {} / {}",
                    units::percent1(mem.swap_usage_ratio() * 100.0),
                    units::bibytes1(mem.swap_used as f64),
                    units::bibytes1(mem.swap_total as f64)
                ),
                Def,
            ),
        ]),
        blank(),
        line(vec![
            span("Note: ", HistoryFg),
            span(
                "RAM Used now uses vm_stat for Activity Monitor compatibility",
                Def,
            ),
        ]),
    ];

    MemoryFrame {
        vm_lines,
        sysinfo_lines,
    }
}

fn span(text: impl Into<String>, role: frame::ColorRole) -> MemSpan {
    MemSpan {
        text: text.into(),
        role,
    }
}

fn line(spans: Vec<MemSpan>) -> MemLine {
    MemLine { spans }
}

fn blank() -> MemLine {
    MemLine { spans: Vec::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic SoC for deterministic Frame/JSON tests (no live `sysctl`).
    fn test_soc() -> SocInfo {
        SocInfo {
            cpu_brand_name: "Apple M1".into(),
            num_cpu_cores: 8,
            num_efficiency_cores: 4,
            num_performance_cores: 4,
            num_gpu_cores: 8,
            max_cpu_w: 20.0,
            max_gpu_w: 20.0,
            max_ane_w: 8.0,
            max_package_w: 48.0,
        }
    }

    fn m1_metrics() -> Metrics {
        let content = std::fs::read_to_string("./tests/data/powermetrics-output-m1.xml")
            .expect("read m1 fixture");
        Metrics::from_bytes(content.as_bytes()).expect("parse m1 fixture")
    }

    /// The Frame builder formats the expected title strings and applies the
    /// Package "no overshoot" exception.
    #[test]
    fn frame_titles_and_package_spark_max_rule() {
        let soc = test_soc();
        let metrics = m1_metrics();
        let mut history: History = HashMap::new();
        update_history(&mut history, &soc, 128, &metrics);

        let frame = build_frame(&metrics, &soc, &history);

        // Border title for the CPU Clusters panel.
        assert!(
            frame
                .overview
                .cpu_clusters_title
                .starts_with(" CPU Clusters: "),
            "got: {:?}",
            frame.overview.cpu_clusters_title
        );

        // First E-cluster meter title is the formatted cluster line.
        let e0 = &frame.overview.e_meters[0];
        assert!(e0.title.starts_with("E-Cluster: "), "got: {:?}", e0.title);
        // Cluster signals scale to 100% with the 1.05 overshoot applied.
        assert_eq!(e0.spark_max, (SPARKLINE_MAX_OVERSHOOT * 100.0_f32) as u64);
        assert!(
            e0.spark_max > 100,
            "overshoot must lift the ceiling above max"
        );

        // Package uses NO overshoot: spark_max == signal.max == max_package_w.
        assert_eq!(frame.overview.package.spark_max, soc.max_package_w as u64);
        // And that is strictly below the overshoot value every other meter uses,
        // proving the Package exception bites.
        assert!(
            frame.overview.package.spark_max
                < (SPARKLINE_MAX_OVERSHOOT * soc.max_package_w as f32) as u64
        );

        // The freq line-gauge default label is built: MHz value + "{:3.0}%".
        let cpu0 = &frame.cpu.clusters[0].cpus[0];
        assert!(
            cpu0.freq_value.ends_with("MHz"),
            "got: {:?}",
            cpu0.freq_value
        );
        assert_eq!(cpu0.freq_label, format!("{:3.0}%", cpu0.freq_ratio * 100.0));
        assert_eq!(
            frame.gpu.freq_label,
            format!("{:3.0}%", frame.gpu.freq_ratio * 100.0)
        );
    }

    /// `run --json` output is byte-identical to a committed golden line. Guards
    /// the JSON serialization (field set/format) against drift. Uses the raw
    /// powermetrics metrics (no live sysinfo merge) for determinism.
    #[test]
    fn json_export_line_matches_golden() {
        let soc = test_soc();
        let metrics = m1_metrics();
        let actual = export_line(&soc, &metrics);

        let path = format!(
            "{}/tests/snapshots/json_export_m1.golden",
            env!("CARGO_MANIFEST_DIR")
        );
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(&path, &actual).expect("write golden");
            return;
        }
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read golden {path}: {e} (run with UPDATE_SNAPSHOTS=1)"));
        assert_eq!(actual, expected.trim_end_matches('\n'), "JSON export drift");
    }
}
