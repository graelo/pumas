# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-04-14

### Added

- Renovate for automated dependency updates.
- Reusable supply-chain workflows (cargo-audit, ci-security,
    supply-chain-schedule).
- Build provenance attestation via Sigstore on releases.
- Zizmor and poutine CI security scanning.

### Changed

- Bump to Rust edition 2024 with MSRV 1.88.
- Harden all GitHub Actions workflows: pin actions by SHA, least-privilege
  permissions, `persist-credentials: false`, cache poisoning prevention,
  template injection fixes, semver tag filter.
- Replace `ncipollo/release-action` with `gh release create`.
- Replace PATs with short-lived GitHub App tokens for Homebrew and Renovate.

### Fixed

- Avoid underflow in startup screen layout using `saturating_sub`.
  ([@yeungtuzi](https://github.com/yeungtuzi))
- Trigger cargo-audit on `Cargo.toml` changes (not just `Cargo.lock`).

## [0.4.0] - 2026-03-30

### Added

- Support for M5 Pro/Max cores.
  ([@Frostman](https://github.com/Frostman))
- `Ctrl-C` handling to quit the application.
  ([@kunlinglio](https://github.com/kunlinglio))
- Helpful error message when running without `sudo`.
  ([@kunlinglio](https://github.com/kunlinglio))

### Changed

- Rename M-clusters to S-clusters following macOS 26.4.
- Refresh SoC max power ceilings for Apple Silicon.
  ([@yeungtuzi](https://github.com/yeungtuzi))
- Simplify error handling for powermetrics failures.

### Fixed

- Exit TUI and show error message correctly when powermetrics crashes.
  ([@kunlinglio](https://github.com/kunlinglio))

## [0.3.4] - 2026-02-05

### Changed

- Replace stringly-typed history keys with `MetricKey` enum.
- Remove remaining uses of `unwrap`.

### Fixed

- Better layout iterator context.
- Better error propagation for powermetrics.
- Handle numeric edge cases.

## [0.3.3] - 2025-12-16

### Added

- GPU power and thermals in the GPU tab.

### Changed

- Strip symbols in release builds by default.

## [0.3.2] - 2025-12-16

### Fixed

- Use "Pages occupied by compressor" for memory stats.

## [0.3.1] - 2025-09-27

### Fixed

- Memory ratio calculation bug.
- More robust memory metrics collection.

## [0.3.0] - 2025-09-27

### Added

- New Memory tab with detailed memory breakdown.
- Better memory reporting using `vm_stat`.
- Memory calculation example (`vmstat` example).

### Removed

- Thermal pressure display (superseded by per-tab reporting).

## [0.2.5] - 2025-05-24

### Changed

- Update documentation with quick-launch instructions.

## [0.2.4] - 2024-08-16

### Changed

- Upgrade CI actions (`checkout`, `cache`, `download-artifact`,
  `upload-artifact`).

## [0.2.3] - 2024-04-13

### Fixed

- Allow larger values for `gpu_energy` and `cpu_energy` in plist parsing.

## [0.2.2] - 2024-03-18

### Added

- Configurable history size.
- History colors.

## [0.2.1] - 2024-02-24

### Changed

- Upgrade to termion 3.0.0.

## [0.2.0] - 2024-02-17

### Added

- Memory tab with memory usage history on the overview.
- Tab and BackTab keyboard navigation.

## [0.1.2] - 2023-09-06

### Fixed

- Remove nerdfont symbol (󱐋) for broader terminal compatibility.

## [0.1.1] - 2023-08-09

### Fixed

- Better CPU alignment and layout logic.

## [0.1.0] - 2023-08-03

### Added

- JSON streaming mode (`pumas run --json`).

## [0.0.11] - 2023-08-03

### Added

- GPU tab with frequency table reporting.

## [0.0.10] - 2023-08-02

### Added

- CPU per-core usage and frequency display.

## [0.0.9] - 2023-08-01

### Added

- Thermal block moved next to package block for better readability.

### Fixed

- Better gauge labels for GPU and ANE.

## [0.0.8] - 2023-07-25

### Added

- CPU peak percentage and power display.
- GPU and ANE peak percentage and power display.

## [0.0.7] - 2023-07-23

### Added

- Customizable colors.

## [0.0.6] - 2023-07-23

### Added

- M2 max power estimates.

### Fixed

- Correct number of CPU cluster blocks.
- Patch powermetrics CPU usage with sysinfo's more accurate values.

### Changed

- Restructure modules (`src/parsers` → `src/modules`).

## [0.0.5] - 2023-05-29

### Changed

- Improved logo.

## [0.0.4] - 2023-04-10

### Added

- Thermal pressure block on the overview tab.
- Small margin on top of sparklines.

## [0.0.3] - 2023-04-09

### Added

- Sparklines on the overview tab.

## [0.0.2] - 2023-04-08

Initial packaged release.

## [0.0.1] - 2023-04-08

Initial release.

[0.5.0]: https://github.com/graelo/pumas/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/graelo/pumas/compare/v0.3.4...v0.4.0
[0.3.4]: https://github.com/graelo/pumas/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/graelo/pumas/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/graelo/pumas/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/graelo/pumas/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/graelo/pumas/compare/v0.2.5...v0.3.0
[0.2.5]: https://github.com/graelo/pumas/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/graelo/pumas/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/graelo/pumas/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/graelo/pumas/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/graelo/pumas/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/graelo/pumas/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/graelo/pumas/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/graelo/pumas/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/graelo/pumas/compare/v0.0.11...v0.1.0
[0.0.11]: https://github.com/graelo/pumas/compare/v0.0.10...v0.0.11
[0.0.10]: https://github.com/graelo/pumas/compare/v0.0.9...v0.0.10
[0.0.9]: https://github.com/graelo/pumas/compare/v0.0.8...v0.0.9
[0.0.8]: https://github.com/graelo/pumas/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/graelo/pumas/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/graelo/pumas/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/graelo/pumas/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/graelo/pumas/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/graelo/pumas/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/graelo/pumas/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/graelo/pumas/releases/tag/v0.0.1
