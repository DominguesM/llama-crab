# Introduction

**`llama-crab`** is a safe, ergonomic, and complete Rust binding for
[`llama.cpp`](https://github.com/ggml-org/llama.cpp). It exposes two
crates:

| Crate            | Purpose                                                   |
|------------------|-----------------------------------------------------------|
| `llama-crab-sys` | Raw FFI generated via `bindgen` over `wrapper.h` + cmake  |
| `llama-crab`     | 100% safe-Rust API: model loading, sampling, chat, …      |

`llama-crab` is built with three goals in mind:

1. **Ergonomics** — a high-level [`Llama`](https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html) orchestrator that mirrors the surface of `llama-cpp-python`'s `Llama` class, but stays 100% safe Rust and uses idiomatic builders.
2. **Completeness** — every sampling strategy, every chat format and every modern llama.cpp feature is exposed, including `mtmd`-based vision and `llguidance` grammars.
3. **No surprises** — `SemVer` is honored, the build is reproducible (llama.cpp is pinned to a release tag) and CI runs on a matrix of CPU/CUDA/Vulkan/Metal/ROCm combinations.

## What you can build with it

* Local assistants, REPLs, and CLI tools
* Embedding-based retrieval systems
* Speculative-decoding servers
* Vision-language agents (with the `mtmd` feature)
* Tools that emit structured JSON (with the GBNF-grammar sampler)

## Quickstart

```rust,no_run
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(2048))?;
let resp = llama.create_completion("Once upon a time", 64)?;
println!("{}", resp.text);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Continue to [Getting started](./getting-started.md) for the full setup walkthrough.
