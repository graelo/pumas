# Web Dashboard

Web-based dashboard for Pumas using **axum** (Rust HTTP framework) and **ECharts** (JavaScript charting library).

## Plan

### Motivation

Pumas already provides a TUI (`pumas run`) and JSON export (`pumas run --json`). A web dashboard makes the same data accessible from any browser, with richer visualizations (ECharts) and an easily customizable layout.

### Design Goals

- Serve a browser-based dashboard from the same `pumas` binary (no separate frontend build step)
- Frontend assets embedded in the binary via `rust-embed` — zero external files at runtime
- User-editable `config.js` for chart layout, colors, refresh rate
- Light/dark theme with automatic system preference detection
- `--web-dir` flag to override embedded assets with a local directory for customization without rebuilding

### Implementation Order

1. Add dependencies: `axum`, `tokio`, `tower-http`, `rust-embed` to `Cargo.toml`
2. Add `WebConfig` struct and `Command::Web` variant to `src/config.rs`
3. Create frontend: `web/index.html`, `web/app.js`, `web/config.js`, `web/style.css`
4. Create backend: `src/web.rs` — axum HTTP server with routes
5. Add `run_web()` and metrics streaming to `src/monitor.rs`
6. Wire up: register module in `src/lib.rs`, add dispatch in `src/bin/pumas.rs`

## Architecture

```
powermetrics ──std thread──► Arc<RwLock<Option<Metrics>>>
                                       │
                             axum HTTP server
                            ┌─────┼─────────┐
                            ▼     ▼         ▼
                       GET /  GET /api/   GET /assets/*
                     index.html  metrics    (static files)
```

### Data Flow

1. **Metrics Collection** runs in a standard thread (powermetrics requires synchronous I/O). It spawns `/usr/bin/powermetrics`, parses the plist output, merges with sysinfo CPU data, and writes the result to `Arc<RwLock<Option<Metrics>>>`.
2. **axum Server** runs on a separate tokio runtime. HTTP handlers read from the same shared state on each request.
3. **Frontend** polls `/api/metrics` every `refreshInterval` ms and updates ECharts charts in-place.

### Key Components

| Layer | File | Role |
| --- | --- | --- |
| Frontend | `web/index.html` | Dashboard HTML with ECharts CDN, chart containers, memory/thermal DOM |
| Frontend | `web/app.js` | Polling loop, ECharts instances, history buffers (60 samples), theme toggle |
| Frontend | `web/config.js` | User-editable layout configuration |
| Frontend | `web/style.css` | CSS custom properties for light/dark themes |
| Backend | `src/web.rs` | axum HTTP server with 4 routes |
| Backend | `src/monitor.rs` | `run_web()` entry point, `stream_powermetrics_to_state()` |
| Config | `src/config.rs` | `WebConfig` struct + `Command::Web` variant |

### Dependencies Added

- `axum = "0.7"` — async HTTP framework
- `tokio = { features = ["rt-multi-thread", "macros", "sync", "fs"] }` — async runtime + file I/O
- `tower-http = { features = ["fs"] }` — (added but ServeDir not used; kept for future use)
- `rust-embed = "8"` — embed `web/` directory in the binary

## Implementation

### Usage

```sh
sudo pumas web
```

Opens at `http://127.0.0.1:9810`.

#### CLI Options

| Flag | Default | Description |
| --- | --- | --- |
| `-i, --sample-rate` | `1000` | Polling interval (ms, min 100) |
| `-a, --listen-address` | `127.0.0.1:9810` | HTTP listen address |
| `--web-dir` | — | Override embedded web assets with files from this directory |

### Backend: `src/web.rs`

```rust
#[derive(RustEmbed)]
#[folder = "web/"]
struct Assets;
```

Routes:
- `GET /` → serves `index.html` (from embedded assets or `--web-dir`)
- `GET /assets/*path` → serves static files (JS, CSS)
- `GET /api/metrics` → `{ soc: SocInfo, metrics: Option<Metrics> }` as JSON
- `GET /api/config` → `{ soc: SocInfo }` as JSON

File resolution: if `--web-dir` is set, the server checks that directory first for any requested file, falling back to the embedded assets. This allows users to customize `config.js`, `app.js`, or `style.css` without rebuilding.

### Backend: Metrics Collection (`src/monitor.rs`)

`run_web()` creates the shared state, spawns a std thread running `stream_powermetrics_to_state()`, then starts the axum server on a tokio runtime:

```rust
pub fn run_web(args: WebConfig) -> Result<()> {
    let state = Arc::new(SharedMetricsState { ... });
    thread::spawn(move || stream_powermetrics_to_state(tick_rate, &state));
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(crate::web::serve(state, args.listen_address))?;
}
```

`stream_powermetrics_to_state()` is a dedicated version of the existing `stream_metrics()` that writes directly to `Arc<RwLock<Option<Metrics>>>` instead of sending through an mpsc channel.

### Frontend: Config-Driven Layout

The `config.js` file controls chart positions, sizes, visibility, and titles. The `app.js` init loop reads it and applies grid positioning dynamically:

```js
for (const [key, chart] of Object.entries(cfg)) {
    card.style.gridRow = (chart.row + 1).toString();
    card.style.gridColumn = `${chart.col + 1} / span ${chart.width}`;
}
```

The layout uses a 12-column CSS grid. Chips with multiple SoC cluster types (E/P/S cores) render one line series per cluster.

### Frontend: Charts

| Chart | Type | Data |
| --- | --- | --- |
| CPU Utilization | Multi-line, 0-100% | One series per cluster, 60-sample rolling history |
| CPU Frequency | Multi-line, MHz | One series per cluster, 60-sample rolling history |
| GPU | Dual Y-axis | Utilization % + Frequency MHz, 60-sample history |
| Power | Stacked area + dashed line | CPU/GPU/ANE stack + Package total, 60-sample history |
| Memory | Progress bars | RAM used/total, Swap used/total (formatted bytes) |
| Thermal | Colored label + fill bar | nominal / light / moderate / heavy / critical |

### Frontend: Theme

- CSS custom properties for light/dark themes, toggled via button or system preference
- ECharts theme switches via `echarts.init(dom, null)` / `echarts.init(dom, 'dark')`
- Preference persisted to `localStorage`

## Files Changed

| File | Status |
| --- | --- |
| `Cargo.toml` | Modified — added axum, tokio, tower-http, rust-embed |
| `src/config.rs` | Modified — added `WebConfig`, `Command::Web` |
| `src/web.rs` | **New** — axum server module |
| `src/monitor.rs` | Modified — added `run_web()`, `stream_powermetrics_to_state()` |
| `src/lib.rs` | Modified — registered `mod web` |
| `src/error.rs` | Modified — added `WebServerError` |
| `src/modules/soc.rs` | Modified — added `Clone` derive |
| `src/bin/pumas.rs` | Modified — added `Command::Web` dispatch |
| `web/index.html` | **New** — dashboard HTML |
| `web/app.js` | **New** — dashboard application logic |
| `web/config.js` | **New** — user-editable configuration |
| `web/style.css` | **New** — dashboard styles |
