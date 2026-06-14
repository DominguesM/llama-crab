---
title: Development
---

# Development

This page is for contributors and for applications that intentionally consume a
local source checkout. User-facing install and runtime guides avoid repository
paths and wrapper scripts.

## Local Rust dependency

Use a path dependency only while developing against this repository:

```toml
[dependencies]
llama-crab = { path = "../llama-crab/crates/llama-crab" }
```

For normal applications, prefer the published crate:

```toml
[dependencies]
llama-crab = "0.1.4"
```

## Repository package map

| Path | Package |
| --- | --- |
| `crates/llama-crab` | High-level Rust API. |
| `crates/llama-crab-sys` | Low-level llama.cpp, GGML, GGUF, and mtmd bindings. |
| `crates/llama-crab-server` | HTTP server binary and OpenAI-compatible routes. |
| `crates/tauri-plugin-llama-crab` | Tauri v2 plugin. |
| `packages/core` | Shared TypeScript contracts and helpers. |
| `packages/tauri` | TypeScript Tauri client. |
| `docs` | Docusaurus documentation site. |

## Server from checkout

The user documentation starts from the installed binary:

```bash
cargo install llama-crab-server --features mtmd --force
llama-crab-server --model /path/to/model.gguf
```

From a repository checkout, run the same server through Cargo:

```bash
cargo run -p llama-crab-server -- --model /path/to/model.gguf
```

For multimodal server work, enable the feature explicitly:

```bash
cargo run -p llama-crab-server --features mtmd -- \
  --model /models/vision.gguf \
  --mmproj /models/mmproj.gguf
```

## Example repository

Runnable examples live in
[`llama-crab-examples`](https://github.com/DominguesM/llama-crab-examples),
not in this source repository. Use that repository when validating example
workflows locally:

```bash
git clone https://github.com/DominguesM/llama-crab-examples
cd llama-crab-examples
./run.sh quickstart
```

The wrapper resolves the model target, calls `./scripts/download_models.sh`,
then runs the right binary in release mode.

Useful targets include:

| Area | Targets |
| --- | --- |
| Text and chat | `quickstart`, `simple`, `streaming`, `chat`, `stateful_chat`, `speculative` |
| Embeddings and ranking | `embeddings`, `embedding_search`, `reranker`, `rerank` |
| Multimodal | `vision`, `mtmd`, `lfm_vl`, `multimodal_http` |
| Server | `server_lfm`, `multimodal_http`, `rerank` |
| Structured output and tools | `structured`, `tools`, `tool_calls_qwen` |

Without arguments, the wrapper prints the available example names:

```bash
./run.sh
```

Downloaded model files are stored in `./models/`. If the file is already
present, download is skipped. To skip download checks entirely:

```bash
LLAMA_CRAB_SKIP_DOWNLOAD=1 ./run.sh quickstart
```

You can run example binaries directly once model files exist:

```bash
cargo run --release --bin run_quickstart -- models/qwen2.5-0.5b-instruct-q4_k_m.gguf
cargo run --release --bin run_streaming -- models/qwen2.5-0.5b-instruct-q4_k_m.gguf
```

Vision examples take the text model, projector, image path, and optional prompt.
See [Source examples](../examples/index.md) for the detailed example pages.

## Model download helper

Use the example repository helper when you need the same model names as the
examples:

```bash
./scripts/download_models.sh smol
./scripts/download_models.sh bge
./scripts/download_models.sh lfm-vl
```

## TypeScript package work

From the repository root:

```bash
pnpm --filter @llama-crab/core typecheck
pnpm --filter @llama-crab/core test
pnpm --filter @llama-crab/tauri typecheck
pnpm --filter @llama-crab/tauri test
```

The Tauri package builds `@llama-crab/core` first in its `build`, `typecheck`,
and `test` scripts.

## Validation

Common validation commands:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
pnpm typecheck
pnpm test
```

For docs changes:

```bash
pnpm docs:build
```

The docs build must:

1. Generate Rust API HTML for the public Rust crates.
2. Copy `target/doc` into `docs/static/api/rust`.
3. Generate TypeScript API Markdown with TypeDoc.
4. Build the Docusaurus static site into `docs/build`.

Use narrower package or crate commands while iterating, then run the broader
checks before publishing or opening a PR.
