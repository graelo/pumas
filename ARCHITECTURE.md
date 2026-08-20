# Architecture

Pumas is a power-usage monitor for Apple Silicon Macs. It shells out to macOS
`powermetrics` (which requires root), merges in `sysinfo`/`vm_stat` data, and
renders a terminal UI built with [`iocraft`](https://crates.io/crates/iocraft).

This document describes how the pieces fit together. For build/test commands and
usage, see the `README.md`.

## Data flow

The UI is a **one-directional backend → frontend data plane**. A backend thread
gathers and prepares everything; the frontend only lays out and draws. The
frontend never drives the backend.

```text
bin/pumas.rs ── CLI parse ──▶ monitor::run(args)
                                   │
                    args.json ─────┴───── UI mode
                        │                    │
                        ▼                    ▼
             backend::run_exporter    monitor::run_ui
             (prints JSON lines)      spawn collector thread
                                      + smol::block_on(PumasApp.fullscreen())
                                             │
                    backend::run_collector   │   frontend (ui::PumasApp)
                    ─────────────────────▶  smol::channel::bounded::<Frame>(4)
                    stream powermetrics,        │
                    merge sysinfo/vm_stat,      ▼
                    own all history,        recv().await → render tabs
                    build owned Frame,      (no history, no formatting)
                    tx.send_blocking(frame)
```

1. **`src/bin/pumas.rs`** — entry point; parses CLI args and dispatches to
   `monitor::run()`.
2. **`src/monitor.rs`** — `run()` branches on `--json`. JSON mode calls
    `backend::run_exporter` directly. UI mode (`run_ui`) builds the frame
    channel, spawns the collector on its own OS thread, and runs the fullscreen
    frontend via `smol::block_on(PumasApp.fullscreen())`.
3. **`src/backend/`** — the collector thread streams `powermetrics`, merges
    `sysinfo`, owns all metric history, and ships each sample as one owned,
    `Clone` `Frame` over a bounded `smol::channel`.
4. **`src/ui/`** — the frontend renders `Frame`s. It holds no history and does
    no string formatting or scaling; every string is pre-formatted and every
    sparkline slice pre-computed in the backend. iocraft re-renders on state
    change, so a new frame (≈1/s) or a keypress drives the repaint — there is no
    free-running animation loop.

### The `Frame` contract

`Frame` (`src/backend/frame.rs`) is the single snapshot type crossing the
channel. It carries per-tab sub-structs (`OverviewFrame`, `CpuFrame`,
`GpuFrame`, `MemoryFrame`) with everything **already prepared**: formatted
labels, gauge ratios, and sparkline data plus its scaling ceiling. Geometry
(widths/heights) is deliberately absent — that is a frontend concern derived
from terminal size.

Session-static data (the title-bar header and SoC-tab rows) is built once via
`render_header` / `render_soc_rows` and passed to `PumasApp` as props, not
carried per-frame.

## Module structure

- **`src/modules/`** — UI-agnostic data collection:
  - `powermetrics/` — parses the plist output of macOS `powermetrics` (GPU,
    frequencies, power).
  - `sysinfo.rs` — CPU utilization via the `sysinfo` crate (more accurate than
    powermetrics on M2).
  - `soc.rs` — SoC info via `sysctl` and `system_profiler`.
  - `vm_stat.rs` — memory statistics via the `vm_stat` command.
- **`src/backend/`** — collector + data plane:
  - `mod.rs` — collector thread (`stream` powermetrics loop + sysinfo merge),
    `Frame` builder, and the JSON exporter loop.
  - `frame.rs` — the owned `Frame` snapshot and per-tab sub-structs.
  - `history.rs` — `Signal<T>` ring buffer + per-metric history (backend-owned).
- **`src/ui/`** — terminal UI (iocraft):
  - `app_root.rs` — the `PumasApp` component: render loop, tab state, keyboard
    events, and the `use_future` that drains the frame channel.
  - `layout.rs` — all pixel geometry derived from `use_terminal_size()`
    (frontend-only).
  - `theme.rs` — maps `config::UiColors` to iocraft `Color`.
  - `components/` — reusable widgets (gauge, line_gauge, sparkline, panel,
    tab/title bars).
  - `views/` — per-tab views: Overview, CPU, GPU, Memory, SoC, plus the splash.
- **`src/metrics.rs`** — unified metrics struct combining all data sources.

## JSON mode

`run --json` shares the same collector loop but, instead of building `Frame`s,
`backend::run_exporter` prints one JSON line per sample. There is no UI and no
channel; `monitor::run` branches on `args.json` up front.

## Key dependencies

- **`iocraft`** — terminal UI (component/render model); runs on `smol`.
- **`smol`** — async runtime + the bounded channel for the backend → frontend
  data plane.
- **`plist`** + **`serde`** — parse powermetrics plist output.
- **`sysinfo`** — cross-platform system info (CPU utilization).
- **`clap`** — CLI argument parsing.

## Why this shape

- **One collector thread, one channel.** Blocking `powermetrics`/`sysinfo`/
    `vm_stat` I/O lives on a plain OS thread and pushes owned frames; the UI
    never blocks on I/O. No shared-state locking, no request/response protocol —
    the flow is strictly one-directional.
- **Backend owns history and formatting.** The frontend is a pure function of
    the latest `Frame` plus terminal size, which keeps the render path trivial
    and makes views testable headlessly (render to a `Canvas`, compare text)
    without `sudo` or live `powermetrics`.
