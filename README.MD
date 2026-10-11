# wslpluginapi-sys

[![Crates.io](https://img.shields.io/crates/v/wslpluginapi-sys?logo=rust)](https://crates.io/crates/wslpluginapi-sys)  
[![Docs.rs](https://img.shields.io/badge/docs.rs-wslpluginapi--sys-blue?logo=docs.rs)](https://docs.rs/wslpluginapi-sys)  
[![Build Status](https://github.com/mveril/wslpluginapi-sys/actions/workflows/ci.yml/badge.svg?logo=github)](https://github.com/mveril/wslpluginapi-sys/actions)  
[![License](https://img.shields.io/badge/license-MIT-blue.svg?logo=license)](LICENSE-MIT)  
[![Native API version](https://img.shields.io/badge/Microsoft.WSL.PluginApi-2.9.3-blue?logo=nuget)](https://www.nuget.org/packages/Microsoft.WSL.PluginApi/2.9.3)  
[![Platform](https://img.shields.io/badge/platform-Windows-blue?logo=windows&logoColor=white)](#)

`wslpluginapi-sys` is a Rust crate that provides low-level bindings to the Windows Subsystem for Linux (WSL) Plugin API. It offers a direct interface to the functions and structures defined in the WSL Plugin API for developing WSL plugins in Rust. The bindings target Microsoft.WSL.PluginApi 2.9.3.

## Features

- **API bindings**: Provides bindings to the WSL Plugin API, including structures such as `WSLPluginAPIV1` and `WSLPluginHooksV1`.  
- **Unsafe Abstractions**: Direct, unsafe bindings closely mirroring the original C API for maximum control and flexibility.

## Prerequisites

Before using `wslpluginapi-sys`, ensure you have the following installed:
- **Rust**: Latest stable version.


## Installation

Add `wslpluginapi-sys` to your `Cargo.toml`:

```toml
[dependencies]
wslpluginapi-sys = "0.3.0"
```

## Safety

This crate provides unsafe bindings that closely follow the original C API. Users must ensure they uphold the necessary safety invariants when interacting with these bindings. Proper handling of pointers, memory management, and adherence to the API's expected usage patterns are crucial.

## NuGet package dependency

The bindings in this crate are generated from Microsoft's [Microsoft.WSL.PluginApi](https://www.nuget.org/packages/Microsoft.WSL.PluginApi) NuGet package. The generated bindings are included in this repository, so building the crate does not require the NuGet package or Windows API headers.

## xtask bindgen

To update the bindings, change the package version metadata in the workspace version (the part after `+`) to the target Microsoft.WSL.PluginApi version, then run `cargo xtask bindgen`. The xtask downloads the NuGet package and runs bindgen on its header to generate the Rust bindings. Commit the generated files in the `build` directory along with the metadata and checksum files. At build time, the crate uses that metadata to select the bindings and verify their checksum and API version.

### Requirements

To run this command, you need:
- LLVM
- `rustfmt` (available through rustup)
- The NuGet CLI
## License

This project is licensed under either MIT or Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE` for details.

Note: This crate is part of the [WSLPlugins-rs](https://github.com/mveril/wslplugins-rs) project, which aims to create an idiomatic Rust framework for developing WSL plugins.
