---
sidebar_position: 1
slug: /
title: llama-crab documentation
---

# llama-crab

`llama-crab` helps you run local GGUF models from applications. It provides a
Rust SDK, an installable HTTP server with OpenAI-compatible routes, a Tauri
integration, and TypeScript client contracts.

Start with the surface you plan to use:

| Audience | Start here |
| --- | --- |
| You want to run a model quickly | [Getting started](getting-started/) |
| You are writing a Rust application | [Rust SDK](rust/) |
| You need an HTTP API | [Server](server/) |
| You are building a Tauri app | [Tauri](tauri/) |
| You consume the TypeScript packages | [TypeScript](typescript/) |
| You are contributing to llama-crab itself | [Contributing](contributing/) |

## Product surfaces

- `llama-crab`: Rust SDK for loading models and running inference in-process.
- `llama-crab-server`: command-line HTTP server for local model serving.
- `tauri-plugin-llama-crab`: local inference inside a Tauri desktop app.
- `@llama-crab/core`: shared TypeScript request and response contracts.
- `@llama-crab/tauri`: TypeScript client for the Tauri plugin.

## API reference

- [Rust API](https://dominguesm.github.io/llama-crab/api/rust/llama_crab/) documents exact Rust signatures.
- [TypeScript API](https://dominguesm.github.io/llama-crab/api/typescript/) is generated with TypeDoc.

The guides explain recommended usage. The generated API references are the
source of truth for exact signatures.
