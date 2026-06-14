---
title: Installation
---

# Installation

Add `llama-crab` to a Rust project when you want to load GGUF models directly
from application code.

```toml title="Cargo.toml"
[dependencies]
llama-crab = "0.1"
```

## Native requirements

`llama-crab-sys` builds the native llama.cpp stack. Install the platform C++
toolchain and CMake before the first `cargo build`.

| Platform | Minimum setup |
| --- | --- |
| macOS | Xcode Command Line Tools and CMake |
| Debian/Ubuntu | `build-essential` and `cmake` |
| Fedora/RHEL | `gcc`, `gcc-c++`, `cmake`, and `make` |
| Windows | Visual Studio C++ workload and CMake |

## Feature selection

Be explicit about compute backends in applications:

```toml title="Cargo.toml"
[dependencies]
llama-crab = { version = "0.1", default-features = false, features = ["openmp"] }
```

Use [Cargo features](../reference/cargo-features.md) for the complete current
feature matrix.
