---
title: Examples
---

# Examples

The `examples/` directory contains standalone Cargo packages that demonstrate
the public Rust APIs. Prefer the wrapper first:

```bash
./examples/run.sh quickstart
```

It resolves the model target, calls `./scripts/download_models.sh`, then runs the
right binary in release mode.

## Example groups

| Group | Examples |
| --- | --- |
| Text and chat | `quickstart`, `simple`, `streaming`, `chat`, `stateful_chat`, `speculative` |
| Embeddings and ranking | `embeddings`, `embedding_search`, `reranker`, `rerank` |
| Multimodal | `vision`, `mtmd`, `lfm_vl`, `multimodal_http` |
| Server | `server_lfm`, `multimodal_http`, `rerank` |
| Structured and tools | `structured`, `tools`, `tool_calls_qwen` |

Without arguments, the wrapper prints the available example names:

```bash
./examples/run.sh
```

## Model storage

Downloaded files are stored in `./models/`. If the file is already present,
download is skipped. To skip download checks entirely:

```bash
LLAMA_CRAB_SKIP_DOWNLOAD=1 ./examples/run.sh quickstart
```

## Direct Cargo runs

You can run binaries directly once the model files exist:

```bash
cargo run --release --bin run_quickstart -- models/qwen2.5-0.5b-instruct-q4_k_m.gguf
cargo run --release --bin run_streaming -- models/qwen2.5-0.5b-instruct-q4_k_m.gguf
```

Vision examples take the text model, projector, image path, and optional prompt.
