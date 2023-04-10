# `pumas`

[![crate](https://img.shields.io/crates/v/pumas.svg)](https://crates.io/crates/pumas)
[![documentation](https://docs.rs/pumas/badge.svg)](https://docs.rs/pumas)
[![minimum rustc 1.64](https://img.shields.io/badge/rustc-1.64+-red.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![rust 2021 edition](https://img.shields.io/badge/edition-2021-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![build status](https://github.com/graelo/pumas/actions/workflows/essentials.yml/badge.svg)](https://github.com/graelo/pumas/actions/workflows/essentials.yml)

![logo](./images/pumas-logo.svg)

<!-- cargo-sync-readme start -->

A power usage monitor for Apple Silicon.

Version requirement: _rustc 1.64+_

## Features

![Screenshot](./images/screenshot.png)

This is a work in progress.

Note: because this leverages the macOS `powermetrics` command, this only works on macOS running
on Apple Silicon and requires you to run it with `sudo`:

```sh
sudo pumas run
```

## License

Licensed under the [MIT License].

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the MIT license, shall
be licensed as MIT, without any additional terms or conditions.

[MIT license]: http://opensource.org/licenses/MIT

<!-- cargo-sync-readme end -->
