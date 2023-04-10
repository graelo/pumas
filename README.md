# `pumas`

[![crate](https://img.shields.io/crates/v/pumas.svg)](https://crates.io/crates/pumas)
[![documentation](https://docs.rs/pumas/badge.svg)](https://docs.rs/pumas)
[![minimum rustc 1.64](https://img.shields.io/badge/rustc-1.64+-red.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![rust 2021 edition](https://img.shields.io/badge/edition-2021-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![build status](https://github.com/graelo/pumas/actions/workflows/essentials.yml/badge.svg)](https://github.com/graelo/pumas/actions/workflows/essentials.yml)

![logo](./images/pumas-logo.svg)

<!-- cargo-sync-readme start -->

A nvtop-inspired command line tool for Apple Silicon Macs: aka M1, M2, ... This is basically a
reimplemented version of [asitop] in Rust.

Utilization info:

- CPU (E-cluster and P-cluster), GPU
- Frequency and utilization
- ANE utilization (measured by power)

Memory info:
- RAM and swap, size and usage
- (Apple removed memory bandwidth from powermetrics)

Power info:
- CPU power, GPU power (Apple removed package power from powermetrics)
- Chart for CPU/GPU power
- Peak power, rolling average display

Pumas uses the built-in `powermetrics` utility on macOS, which allows access to a
variety of hardware performance counters. Note that it requires `sudo` to run due
to powermetrics needing root access to run. Pumas is lightweight and has
minimal performance impact.

Pumas only works on Apple Silicon Macs on macOS Monterey and later.

This is a work in progress.

## Quickstart

```sh
sudo pumas run
```

![Screenshot](./images/screenshot.png)

## Usage

```sh
$ pumas --help
A power usage monitor for Apple Silicon.

Usage: pumas <COMMAND>

Commands:
  run                  Run the power usage monitor
  generate-completion  Print a shell completion script to stdout
  help                 Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

and

```sh
$ pumas run --help
Run the power usage monitor

Usage: pumas run [OPTIONS]

Options:
  -i, --sample-rate <SAMPLE_RATE_MS>  Rate at which metrics are sampled and displayed (milliseconds) [default: 1000]
  -c, --color <COLOR>                 Choose display color (0~8) [default: 2]
  -h, --help                          Print help
  -V, --version                       Print version
```

## Details

`powermetrics` is used to measure the following:

  - CPU/GPU utilization via active residency
  - CPU/GPU frequency
  - Package/CPU/GPU/ANE energy consumption

`psutil` is used to measure the following:

  - memory and swap usage

`sysctl` is used to measure the following:

  - CPU name
  - CPU core counts

`system_profiler` is used to measure the following:

  - GPU core count

Some information is guesstimate and hardcoded as there doesn't seem to be a official source for
it on the system:

  - CPU/GPU TDP

## License

Licensed under the [MIT License].

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the MIT license, shall
be licensed as MIT, without any additional terms or conditions.

[MIT license]: http://opensource.org/licenses/MIT
[asitop]: https://github.com/tlkh/asitop

<!-- cargo-sync-readme end -->
