---
title: Package Map
---

# Package map

Use the package that matches your integration surface.

## Rust packages

| Package | Purpose |
| --- | --- |
| `llama-crab` | Safe Rust SDK for applications. Start here for in-process inference. |
| `llama-crab-sys` | Low-level bindings for advanced users who need direct native access. |
| `llama-crab-server` | Installable OpenAI-compatible local HTTP server. |
| `tauri-plugin-llama-crab` | Tauri v2 plugin for desktop applications. |

The generated Rust API reference for the main crate is available at
[`/llama-crab/api/rust/llama_crab/`](https://dominguesm.github.io/llama-crab/api/rust/llama_crab/).

## TypeScript packages

| Package | Purpose |
| --- | --- |
| `@llama-crab/core` | Shared TypeScript request and response contracts. |
| `@llama-crab/tauri` | OpenAI-like client for Tauri apps using the plugin. |

## Public API layers

Use the high-level API first:

- `Llama::load` owns backend, model, and context setup.
- `LlamaParams` configures model path, context size, threads, GPU layers,
  embedding mode, pooling, flash attention, and mobile presets.
- `create_completion`, `create_chat_completion`, and `embed` cover common
  application flows.

Drop to lower-level modules when you need precise control:

- `model` for tokenization, detokenization, and metadata.
- `context` and `batch` for manual decode/encode loops.
- `sampling` for sampler chains.
- `chat` for templates, roles, tools, and rendering.
- `multimodal` for `mtmd` image/projector handling.

Repository layout, source examples, and build profiles are documented in
[Development](../contributing/development.md).
