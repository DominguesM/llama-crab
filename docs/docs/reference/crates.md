---
title: Crates and packages
---

# Crates and packages

Use the package that matches the integration surface in your application.

## Rust crates

| Crate | Role |
| --- | --- |
| `llama-crab` | Safe high-level Rust API for model loading, completion, chat, embeddings, multimodal helpers, sampling, and structured output. |
| `llama-crab-sys` | Low-level FFI and native build crate for llama.cpp, GGML, GGUF, and mtmd bindings. |
| `llama-crab-server` | HTTP server exposing local model inference through OpenAI-compatible routes and extras. |
| `tauri-plugin-llama-crab` | Tauri v2 command plugin that wraps local model state for desktop applications. |

## TypeScript packages

| Package | Role |
| --- | --- |
| `@llama-crab/core` | Shared TypeScript contracts and client helpers. |
| `@llama-crab/tauri` | Tauri adapter that uses the plugin commands from frontend code. |

## Generated references

- Rust API reference is published under `/api/rust/`.
- TypeScript API reference is published under `/api/typescript/`.

Repository layout and generated-doc commands for contributors are covered in
[Development](../contributing/development.md).
