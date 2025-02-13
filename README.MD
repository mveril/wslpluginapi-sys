# wslpluginapi-sys

[![Crates.io](https://img.shields.io/crates/v/wslpluginapi-sys?logo=rust)](https://crates.io/crates/wslpluginapi-sys)
[![Docs.rs](https://img.shields.io/badge/docs.rs-wslpluginapi--sys-blue?logo=docs.rs)](https://docs.rs/wslpluginapi-sys)
[![Build Status](https://github.com/mveril/wslpluginapi-sys/actions/workflows/ci.yml/badge.svg?logo=github)](https://github.com/mveril/wslpluginapi-sys/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?logo=license)](LICENSE)
[![Native API version](https://img.shields.io/badge/Microsoft.WSL.PluginApi-2.1.3-blue?logo=nuget)](https://www.nuget.org/packages/Microsoft.WSL.PluginApi/2.1.3)
[![Platform](https://img.shields.io/badge/platform-Windows-blue?logo=windows&logoColor=white)](#)

`wslpluginapi-sys` is a Rust crate that provides low-level bindings to the Windows Subsystem for Linux (WSL) Plugin API. It offers a direct interface to the functions and structures defined in the WSL Plugin API, facilitating the development of WSL plugins in Rust.

## Features

- **Comprehensive Bindings**: Provides complete bindings to the WSL Plugin API, including structures like `GUID` and other essential components.
- **Unsafe Abstractions**: Direct, unsafe bindings closely mirroring the original C API for maximum control and flexibility.

## Prerequisites

Before using `wslpluginapi-sys`, ensure you have the following installed:

- **Rust**: Latest stable version.
- **Nuget**: require nuget cli in the Path or use the `no-nuget` feature to manage the nuget packages via reqwest and zip (useful if nuget is not installed or on non windows environement).

## Installation

Add `wslpluginapi-sys` to your `Cargo.toml`:

```toml
[dependencies]
wslpluginapi-sys = "0.1.0-beta.1+2.1.3"
```

Safety
This crate provides unsafe bindings that closely follow the original C API. Users must ensure they uphold the necessary safety invariants when interacting with these bindings. Proper handling of pointers, memory management, and adherence to the API's expected usage patterns are crucial.

License
This project is licensed under the MIT License. See the LICENSE file for details.

Note: This crate is part of the [WSLPlugin-rs](https://github.com/mveril/wslpluginapi-sys) project, which aims to create an idiomatic Rust framework for developing WSL plugins.
