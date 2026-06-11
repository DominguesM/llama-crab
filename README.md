# llama-crab

> **Safe, ergonomic and complete Rust bindings for [llama.cpp](https://github.com/ggml-org/llama.cpp).**
>
> Inspired by [`llama-cpp-rs`](https://github.com/utilityai/llama-cpp-rs) and the feature completeness of [`llama-cpp-python`](https://github.com/abetlen/llama-cpp-python).

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE-APACHE)
[![MSRV: 1.80](https://img.shields.io/badge/MSRV-1.80-blue.svg)](https://blog.rust-lang.org/2024/07/25/Rust-1.80.0.html)

`llama-crab` provides two crates:

| Crate | Purpose |
|---|---|
| `llama-crab-sys` | Low-level, hand-curated FFI over `llama.h`, `ggml.h`, `gguf.h` (and `mtmd.h`) generated via `bindgen` and `cmake`. |
| `llama-crab` | Safe, idiomatic Rust API: `LlamaModel`, `LlamaContext`, sampling chains, chat templates, tool calling, multimodal, speculative decoding, caching, embeddings, reranking. |

## Quickstart

Add to your `Cargo.toml`:

```toml
[dependencies]
llama-crab = "0.1"
```

Load a GGUF model and generate text:

```rust,no_run
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llama = Llama::load(LlamaParams::default()
        .with_model_path("models/llama-3.1-8b-instruct-q4_k_m.gguf")
        .with_n_ctx(2048)
        .with_n_gpu_layers(99))?;

    let response = llama.create_completion("Once upon a time", 64)?;
    println!("{}", response.text);
    Ok(())
}
```

## Feature matrix

| Feature | Status |
|---|---|
| GGUF model loading (mmap, mlock) | ✅ |
| Multi-GPU layer offload (Metal, CUDA, Vulkan, HIP) | ✅ |
| KV cache quantization (Q2_K … Q8_K, IQ\*) | ✅ |
| RoPE scaling (linear, yarn, longrope) | ✅ |
| Flash attention, SWA, MTP | ✅ |
| All sampling strategies (greedy, top-k/p, min-p, typical, xtc, mirostat v1/v2, dry, **adaptive_p**, infill, logit-bias, grammar, …) | ✅ |
| Custom samplers (Rust C-ABI vtable) | ✅ |
| GBNF grammar + JSON schema constrained decoding | ✅ |
| Chat templates (Jinja2 subset + 20+ builtins) | ✅ |
| Tool calling (functionary v1/v2, chatml, hermes, qwen, llama-3) | ✅ |
| Streaming JSON parsers (incremental tool-call deltas) | ✅ |
| Embeddings (mean/cls/last pooling + L2 normalize) | ✅ |
| Reranking (rank pooling) | ✅ |
| FIM infill (PSM/SPM) | ✅ |
| Speculative decoding (prompt-lookup n-gram + custom draft models) | ✅ |
| State save/load (full + per-sequence, with flags) | ✅ |
| Prompt + KV cache (RAM/Disk, prefix-match) | ✅ |
| Multimodal (mtmd): vision + audio chat handlers | ✅ (feature `mtmd`) |
| HF AutoTokenizer (feature `hf-tokenizer`) | ✅ |
| llguidance (feature `llguidance`) | ✅ |
| OpenAI-compatible HTTP server | ⛔ out of v0.1 (planned as `llama-crab-server`) |

## Backends

| Backend | Feature | Default? |
|---|---|---|
| CPU (OpenMP) | `openmp` | ✅ |
| Apple Metal (macOS/iOS) | `metal` | ✅ on macOS aarch64 |
| NVIDIA CUDA | `cuda` | – |
| NVIDIA CUDA (no VMM) | `cuda-no-vmm` | – |
| Vulkan | `vulkan` | – |
| AMD ROCm/HIP | `rocm` | – |
| Dynamic linking | `dynamic-link` | – |
| System GGML | `system-ggml` | – |
| Dynamic backends | `dynamic-backends` | – |

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.
