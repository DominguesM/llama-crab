---
title: Examples
---

# Examples

Runnable examples live in the separate
[`llama-crab-examples`](https://github.com/DominguesM/llama-crab-examples)
repository. Clone it when you want executable examples that depend on the
published crates and npm packages:

```bash
git clone https://github.com/DominguesM/llama-crab-examples
cd llama-crab-examples
./run.sh quickstart
```

The wrapper resolves the model target, calls `./scripts/download_models.sh`,
then runs the right binary in release mode.

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
./run.sh
```

## Model storage

Downloaded files are stored in `./models/`. If the file is already present,
download is skipped. To skip download checks entirely:

```bash
LLAMA_CRAB_SKIP_DOWNLOAD=1 ./run.sh quickstart
```

## Direct Cargo runs

You can run binaries directly once the model files exist:

```bash
cargo run --release --bin run_quickstart -- models/qwen2.5-0.5b-instruct-q4_k_m.gguf
cargo run --release --bin run_streaming -- models/qwen2.5-0.5b-instruct-q4_k_m.gguf
```

Vision examples take the text model, projector, image path, and optional prompt.
