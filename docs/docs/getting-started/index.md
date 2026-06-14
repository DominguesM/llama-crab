---
title: Getting started
---

# Getting started

This path gets a local GGUF model running from a normal application setup. Once
the model works, you can use it through the Rust SDK, the HTTP server, Tauri, or
the TypeScript client packages.

## Prerequisites

- Rust 1.88 or newer.
- C and C++ build tools for the target platform.
- CMake.
- A GGUF model file on disk, or a download source for one.

The native library is built from the bundled `llama.cpp` source. First builds
are slower than normal Rust-only builds because CMake compiles the native
backend.

## Recommended first path

1. Install native build tools for your platform.
2. Add the Rust crate or install the server binary.
3. Place a small text GGUF model somewhere predictable, such as `models/`.
4. Run the smallest request path for the surface you want to use.

## Choose an integration surface

| Need | Use |
| --- | --- |
| Embed inference directly in a Rust app | `llama-crab` |
| Expose a local OpenAI-compatible HTTP API | `llama-crab-server` |
| Ship local inference in a Tauri app | `tauri-plugin-llama-crab` plus `@llama-crab/tauri` |
| Share contracts with a TS frontend | `@llama-crab/core` |

Source checkout workflows, repository scripts, and example wrappers are covered
in [Development](../contributing/development.md).
