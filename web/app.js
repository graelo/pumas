// Pumas Dashboard Application

const MAX_HISTORY = 60;  // Keep 60 samples (~2 min at 2s interval)

// History buffers keyed by series identifier
const history = {};

function initHistory(id) {
    if (!history[id]) history[id] = { time: [], values: [] };
    return history[id];
}

function pushHistory(id, time, value) {
    const h = initHistory(id);
    h.time.push(time);
    h.values.push(value);
    if (h.time.length > MAX_HISTORY) {
        h.time.shift();
        h.values.shift();
    }
}

// ── Theme ──────────────────────────────────────────────────────────────

function getSystemTheme() {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function resolveTheme(pref) {
    if (pref === 'auto') return getSystemTheme();
    return pref;
}

let currentTheme = resolveTheme(DASHBOARD_CONFIG.defaultTheme);

function applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    document.getElementById('themeBtn').textContent = theme === 'dark' ? '☀️' : '🌙';
    localStorage.setItem('pumas-theme', theme);
    currentTheme = theme;
}

function toggleTheme() {
    const next = currentTheme === 'dark' ? 'light' : 'dark';
    applyTheme(next);
}

// Listen for system theme changes when in auto mode
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (DASHBOARD_CONFIG.defaultTheme === 'auto') {
        applyTheme(getSystemTheme());
    }
});

// ── Adaptive frequency formatter ───────────────────────────────────────

function formatFreq(v) {
    return v >= 1000 ? (v / 1000).toFixed(2) + ' GHz' : v.toFixed(0) + ' MHz';
}

// ── ECharts helpers ────────────────────────────────────────────────────

const charts = {};

function createChart(id, height) {
    const dom = document.getElementById(id);
    if (!dom) return null;
    dom.style.height = height + 'px';
    const chart = echarts.init(dom, currentTheme === 'dark' ? 'dark' : null);
    charts[id] = chart;
    return chart;
}

function resizeAll() {
    Object.values(charts).forEach(c => c && c.resize());
}

// ── Render functions ───────────────────────────────────────────────────

function renderCpuUtil(metrics) {
    const chart = charts['chart-cpuUtil'];
    if (!chart) return;

    const clusters = [
        ...metrics.e_clusters.map(c => ({ ...c, kind: 'E' })),
        ...metrics.p_clusters.map(c => ({ ...c, kind: 'P' })),
        ...metrics.s_clusters.map(c => ({ ...c, kind: 'S' })),
    ];

    const now = Date.now();
    const series = [];

    for (const cluster of clusters) {
        const id = cluster.name;
        const avg = cluster.cpus.reduce((s, c) => s + c.active_ratio, 0) / cluster.cpus.length;
        const pct = Math.max(0.1, avg * 100).toFixed(1);
        pushHistory('util-' + id, now, pct);

        const h = history['util-' + id];
        series.push({
            name: cluster.name,
            type: 'line',
            smooth: true,
            data: h.values.map((v, i) => [h.time[i], v]),
            symbol: 'none',
            lineStyle: { width: 2 },
        });
    }

    chart.setOption({
        tooltip: { trigger: 'axis', valueFormatter: v => v + '%' },
        legend: { data: clusters.map(c => c.name), bottom: 0 },
        grid: { left: 45, right: 10, top: 30, bottom: 40 },
        xAxis: { type: 'time', show: false },
        yAxis: { type: 'log', min: 0.1, axisLabel: { formatter: '{value}%' } },
        series,
    }, true);
}

function renderCpuFreq(metrics) {
    const chart = charts['chart-cpuFreq'];
    if (!chart) return;

    const clusters = [
        ...metrics.e_clusters.map(c => ({ ...c, kind: 'E' })),
        ...metrics.p_clusters.map(c => ({ ...c, kind: 'P' })),
        ...metrics.s_clusters.map(c => ({ ...c, kind: 'S' })),
    ];

    const now = Date.now();
    const series = [];

    for (const cluster of clusters) {
        const id = cluster.name;
        pushHistory('freq-' + id, now, cluster.freq_mhz.toFixed(0));

        const h = history['freq-' + id];
        series.push({
            name: cluster.name,
            type: 'line',
            smooth: true,
            data: h.values.map((v, i) => [h.time[i], v]),
            symbol: 'none',
            lineStyle: { width: 2 },
        });
    }

    chart.setOption({
        tooltip: { trigger: 'axis', valueFormatter: v => formatFreq(v) },
        legend: { data: clusters.map(c => c.name), bottom: 0 },
        grid: { left: 60, right: 10, top: 30, bottom: 40 },
        xAxis: { type: 'time', show: false },
        yAxis: { type: 'value', axisLabel: { formatter: v => formatFreq(v) } },
        series,
    }, true);
}

function renderGpu(metrics) {
    const chart = charts['chart-gpu'];
    if (!chart) return;

    const gpu = metrics.gpu;
    const now = Date.now();

    pushHistory('gpu-util', now, Math.max(0.1, gpu.active_ratio * 100).toFixed(1));
    pushHistory('gpu-freq', now, gpu.freq_mhz.toFixed(0));

    const utilH = history['gpu-util'];
    const freqH = history['gpu-freq'];

    chart.setOption({
        tooltip: { trigger: 'axis' },
        legend: { data: ['Utilization', 'Frequency'], bottom: 0 },
        grid: { left: 45, right: 55, top: 30, bottom: 40 },
        xAxis: { type: 'time', show: false },
        yAxis: [
            { type: 'log', min: 0.1, axisLabel: { formatter: '{value}%' } },
            { type: 'value', axisLabel: { formatter: v => formatFreq(v) }, splitLine: { show: false } },
        ],
        series: [
            {
                name: 'Utilization',
                type: 'line',
                smooth: true,
                data: utilH.values.map((v, i) => [utilH.time[i], v]),
                symbol: 'none',
                lineStyle: { width: 2, color: '#42a5f5' },
                itemStyle: { color: '#42a5f5' },
            },
            {
                name: 'Frequency',
                type: 'line',
                smooth: true,
                yAxisIndex: 1,
                data: freqH.values.map((v, i) => [freqH.time[i], v]),
                symbol: 'none',
                lineStyle: { width: 2, color: '#ff7043' },
                itemStyle: { color: '#ff7043' },
            },
        ],
    }, true);
}

function renderPower(metrics) {
    const chart = charts['chart-power'];
    if (!chart) return;

    const { cpu_w, gpu_w, ane_w, package_w } = metrics.consumption;
    const now = Date.now();

    pushHistory('power-cpu', now, cpu_w);
    pushHistory('power-gpu', now, gpu_w);
    pushHistory('power-ane', now, ane_w);
    pushHistory('power-pkg', now, package_w);

    const series = ['cpu', 'gpu', 'ane'].map(k => {
        const h = history['power-' + k];
        return {
            name: k.toUpperCase(),
            type: 'line',
            stack: 'total',
            areaStyle: {},
            smooth: true,
            symbol: 'none',
            data: h.values.map((v, i) => [h.time[i], v]),
            lineStyle: { width: 1 },
        };
    });

    const pkgH = history['power-pkg'];
    series.push({
        name: 'Package',
        type: 'line',
        smooth: true,
        symbol: 'none',
        lineStyle: { width: 2, type: 'dashed' },
        data: pkgH.values.map((v, i) => [pkgH.time[i], v]),
    });

    chart.setOption({
        tooltip: { trigger: 'axis', valueFormatter: v => v + ' W' },
        legend: { data: ['CPU', 'GPU', 'ANE', 'Package'], bottom: 0 },
        grid: { left: 40, right: 10, top: 30, bottom: 40 },
        xAxis: { type: 'time', show: false },
        yAxis: { type: 'value', axisLabel: { formatter: '{value} W' } },
        series,
    }, true);
}

function renderMemory(metrics) {
    const mem = metrics.memory;
    const ramPct = mem.ram_total > 0 ? ((mem.ram_used / mem.ram_total) * 100).toFixed(1) : 0;
    const swapPct = mem.swap_total > 0 ? ((mem.swap_used / mem.swap_total) * 100).toFixed(1) : 0;

    const fmt = v => {
        if (v >= 1e12) return (v / 1e12).toFixed(1) + ' TB';
        if (v >= 1e9) return (v / 1e9).toFixed(1) + ' GB';
        if (v >= 1e6) return (v / 1e6).toFixed(1) + ' MB';
        return v + ' B';
    };

    document.getElementById('mem-ram-text').textContent =
        `${fmt(mem.ram_used)} / ${fmt(mem.ram_total)} (${ramPct}%)`;
    document.getElementById('mem-ram-fill').style.width = ramPct + '%';

    document.getElementById('mem-swap-text').textContent =
        `${fmt(mem.swap_used)} / ${fmt(mem.swap_total)} (${swapPct}%)`;
    document.getElementById('mem-swap-fill').style.width = swapPct + '%';
}

function renderThermal(metrics) {
    const level = metrics.thermal_pressure.toLowerCase();
    const el = document.getElementById('thermal-label');
    el.textContent = metrics.thermal_pressure;

    // Remove all color classes
    el.className = 'thermal-label ' + level;

    // Map level to fill width
    const levels = { nominal: 10, light: 30, moderate: 50, heavy: 75, critical: 100 };
    const fill = document.getElementById('thermal-fill');
    const colors = { nominal: '#4caf50', light: '#8bc34a', moderate: '#ffc107', heavy: '#ff9800', critical: '#f44336' };
    fill.style.width = (levels[level] || 10) + '%';
    fill.style.background = colors[level] || '#4caf50';
}

// ── Status indicator ───────────────────────────────────────────────────

function updateStatus(connected) {
    const dot = document.getElementById('statusDot');
    const text = document.getElementById('statusText');
    if (connected) {
        dot.className = 'status-dot active';
        text.textContent = 'active';
    } else {
        dot.className = 'status-dot starting';
        text.textContent = 'connecting...';
    }
}

// ── Main poll loop ─────────────────────────────────────────────────────

async function pollMetrics() {
    try {
        const resp = await fetch('/api/metrics');
        if (!resp.ok) throw new Error('HTTP ' + resp.status);
        const data = await resp.json();
        updateStatus(true);

        // Update SoC info header on first load
        const socInfo = document.getElementById('socInfo');
        if (socInfo && data.soc) {
            socInfo.textContent = `${data.soc.cpu_brand_name} · ${data.soc.num_cpu_cores}C${data.soc.num_gpu_cores}G`;
        }

        renderCpuUtil(data.metrics);
        renderCpuFreq(data.metrics);
        renderGpu(data.metrics);
        renderPower(data.metrics);
        renderMemory(data.metrics);
        renderThermal(data.metrics);

    } catch (err) {
        updateStatus(false);
        console.warn('poll failed:', err);
    }
}

// ── Init ───────────────────────────────────────────────────────────────

document.addEventListener('DOMContentLoaded', () => {
    // Restore saved theme or use default
    const saved = localStorage.getItem('pumas-theme');
    if (saved) {
        applyTheme(saved);
    } else {
        applyTheme(resolveTheme(DASHBOARD_CONFIG.defaultTheme));
    }

    // Apply config-driven layout to cards
    const cfg = DASHBOARD_CONFIG.charts;
    for (const [key, chart] of Object.entries(cfg)) {
        const card = document.getElementById('card-' + key);
        if (!card) continue;
        if (!chart.show) {
            card.style.display = 'none';
            continue;
        }
        card.style.gridRow = (chart.row + 1).toString();
        card.style.gridColumn = `${chart.col + 1} / span ${chart.width}`;
        // Set title from config
        const titleEl = document.getElementById('title-' + key);
        if (titleEl) titleEl.textContent = chart.title;

        // Create chart or set height for non-chart cards
        // Non-chart cards: thermal gets fixed height, memory auto-sizes to content
        if (key === 'thermal') {
            const el = document.getElementById('chart-' + key);
            if (el) el.style.height = chart.height + 'px';
        } else if (key === 'memory') {
            // auto-height — no explicit size needed
        } else {
            createChart('chart-' + key, chart.height);
        }
    }

    // Resize charts on window resize
    window.addEventListener('resize', resizeAll);

    // Start polling
    pollMetrics();
    setInterval(pollMetrics, DASHBOARD_CONFIG.refreshInterval);
});
