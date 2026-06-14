---
title: Model Lifecycle
---

# Model lifecycle

The high-level `Llama` type owns the llama.cpp backend, loaded model, and active
context. In most applications you load one `Llama` per model and keep it alive
for the lifetime of the worker that serves requests.

## Load

```rust
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(
    LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
        .with_n_ctx(2048)
        .with_n_batch(512)
        .with_n_threads(4)
        .with_n_gpu_layers(0),
)?;
```

Useful parameters:

| Setter | Effect |
| --- | --- |
| `with_n_ctx` | Maximum context window for prompt plus generated tokens. |
| `with_n_batch` | Logical maximum batch size. |
| `with_n_ubatch` | Physical batch size used by forward passes. |
| `with_n_threads` | CPU threads for generation. |
| `with_n_threads_batch` | CPU threads for batch work. |
| `with_n_gpu_layers` | Number of model layers to offload to the active GPU backend. |
| `with_use_mmap` | Memory-map the GGUF file. |
| `with_embeddings(true)` | Enable embedding extraction on the context. |
| `with_pooling_type` | Select embedding pooling, for example `PoolingType::Cls`. |
| `with_flash_attn` | Enable flash attention when supported. |

## Mobile presets

`MobilePreset` provides coarse defaults for device classes:

```rust
use llama_crab::{LlamaParams, MobilePreset};

let params = LlamaParams::new("models/model.gguf")
    .with_mobile_preset(MobilePreset::Balanced)
    .with_n_ctx(1024); // explicit setters after the preset override it
```

Available presets are `LowRam`, `Balanced`, and `GpuMax`. The server CLI accepts
the same idea as `--mobile-preset low-ram`, `balanced`, or `gpu-max`.

## Inspect the model

After loading, `llama.model()` exposes metadata and tokenization helpers:

```rust
let tokens = llama.model().tokenize("Hello from Rust", true, false)?;
println!("layers: {}", llama.model().n_layer());
println!("vocab: {}", llama.model().n_vocab());
println!("tokens: {tokens:?}");
```

## Reuse and state

Generation updates the context KV cache. High-level helpers clear or prepare
sequence state for their own flow where appropriate, but long-running services
should still treat one `Llama` instance as mutable state. If multiple requests
must run concurrently, use a worker model where requests are serialized per
loaded model, or load multiple model instances.

The HTTP server follows the worker approach: Axum accepts requests, then sends
inference jobs to a dedicated model thread.
