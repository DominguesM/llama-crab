---
title: Performance
---

# Performance

Start with the smallest model that exercises your workload, then tune one
variable at a time.

```bash
llama-crab-server --model /models/small-chat.gguf
```

## Build in release mode

Use `--release` for realistic inference behavior:

```bash
cargo run --release
```

Debug builds are useful for Rust diagnostics, but llama.cpp inference is much
slower and some backend-sensitive paths are easier to evaluate in release mode.

## Context and batch size

`n_ctx` controls the maximum prompt plus generation window. Larger values use
more memory. `n_batch` and `n_ubatch` control how much work is fed through the
model at once.

```rust
let params = LlamaParams::new("models/model.gguf")
    .with_n_ctx(4096)
    .with_n_batch(512)
    .with_n_ubatch(256);
```

If allocation fails, reduce context first, then batch sizes, then choose a
smaller quantized model.

## CPU threads

Set explicit thread counts when you want reproducible local behavior:

```rust
let params = LlamaParams::new("models/model.gguf")
    .with_n_threads(4)
    .with_n_threads_batch(4);
```

The server exposes the same knobs as `--n-threads` and `--n-batch` style flags.

## GPU offload

`with_n_gpu_layers` controls how many layers are offloaded to the active backend:

```rust
let params = LlamaParams::new("models/model.gguf").with_n_gpu_layers(99);
```

The right value depends on model size, backend, and available VRAM or unified
memory. If startup fails after increasing offload, lower this value before
changing unrelated settings.

## Feature selection

The default crate features enable `openmp` and `metal`. On non-Apple targets or
CI, you may want explicit features:

```toml
llama-crab = { version = "0.1.300", default-features = false, features = ["openmp"] }
```

Use only one primary accelerator backend unless you have a reason to compile
several.

## Measure the right layer

For application tuning, measure:

- model load time,
- prompt token count,
- time to first token,
- generated tokens per second,
- peak memory.

For serious benchmarking, use a dedicated harness with fixed prompts, fixed
sampler settings, and warm caches. Repository examples and wrappers for local
validation are covered in [Development](../contributing/development.md).
