---
title: Rust SDK
---

# Rust SDK

`llama-crab` is the safe Rust layer over the bundled `llama.cpp` runtime. Use it
when you want local GGUF inference directly from Rust code: text completion,
chat, embeddings, structured output, tool-style prompts, and multimodal vision
through `mtmd`.

The generated API reference is published at
[`/llama-crab/api/rust/llama_crab/`](https://dominguesm.github.io/llama-crab/api/rust/llama_crab/).

## Install

```toml
[dependencies]
llama-crab = "0.1.300"
```

Use the published crate version that matches your target release.

## Feature flags

The library defaults enable `openmp` and `metal`. You can select a different
backend by changing features:

```toml
[dependencies]
llama-crab = { version = "0.1.300", default-features = false, features = ["openmp"] }
```

Common feature flags:

| Feature | Purpose |
| --- | --- |
| `openmp` | CPU parallelism through the bundled llama.cpp build. |
| `metal` | Apple Metal acceleration. Enabled by default. |
| `cuda`, `vulkan`, `rocm`, `opencl` | Alternative GPU backends forwarded to `llama-crab-sys`. |
| `mtmd` | Multimodal support through llama.cpp `mtmd.h`; also enables image/base64 helpers. |
| `llguidance` | Grammar-guided generation integration. |
| `hf-tokenizer` | Optional Hugging Face tokenizer support. |
| `disk-cache` | Optional on-disk cache support. |

## First program

```rust
use llama_crab::{Llama, LlamaParams};

fn main() -> anyhow::Result<()> {
    let mut llama = Llama::load(
        LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
            .with_n_ctx(2048)
            .with_n_threads(4),
    )?;

    let completion = llama.create_completion("The capital of France is", 16)?;
    println!("{}", completion.text);
    Ok(())
}
```

Run your application in release mode:

```bash
cargo run --release
```

## Where to go next

- [Package map](./workspace.md)
- [Model lifecycle](./lifecycle.md)
- [Text completion](./completion.md)
- [Chat](./chat.md)
- [Embeddings](./embeddings.md)
- [Multimodal](./multimodal.md)
- [Structured output](./structured-output.md)
