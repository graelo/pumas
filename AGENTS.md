# AGENTS.md

This file contains instructions for coding agents working in this repository.

- Repository: <https://github.com/graelo/pumas>
- Prefer `gh` for GitHub operations.
- Do not mention an agent or assistant in issues, pull requests, comments, or
  commit messages.
- Do not expose private local information, including machine-specific paths.

## Project

`pumas` is a power usage monitor for Apple Silicon Macs, inspired by `nvtop`
and reimplemented from the ideas in `asitop`. The package contains a small
library API and one `pumas` binary. The binary uses macOS's `powermetrics`,
`sysinfo`, `sysctl`, `system_profiler`, and `vm_stat` utilities to collect and
display CPU, GPU, ANE, memory, frequency, and package power metrics.

The monitor requires `sudo` because `powermetrics` requires root access. The
project is Apple-Silicon/macOS-specific; do not add cross-platform behavior
unless the task explicitly requires it.

## Architecture

1. `monitor` selects JSON export or the interactive UI.
2. `backend` owns the collector thread, external processes, metric merging, and
   history. It sends an owned `Frame` snapshot through a `smol::channel`.
3. `ui` owns the iocraft frontend, tab selection, terminal input, and rendering.
4. `modules` contains the external data sources and their parsers.

Key modules:

- `src/config.rs`: Clap configuration shared by the library and binary.
- `src/monitor.rs`: public monitor entry point and UI/JSON dispatch.
- `src/backend/`: collector data plane, prepared frames, and history.
- `src/modules/`: `powermetrics`, `sysinfo`, SoC, and VM-stat sources.
- `src/ui/`: iocraft application, components, layouts, themes, and tab views.
- `src/bin/pumas.rs`: the command-line entry point and completion generator.
- `schema/`: JSON schema and sample export output.

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the detailed data-plane design.

## Verification

The `Makefile` is the canonical definition of local verification tasks. **Read
it before choosing or running verification commands**; do not duplicate its
command implementations here. `make help` lists every target.

The primary targets are:

- `make check`: pre-push gate (formatting, linting, and tests).
- `make check-all`: pre-PR gate (adds dependency, commit-message, Markdown,
  manpage, and GitHub Actions security checks).
- `make fix`: formats code and applies Clippy fixes.
- `make md`: lints Markdown against `rumdl.toml`.
- `make man`: lints `man/pumas.1` with `mandoc`.
- `make ci-security`: runs the Poutine and Zizmor GitHub Actions scans.

The checks assume external tools such as `cargo-nextest`, `cargo-deny`,
`cargo-pants`, `convco`, `poutine`, `zizmor`, `rumdl`, `mandoc`, and
`cargo-llvm-cov` are installed locally. The complete CI test sequence is
implemented in `ci/test_full.sh`; its Nextest profile is configured in
`.config/nextest.toml`.

For focused Rust tests, use `cargo nextest run <test_name>` or
`cargo nextest run <module::tests::name>`. Doc tests run separately with
`cargo test --locked --doc`.

## Documentation and releases

The README is the source of truth for end-user documentation. Keep it and the
[`pumas` manpage](man/pumas.1) in sync with the Clap interface and behavior.
Update both when changing a command, option, default, output mode, or runtime
control. Run `make md` after Markdown edits and `make man` after manpage edits.

Keep crate-level rustdocs in `src/lib.rs` short; do not duplicate the README
there. Public API items still need useful rustdoc because the crate enables the
`missing_docs` lint.

For a release version bump, update `Cargo.toml`, `Cargo.lock`, the versioned
section and comparison links in `CHANGELOG.md`, and the manpage `.TH` header.
Create a `vX.Y.Z` tag; the release workflow derives artifact and GitHub Release
versions from it, so do not manually change workflow tool pins or release
logic.

Commit messages must follow `.convco` Conventional Commit rules. Use
`make commits` to check them.

`Cargo.toml`, `Cargo.lock`, `deny.toml`, and the GitHub workflows define the
release and supply-chain constraints. Preserve `--locked` behavior in Cargo
commands that resolve dependencies.
