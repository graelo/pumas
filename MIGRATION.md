# MIGRATION.md — ratatui → iocraft 0.8 (SOURCE OF TRUTH)

This file is the authoritative design doc for migrating the Pumas TUI from
`ratatui 0.30` + `termion 4` to **`iocraft 0.8.3`**. It is owned by the
**architect** agent and reviewed at every phase for drift. When this doc and
the live code disagree, **trust the code and update this doc** (noting the
change). Verified facts below were checked against the actual source, the
installed `iocraft-0.8.3` crate, the `../gh-board` reference app, and the
screenshots in `~/Downloads/screenshots/` on 2026-06-29.

Branch: `iocraft-migration-v2` (off `main`). Dead attempts live only at the
deleted-branch tip SHA `1d138a2` — reference read-only via `git show <SHA>:path`;
**never check it out.** The local branch `migrate-ratatui-to-iocraft` == `main`
(red herring).

---

## 1. Goals & non-goals

**Goals**
- One-directional data flow: a backend collector thread → frontend render loop.
  The frontend NEVER drives the backend (we omit gh-board's bidirectional half).
- Pixel-close parity with today's ratatui UI across all five tabs (Overview, CPU,
  GPU, Memory, SoC) + title bar + tab bar + splash.
- `run --json` preserved **byte-identical**, sharing the same collector.
- Frontend holds **zero** history or formatting logic — every string, ratio,
  sparkline slice and scaling factor is computed in the backend and shipped in an
  owned `Frame`.

**Non-goals**
- No new features, no key bindings beyond today's (`q`/`x`/`Ctrl-C`/`Esc` quit;
  `←`/`BackTab` prev tab; `→`/`Tab` next tab).
- No mouse interaction beyond what termion gave for free (none functional today).
- No theming changes — `config::UiColors` stays as the only color coupling.

---

## 2. The two failure modes — design these OUT (hard "DO NOT" rules)

**DO NOT #1 — Do not build a custom `Container`/border component.**
The previous attempt hand-rolled a `Container` to draw `┌─┐` borders, never solved
passing children through it, and gutted it. iocraft `View` has **native** border
props. Use them directly (see §7). There is no `panel.rs` that "draws" a
border — `panel.rs` is only a thin convenience wrapper around `View`'s native
props, and may be skipped entirely if a bare `View(...)` is clearer.

**DO NOT #2 — Do not over-engineer the data plane.**
The previous attempt used a tokio runtime + `tokio::select!` + a second unused
channel + a Request/Event actor protocol + a blocking `recv_timeout` on the UI
executor. **None of that.** The design is exactly:

```
[std::thread collector]  --smol::channel::bounded(4)-->  [use_future recv().await -> State<Frame>]
```

No tokio. No actor protocol. No polling timer on the UI thread. No `Mutex`. No
blocking recv on the iocraft executor.

**DO NOT #3 — Do not ship without snapshot tests.** Every component and every tab
lands with a committed headless snapshot test (§7.8, §8). The last attempt shipped
zero tests and never reached parity.

---

## 3. Target module layout

Verified against the real `src/` tree. `error.rs`, `config.rs`, `lib.rs`,
`metrics.rs`, `metric_key.rs`, `modules/*` all stay.

```
src/
  bin/pumas.rs            # KEEP shape: CLI dispatch Run / GenerateCompletion
  monitor.rs              # REWRITE: run(args) branches json vs ui; spawns backend; smol::block_on(PumasApp.fullscreen())
  backend/
    mod.rs                # collector thread: stream_metrics loop + sysinfo merge + history; builds Frame; send_blocking
    frame.rs              # Frame snapshot type (owned, Clone) + per-tab sub-structs (§4)
    history.rs            # Signal<T> ring buffer + peak (port of src/signal.rs verbatim)
  metrics.rs              # KEEP — parsing/merge logic is UI-agnostic
  metric_key.rs           # KEEP
  modules/                # KEEP all: powermetrics/, sysinfo.rs, soc.rs, vm_stat.rs
  units.rs                # KEEP — formatters reused by the backend Frame builder
  ui/
    app_root.rs           # #[component] PumasApp: State<Option<Frame>>, State<usize> tab, State<bool> should_exit
    theme.rs              # UiColors u8 -> iocraft Color (replaces app.rs::AppColors), §6
    components/
      panel.rs            # OPTIONAL thin wrapper over View native borders (NOT a border drawer)
      gauge.rs            # block-bar gauge with centered NN% label (Overview)
      line_gauge.rs       # single-row ━ ratio bar with left label (CPU/GPU rows)
      sparkline.rs        # ▁▂▃▄▅▆▇█ multi-row vertical bars (§5 — REBUILD, do not salvage glyphs)
      tab_bar.rs          # the 5-tab bordered bar, accent highlight
      title_bar.rs        # "Pumas vX" left + "{brand} (cores: …)" right
      freq_table.rs       # 2-col DVFM frequency table (CPU/GPU/SoC share)
    views/
      splash.rs           # startup logo + "Starting up…"
      overview.rs cpu.rs gpu.rs memory.rs soc.rs
    snapshot.rs           # test-only harness: element -> render(Some(w)) -> String
  tests/snapshots/*.snap  # committed golden text
```

**Modules deleted in Phase 3** (after full port): `app.rs`, `signal.rs`,
`ui/main_screen.rs`, `ui/startup_screen.rs`, `ui/tab_overview.rs`, `ui/tab_cpu.rs`,
`ui/tab_gpu.rs`, `ui/tab_memory.rs`, `ui/tab_soc.rs`, `ui/mod.rs::draw`.

---

## 4. Data plane contract — the `Frame`

`Frame` (`backend/frame.rs`) is one **owned, `Clone`** snapshot sent per sample.
It carries everything the UI needs **already prepared**. The frontend pairs/lays-out
but does NO history math and NO string formatting.

### 4.1 History moves into the backend

Port `src/signal.rs` `Signal<T>` **verbatim** into `backend/history.rs`:
- `Signal<f32>` storage: `peak: f32`, `max: f32`, `points: VecDeque<u64>` (values
  pushed as `f32`, stored via `to_u64()`).
- `push(v)` updates `peak`, evicts front at capacity, pushes back.
- `as_slice_last_n(n) -> &[u64]`.
- History is `HashMap<MetricKey, Signal<f32>>` (= today's `app::History`).
- The collector owns the `History`, pushes each sample (port `app.rs::update_history`
  verbatim — the per-cluster/per-cpu/gpu/ane/power/memory key set is unchanged), then
  builds the `Frame` from current `Metrics` + signal slices/peaks.

`MetricKey` (unchanged, see `src/metric_key.rs`): `ClusterActivePercent(ClusterId)`,
`CpuActivePercent(u16)`, `CpuFreqPercent(u16)`, `GpuActivePercent`, `GpuFreqPercent`,
`AneActivePercent`, `CpuPowerW`, `GpuPowerW`, `AnePowerW`, `PackagePowerW`,
`RamUsageBytes`, `SwapUsageBytes`. All signals created with `max=100.0` except the
power signals (`max = soc.max_*_w`) and memory (`max = ram_total`/`swap_total`).

### 4.2 The reusable sub-types

```rust
/// A gauge + its sparkline, fully prepared. ratio drives the gauge fill,
/// `title` is the pre-formatted gauge/label line, `spark`/`spark_max` drive
/// the sparkline. NO peak field — the peak is already baked into `title`.
#[derive(Clone)]
pub struct Meter {
    pub title: String,     // e.g. "E-Cluster: 21.8 % @ 973 MHz (peak: 22.1 %)"
    pub ratio: f64,        // 0.0..=1.0, gauge fill
    pub spark: Vec<u64>,   // already trimmed (last-N) where the tab fixes N; else full history
    pub spark_max: u64,    // scaling, overshoot already applied (see §4.4)
}

/// Package: text title (no gauge) + sparkline.
#[derive(Clone)]
pub struct SparkText {
    pub title: String,     // "CPU+GPU+ANE: 130.55 mW (peak: 6.48 W)"
    pub spark: Vec<u64>,
    pub spark_max: u64,    // = signal.max — NO overshoot (Package only)
}
```

### 4.3 `Frame` shape

```rust
#[derive(Clone)]
pub struct Frame {
    pub overview: OverviewFrame,
    pub cpu: CpuFrame,
    pub gpu: GpuFrame,
    pub memory: MemoryFrame,
    // SoC + header are static for the session -> NOT per-frame (see note).
}

pub struct OverviewFrame {
    pub cpu_clusters_title: String,        // " CPU Clusters: {watts2(cpu_w)} (peak: {watts2(peak)}) " (panel border title)
    pub e_meters: Vec<Meter>,              // one per E-cluster, in order; frontend pairs via chunks(2)
    pub p_meters: Vec<Meter>,              // one per P-cluster
    pub s_meters: Vec<Meter>,              // one per S-cluster (M5 Pro/Max+)
    pub gpu: Meter,                        // "GPU: {p1} @ {mhz} | {w} (peak: {p1} | {w})"
    pub ane: Meter,                        // "ANE: {p1} | {w} (peak: {p1} | {w})"; ratio = ane_w/max_ane_w
    pub package: SparkText,                // spark_max = sig.max (no overshoot)
    pub thermals: Thermals,
    pub ram: Meter,                        // "Memory Used: {p1} = {used} / {total} (peak: {p1} = {used})"
    pub swap: Meter,                       // "SWAP: {p1} = {used} / {total} (peak: {used})"
}

pub struct Thermals { pub pressure: String, pub is_nominal: bool }  // is_nominal => accent, else Yellow

pub struct CpuFrame {
    pub clusters: Vec<CpuCluster>,         // E… then P… then S…, each its own bordered block
    pub freq_table: FreqTable,
}
pub struct CpuCluster { pub title: String, pub cpus: Vec<CpuRow> }   // title = " {name}: "
pub struct CpuRow {
    pub id_label: String,                  // "{id:2} -"  (accent)
    pub act_ratio: f64,
    pub act_label: String,                 // "{:.1}%"
    pub act_spark: Vec<u64>,               // last 8
    pub act_spark_max: u64,                // 1.05*max
    pub freq_value: String,                // units::mhz(freq) e.g. "972 MHz"
    pub freq_ratio: f64,
    pub freq_spark: Vec<u64>,              // last 8
    pub freq_spark_max: u64,               // 1.05*max
}

pub struct GpuFrame {
    pub act_ratio: f64, pub act_label: String,           // "{:.1}%"
    pub act_spark: Vec<u64>, pub act_spark_max: u64,      // last 8, 1.05*max
    pub freq_value: String, pub freq_ratio: f64,
    pub freq_spark: Vec<u64>, pub freq_spark_max: u64,    // last 8, 1.05*max
    pub power_value: String,                              // units::watts2(gpu_w)
    pub power_spark: Vec<u64>, pub power_spark_max: u64,  // last 8, 1.05*max
    pub peak_text: String,                               // "Peak: {p1} | {w}"
    pub thermals: Thermals,
    pub freq_table: FreqTable,
}

/// 2-col DVFM table shared by CPU/GPU. Rows: (left_label, right_value_bold).
pub struct FreqTable { pub rows: Vec<(String, String)> }

pub struct MemoryFrame {
    // VmStats::collect() MUST move into the backend (see Discrepancy D2).
    pub vm_lines: Vec<MemLine>,            // pre-formatted, colored (Activity-Monitor block)
    pub sysinfo_lines: Vec<MemLine>,       // pre-formatted, colored (Sysinfo block)
}
pub struct MemLine { pub spans: Vec<MemSpan> }
pub struct MemSpan { pub text: String, pub role: ColorRole }   // role -> theme color (§6)
```

**Header & SoC are session-static** — `SocInfo` is collected once before the loop
and never changes. Do NOT put it in `Frame`. Pass a `RenderedHeader`
(`program_name`, `machine_desc`) and the SoC-tab rows as separate one-time props /
root state. The splash is shown while `frame_state` is `None`.

### 4.4 Sparkline scaling rule (CRITICAL)

`spark_max = (1.05 * signal.max) as u64` **everywhere** EXCEPT the Overview
**Package** sparkline, which uses `spark_max = signal.max as u64` (no overshoot).
The `1.05` overshoot keeps bars from touching the gauge above. Source constants:
`SPARKLINE_MAX_OVERSHOOT = 1.05` in `tab_overview.rs:22`, `tab_cpu.rs:20`,
`tab_gpu.rs:19`; package exception at `tab_overview.rs:663` (`.max(sig.max as u64)`).

### 4.5 Channel & cadence

`smol::channel::bounded::<Frame>(4)`. Collector calls `tx.send_blocking(frame)`;
on send error (UI gone) it kills powermetrics and exits. Frontend: a single
`use_future` running `while let Ok(frame) = rx.recv().await { frame_state.set(Some(frame)); }`.
iocraft re-renders on `State` change, so this reproduces today's event-gated repaint
(≈1/sample + per keypress) with no free-running animation loop.

> **Note vs gh-board:** gh-board's views use `std::sync::mpsc` + `try_recv()` inside a
> `use_future` *polling* loop with `smol::Timer`. We deliberately do NOT copy that.
> `smol::channel::Receiver::recv()` is `async`, so the blocking-await form above is
> correct and simpler. The gh-board recv code is a counter-example, not a template.

### 4.6 JSON mode (`run --json`)

The collector runs the same `stream_metrics` loop but, instead of building `Frame`s,
prints one JSON line per sample. Port `monitor.rs::export` / `main_exporter_loop`
unchanged: `println!("{}", json!({ "soc": soc_info, "metrics": metrics }))`. No UI,
no channel. Branch in `monitor::run` on `args.json` exactly as today
(`monitor.rs:40`). A golden test guards byte-identity (§8).

---

## 5. Sparkline & gauge math — REBUILD, partial salvage only

The dead-branch `gauge.rs`/`sparkline.rs` at `1d138a2` are only **partially** usable.
Read them for reference via `git show 1d138a2:src/ui/components/gauge.rs` etc.

| Concern | Salvage? | Reason |
|---|---|---|
| Gauge `ratio→filled_width` = `(ratio*width).round()` | ✅ reuse | correct |
| Gauge label placement | ❌ rebuild | dead version *appends* label after the bar; ratatui **centers** the `NN%` label overlaid on the bar (filled fg/bg). Overview parity needs centered label. |
| Sparkline glyphs | ❌ rebuild | dead version uses only 4 glyphs `[' ','▁','▂','▃']` and `(v*4) as usize`; ratatui uses **8-level** `symbols::bar::NINE_LEVELS` = `▁▂▃▄▅▆▇█`. |
| Sparkline rows | ❌ rebuild | dead version is **single-row**; Overview sparklines are **multi-row vertical bars** (3 or 9 rows). |

**Sparkline algorithm to implement (matches ratatui):** given values `v[]`, `max`,
width `W`, height `H` rows. For each column take `min(v, max)`; the bar's total height
in eighths is `e = round(v / max * H * 8)`. Render top row → bottom row: for row `r`
(0=top), the cell shows the glyph for `clamp(e - 8*(H-1-r), 0, 8)` where 0 = space and
1..=8 maps to `▁▂▃▄▅▆▇█`. Single-row sparklines (CPU/GPU per-core, H=1) are the special
case `H=1`. Color: `history_fg` on `history_bg`.

**Gauge (Overview block-bar):** width `W`; `filled = round(ratio*W)`; fill `filled`
cells `gauge_fg`-on-`gauge_bg` and the rest `gauge_bg`; overlay the centered label
`"{NN}%"` (integer percent, ratatui default) — the label text sits centered over the
bar, its glyphs taking the gauge colors of whichever half they fall on. Height 2 rows
(`GAUGE_HEIGHT`), label on the bar row.

**Line gauge (CPU/GPU rows):** single row of `━` (U+2501, `symbols::line::THICK.horizontal`).
`filled = round(ratio*W)` cells in `gauge_fg`, remainder in `gauge_bg`. Activity rows
prefix the label `"{:.1}%"`; frequency rows have no label. (ratatui `LineGauge` renders
label then the line.)

---

## 6. Color mapping (`ui/theme.rs`)

**Verified:** `iocraft::Color` re-exports `crossterm::style::Color` (style.rs:10).
The indexed-color variant is **`Color::AnsiValue(u8)`** (crossterm 0.27,
`color.rs:93`). This replaces ratatui's `Color::Indexed(u8)` used in
`app.rs::AppColors::color`.

| Role | Source (`config::UiColors`) | Default | iocraft mapping |
|---|---|---|---|
| accent | `accent` | 2 (green) | `Color::AnsiValue(accent)` |
| gauge_fg | `gauge_fg` | 2 (green) | `Color::AnsiValue(gauge_fg)` |
| gauge_bg | `gauge_bg` | 7 (white) | `Color::AnsiValue(gauge_bg)` |
| history_fg | `history_fg` | 4 (blue) | `Color::AnsiValue(history_fg)` |
| history_bg | `history_bg` | 7 (white) | `Color::AnsiValue(history_bg)` |

Fixed (named) colors, NOT from `UiColors`:
- Thermal pressure: `is_nominal` → `accent`, else `Color::Yellow`
  (`tab_overview.rs:679`, `tab_gpu.rs:207`).
- Splash logo: top-left `Color::Blue`, top-right `Color::Green`, bottom
  `Color::Magenta`, wordmark default (`startup_screen.rs:80-85`).
- `MemLine`/`MemSpan` `ColorRole` enum → maps to {accent, gauge_fg, history_fg, default}
  per the Memory tab's per-span styling (`tab_memory.rs`).

`theme.rs` exposes a `Theme { accent, gauge_fg, gauge_bg, history_fg, history_bg }`
of `iocraft::Color`, built once from `UiColors`, passed by value (it's `Copy`).

> Note: terminal palettes render AnsiValue(2) as their "green" slot; on the user's
> theme this looks olive/dark-yellow (see screenshots). That is correct — do not
> substitute an RGB green.

---

## 7. iocraft 0.8 idiom cheatsheet (verified vs crate + gh-board)

### 7.1 Component signature & the `Option<Rendered*>` prop idiom
```rust
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct GaugeProps { pub data: Option<RenderedGauge> }   // ONE owned prop

#[component]
pub fn Gauge(props: &mut GaugeProps) -> impl Into<AnyElement<'static>> {
    let Some(g) = props.data.take() else { return element! { View }.into_any(); };
    element! { /* … use g … */ }.into_any()
}
```
- Build an owned `Rendered*`/`Meter` struct outside, pass as the single
  `Option<T>` prop, `props.take()` inside, return `impl Into<AnyElement<'static>>`.
- Always `.into_any()` on both branches so the two arms unify.
- Pattern proven in gh-board `components/{table,footer,scrollbar,tab_bar}.rs`.

### 7.2 `View` layout + native borders
```rust
element! {
    View(
        flex_direction: FlexDirection::Column,   // or ::Row
        flex_grow: 1.0_f32,
        width: 120u32, height: 3u32,
        padding_left: 1, padding_top: 1, margin_right: 2,
        justify_content: JustifyContent::Center,
        background_color: Color::Reset,
        display: Display::Flex,                   // ::None to hide
        // --- native borders (DO NOT hand-roll) ---
        border_style: BorderStyle::Single,        // None|Single|Double|Round|Bold|Classic|Custom(BorderCharacters)
        border_edges: Edges::all(),               // Edges::Top | Edges::Bottom | … (bitflags)
        border_color: theme.accent,
    ) { /* children */ }
}
```
- `BorderStyle` variants verified in `view.rs:10`. `Custom(BorderCharacters{ top, bottom, left, right, top_left, top_right, bottom_left, bottom_right })` for partial/odd borders.
- `Edges` is a bitflags set (`style.rs:225`): `Top|Right|Bottom|Left`, combine with `|`, full = `Edges::all()`.
- **Border titles:** iocraft `View` has no ratatui-style `Block::title`. Ratatui draws
  the title *inside the top border* (e.g. `┌ CPU Clusters: … ────┐`). To match, render
  the panel as a `View` with `border_edges` excluding `Top`, and overlay/compose the
  top line manually (a `View` row containing `┌ `, the title `Text`, and a filled
  `─…┐` tail). Confirm exact glyphs against the snapshot. This is the one fiddly bit of
  border parity — prove it in Phase 0.

### 7.3 Text / MixedText
```rust
Text(content: s, color: theme.accent, weight: Weight::Bold, wrap: TextWrap::NoWrap, align: TextAlign::Right)
MixedText(contents: vec![ MixedTextContent::new(t).color(c).weight(Weight::Bold) ], wrap: TextWrap::NoWrap)
```

### 7.4 Hooks
```rust
let (width, height) = hooks.use_terminal_size();          // (u16, u16)
let mut tab     = hooks.use_state(|| 0usize);
let mut frame   = hooks.use_state(|| Option::<Frame>::None);
let should_exit = hooks.use_state(|| false);
let mut system  = hooks.use_context_mut::<SystemContext>();
hooks.use_future(async move { while let Ok(f) = rx.recv().await { frame.set(Some(f)); } });
```

### 7.5 Keyboard events
```rust
hooks.use_terminal_events(move |event| {
    if let TerminalEvent::Key(KeyEvent { code, kind, modifiers, .. }) = event {
        if kind == KeyEventKind::Release { return; }       // ignore key-up
        match code {
            KeyCode::Char('q') | KeyCode::Char('x') | KeyCode::Esc => should_exit.set(true),
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => should_exit.set(true),
            KeyCode::Left  | KeyCode::BackTab => tab.set(prev),
            KeyCode::Right | KeyCode::Tab     => tab.set(next),
            _ => {}
        }
    }
});
```
Tab math today (`app.rs`): next = `(i+1) % 5`, prev = `i==0 ? 4 : i-1`.

### 7.6 Exit (flag, never call exit() in the closure)
```rust
if should_exit.get() { system.exit(); }   // at render time, after the events hook
```
(gh-board `app.rs:381`.) Setting `system.exit()` directly inside the event closure is wrong.

### 7.7 main / entry
```rust
// monitor.rs run(): non-json branch
std::panic::set_hook(/* write panic.log; TUI swallows stderr */);
let (tx, rx) = smol::channel::bounded::<Frame>(4);
std::thread::spawn(move || backend::run_collector(soc_info, cfg, tx));   // owns history
smol::block_on(element! { PumasApp(rx: …, header: …, theme: …) }.fullscreen())?;
```
(gh-board `main.rs` tail.) `iocraft` runs on `smol` internally — no tokio anywhere.

### 7.8 Snapshot harness (`ui/snapshot.rs`, test-only)
**Verified:** `ElementExt::render(&mut self, Option<usize>) -> Canvas` (element.rs:167);
`Canvas: Display` writes **UNSTYLED** plain text (canvas.rs:436, `write_impl(.., false, false)`).
```rust
pub fn render_to_text(mut el: AnyElement<'static>, width: usize) -> String {
    el.render(Some(width)).to_string()      // plain text, no ANSI
}
```
- Build a synthetic `Frame` fixture (deterministic; e.g. an M5 Max shape matching the
  screenshots), render a view at a fixed width (e.g. 120), compare to
  `tests/snapshots/<name>.snap`.
- ⚠️ **Limitation:** plain-text snapshots capture **layout/glyphs only, NOT color.**
  Color parity (accent/gauge/history/thermal) is verified separately: either by a
  small `Canvas`-cell inspection test (`canvas.rs` exposes cells/`write_ansi`) for one
  representative widget, or by the user's live smoke check at the final gate. State this
  in every snapshot test so reviewers know color is out of scope there.

---

## 8. Phase gates

**Phase 0 — De-risk spike.** Add `iocraft` + `smol` deps (keep ratatui for now).
Build `components/{gauge,sparkline,line_gauge}.rs` and (if used) `panel.rs` using native
`View` borders, plus the snapshot harness. Prove a bordered titled panel containing a
gauge + multi-row sparkline renders pixel-close to an Overview slice, as a committed
snapshot test. **Gate:** panel border-title + gauge centered-label + 8-level multi-row
sparkline parity confirmed against a screenshot crop; harness runs headless (no sudo).

**Phase 1 — Backend & data plane.** `backend/{mod,frame,history}.rs`: collector thread
(port `stream_metrics` + sysinfo merge + `update_history`), history in backend, `Frame`
builder, `smol::channel`. Rewire `monitor::run` to branch json vs ui, spawn backend,
`smol::block_on(PumasApp.fullscreen())`. Minimal `PumasApp` showing raw frame text to
prove the pipe. **Gate:** live metrics flow to a debug view; `run --json` output
byte-identical (golden test); no tokio in the tree; `cargo clippy -D warnings` clean.

**Phase 2 — Tabs, one at a time, snapshot-gated.** Order: title_bar, tab_bar, splash,
then Overview → CPU → GPU → Memory → SoC. Each lands with a committed snapshot test and
verifier parity sign-off against the matching screenshot before the next begins.
**Gate per tab:** snapshot matches; verifier confirms parity vs the screenshot.

**Phase 3 — Remove ratatui & harden.** Delete `ratatui`/`termion` deps and all dead
ratatui code (§3 deletion list). Zero dead code, zero clippy warnings. Run
`./ci/test_full.sh`, `cargo fmt --all -- --check`, `cargo nextest run --locked`,
`cargo test --locked --doc`. Update `CLAUDE.md` dependency/architecture notes.
Conventional-commit history, **no AI attribution** (project convention). **Gate:** full
CI green; ratatui fully gone; all five tabs at parity.

Each phase: coders leave **zero dead code and zero clippy warnings**; a verifier gates
and may halt to ask the user; the architect reviews for drift vs this doc.

---

## 9. Per-tab UI parity reference

Line refs are into the current ratatui source. Heights are the **nominal** ratatui
constraints; ratatui clips child constraints to the parent block, so the *effective*
sparkline height can be smaller than the nominal `9` — **the snapshot fixture is the
final arbiter of pixel parity**, not these numbers.

### 9.1 Title bar — `main_screen.rs:31-50` (1 row)
- Left: `Pumas v{CARGO_PKG_VERSION}` (default color).
- Right (accent, right-aligned): ` {brand} (cores: {E}E+{P}P+{GPU}GPU) ` — uses
  `num_efficiency_cores`, `num_performance_cores`, `num_gpu_cores`.
- Both overlaid on the same 1-row area (left para + right-aligned para). In iocraft:
  one `View(flex_direction: Row, justify_content: SpaceBetween)` with two `Text`.

### 9.2 Tab bar — `main_screen.rs:52-71` (3 rows, bordered all edges)
- Tabs `Overview | CPU | GPU | Memory | SoC`; active tab `accent` + `Bold`; inactive
  default. ratatui `Tabs` divider is `" | "`; first tab has a leading space.
- iocraft: `View(border_style: Single, border_edges: all)` containing a Row of `Text`
  per tab with `" | "` separators (see gh-board `tab_bar.rs` for the highlight idiom,
  but our divider is `|` not a background block).

### 9.3 Overview — `tab_overview.rs`
Vertical stack of 4 bordered blocks then `Min(0)` spacer. Block heights:
`2 + (GAUGE_HEIGHT=2 + SPARKLINE_HEIGHT=3)` per cluster-row etc.

| Block | Border title | Content | Frame fields |
|---|---|---|---|
| CPU Clusters | ` CPU Clusters: {w} (peak: {w}) ` | E then P then S clusters; each cluster = gauge(2 rows)+sparkline(3 rows); clusters paired **2-up** within each kind via `chunks(2)` (left/right halves split by a 2-col gap), single if odd | `overview.cpu_clusters_title`, `e_meters`, `p_meters`, `s_meters` |
| GPU & ANE | ` GPU & ANE ` | left half = GPU gauge(2)+sparkline(9); right half = ANE gauge(2)+sparkline(9); 2-col gap between | `overview.gpu`, `overview.ane` |
| Package + Thermals | ` Package ` (70%) + ` Thermals ` (30%) | Package: text(1 row)+sparkline(3, **no overshoot**); Thermals: `Pressure: {x}` (accent if Nominal else Yellow) | `overview.package`, `overview.thermals` |
| Memory & SWAP | ` Memory & SWAP ` | left = RAM gauge(2)+sparkline(9); right = SWAP gauge(2)+sparkline(9) | `overview.ram`, `overview.swap` |

Gauge title strings (exact, reuse `units.rs`):
- cluster: `"{name}: {percent1(act*100)} @ {mhz(freq)} (peak: {percent1(sig.peak)})"`
- GPU: `"GPU: {percent1(act*100)} @ {mhz(freq)} | {watts2(gpu_w)} (peak: {percent1(sig.peak)} | {watts2(gpu_pow.peak)})"`
- ANE: `"ANE: {percent1(ane_ratio*100)} | {watts2(ane_w)} (peak: {percent1(sig.peak)} | {watts2(ane_pow.peak)})"`; ratio = `ane_w/max_ane_w`
- Package: `"CPU+GPU+ANE: {watts2(package_w)} (peak: {watts2(sig.peak)})"`
- RAM: `"Memory Used: {percent1(ratio*100)} = {bibytes1(used)} / {bibytes1(total)} (peak: {percent1(peak/total*100)} = {bibytes1(peak)})"`
- SWAP: `"SWAP: {percent1(ratio*100)} = {bibytes1(used)} / {bibytes1(total)} (peak: {bibytes1(peak)})"`

### 9.4 CPU — `tab_cpu.rs`
- One bordered block per cluster (E…, P…, S…), title ` {name}: `; block height
  `2 + 1*ncpus`.
- Each CPU = 1 row: col `[5]` `"{id:2} -"` (accent); remainder split half/half into
  **activity** and **frequency**:
  - activity `[hist 8+1][gauge fills rest]`: sparkline(last 8 of `CpuActivePercent`) +
    LineGauge ratio=`active_ratio`, label `"{:.1}%"`.
  - frequency `[6 "freq:"][hist 8+1][10 "{mhz}"][gauge fills rest]`: `freq:` label +
    sparkline(last 8 of `CpuFreqPercent`) + `units::mhz(freq)` + LineGauge ratio=`freq_ratio`
    (no label).
- Then a `Frequencies` bordered table (height `2+5`), 2-col, rows: present clusters
  (`E-Cluster:`/`P-Cluster:`/`S-Cluster:` → space-joined `{:4}` DVFM freqs), a blank row,
  and `Note: Hardware-wise, CPUs quickly shift between the above frequencies.`; right
  column **bold**. Fields: `cpu.clusters`, `cpu.freq_table`.

### 9.5 GPU — `tab_gpu.rs`
- GPU block (border title `GPU: `, height 4): top row = activity (sparkline 8 + LineGauge
  `{:.1}%`) | frequency (`freq:` + sparkline 8 + `{mhz}` + LineGauge); bottom row = power
  (sparkline 8 + `watts2(gpu_w)`) | `Peak: {percent1} | {watts2}`.
- Thermals block (height 3): `Pressure: {x}` accent/Yellow.
- `Frequencies` table (height 5): rows `GPU:` (DVFM freqs), blank, `Note: …GPUs…`.
- Fields: `gpu.*`, `gpu.thermals`, `gpu.freq_table`.

### 9.6 Memory — `tab_memory.rs` (pure text, margin 1)
- ` VM Statistics (Activity Monitor compatible) ` block (height 18): lines —
  `Physical Memory Total: {:.2} GB` (accent); blank; `═══ ACTIVITY MONITOR CALCULATION ═══`
  (accent); `App Memory (Anonymous): {:.2} GB`, `Wired Memory:         + {:.2} GB`,
  `Compressed:           + {:.2} GB` (gauge_fg labels); `                      ─────────`
  (history_fg); `Memory Used Total:      {:.2} GB` (accent); blank;
  `═══ OTHER MEMORY CATEGORIES ═══` (history_fg); `Cached Files: / Free: / Active: /
  Inactive: {:.2} GB` (gauge_fg labels). Values from `VmStats` (page math, see
  `tab_memory.rs:36-49`).
- ` Sysinfo Statistics ` block (height 8): `RAM Used: {percent1} = {bibytes1} /
  {bibytes1} ({:.1}%)` (accent label); `Swap Used: {percent1} = {bibytes1} / {bibytes1}`
  (accent); blank; `Note: …vm_stat…` (history_fg).
- Fields: `memory.vm_lines`, `memory.sysinfo_lines` (pre-built `MemLine`s). **VmStats
  collection moves to backend (D2).**

### 9.7 SoC — `tab_soc.rs` (borderless 2-col table, widths 20/16)
Rows (right col **bold**): `SoC brand name:`, `CPU cores:`, `- Efficiency cores:`,
`- Performance cores:`, `GPU cores:`, `Max CPU power: {watts}`, `Max GPU power:`,
`Max ANE power:`. Static from `SocInfo` — build once, not per-frame.

### 9.8 Splash — `startup_screen.rs`
40×17 logo, vertically + horizontally centered. Top-left (`Color::Blue`, 9 rows ×15w),
top-right (`Color::Green`, 9 rows ×25w), bottom (`Color::Magenta`, 8 rows), then the
`pumas` ASCII wordmark, then a 2-row spacer, then `Starting up…` (centered). Constants:
`LOGO2_HEIGHT=17`, `LOGO2_WIDTH=40`, `LOGO2_TOP_LEFT_HEIGHT=9`, `LOGO2_TOP_LEFT_WIDTH=15`,
`LOGO2_TOP_RIGHT_WIDTH=25`, `LOGO2_BOTTOM_HEIGHT=8`, `PUMAS_TEXT_HEIGHT=6`,
`SPACER_HEIGHT=2`. Shown while `frame_state` is `None`. Logo string literals copy verbatim
from `startup_screen.rs:94-151`.

---

## 10. Discrepancies found (plan vs actual code) & risks

**D1 — Overview memory labels.** The plan §"UI parity" and the stale ASCII doc-comments
say "RAM & SWAP" / "RAM:". The **actual code + screenshots** use border title
` Memory & SWAP ` and gauge label `Memory Used:` (`tab_overview.rs:516,553`). **Use the
code/screenshot strings** (encoded in §9.3). Confirmed against `5.tab-memory.png` and
`2.tab-overview.png`.

**D2 — Memory tab does live I/O on the UI thread.** `tab_memory.rs:35` calls
`VmStats::collect()` (spawns `vm_stat`) *during draw*. In the backend-owns-everything
design this MUST move into the collector; `MemoryFrame` ships the pre-formatted lines.
Cadence: collect `vm_stat` once per sample alongside powermetrics. (Minor behavior
change: today it re-collects every repaint incl. keypress; new design collects per
sample — acceptable and more correct.)

**D3 — Salvage is only partial.** Dead-branch `gauge.rs`/`sparkline.rs` (`1d138a2`) are
NOT parity-correct (4-glyph single-row sparkline; appended gauge label). Reuse only the
`ratio→width` arithmetic; rebuild glyphs/rows/centered-label per §5. **This is the
highest-risk area** — gate it hard in Phase 0.

**D4 — Border titles.** iocraft `View` has no `Block::title`. The ratatui look
(`┌ Title ──┐`) must be composed manually (§7.2). Medium risk; prove in Phase 0.

**D5 — Sample-rate comment drift.** `monitor.rs:189` docstring says "0.5 sec by default"
but `config.rs:40` default is `1000` ms. Not load-bearing for the migration; ignore.

**D6 — Snapshot tests can't see color.** Plain-text `Canvas` output omits ANSI (§7.8).
Color parity needs a separate Canvas-cell test and/or the user's live smoke check. Build
one representative color-assertion test so the gate isn't blind to color regressions.

**D7 — `impl Into<AnyElement<'static>>` lifetime.** gh-board leaf components return
`'static` and take owned `Rendered*` props (no borrowed data in the element). Follow that
exactly; do not return elements borrowing from `props` or locals (the dead-branch
`Sparkline<'a>` borrowed `&'a [f32]` — avoid; ship owned `Vec<u64>`).

**Risk I disagree with / flag in the plan:** the plan's parity section is written from
memory and contains the D1 label error and implies the dead-branch gauge/sparkline are
"reusable math" without qualification (D3). Both are corrected here. Everything else in
the plan checks out against the code.
