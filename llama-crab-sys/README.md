# llama-crab-sys

Low-level FFI bindings to `llama.cpp`, `ggml`, `gguf` and `mtmd`.

This crate is the unsafe system layer used by
[`llama-crab`](https://crates.io/crates/llama-crab). Most users should
depend on `llama-crab` instead.

The package builds the bundled `llama.cpp` sources with CMake and
generates Rust bindings with `bindgen`.

Licensed under the MIT License.
