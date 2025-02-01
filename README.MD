# wslpluginapi-sys

`wslpluginapi-sys` is a Rust crate that provides low-level bindings to the Windows Subsystem for Linux (WSL) Plugin API. It offers a direct interface to the functions and structures defined in the WSL Plugin API, facilitating the development of WSL plugins in Rust.

## Features

- **Comprehensive Bindings**: Provides complete bindings to the WSL Plugin API, including structures like `GUID` and other essential components.
- **Unsafe Abstractions**: Direct, unsafe bindings closely mirroring the original C API for maximum control and flexibility.

## Prerequisites

Before using `wslpluginapi-sys`, ensure you have the following installed:

- **Rust**: Latest stable version.
- **Windows SDK**: Required for access to necessary headers and libraries.
- **Nuget**: require nuget cli in the Windows Path.

## Installation

Add `wslpluginapi-sys` to your `Cargo.toml`:

```toml
[dependencies]
wslpluginapi-sys = "0.1.0"
```

Safety
This crate provides unsafe bindings that closely follow the original C API. Users must ensure they uphold the necessary safety invariants when interacting with these bindings. Proper handling of pointers, memory management, and adherence to the API's expected usage patterns are crucial.

License
This project is licensed under the MIT License. See the LICENSE file for details.

Note: This crate is part of the [WSLPlugin-rs](https://github.com/mveril/wslplugins-rs) project, which aims to create an idiomatic Rust framework for developing WSL plugins.
