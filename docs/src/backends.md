# Backends & GPU offload

`llama-crab` is built on top of `llama.cpp`, which delegates the heavy
linear-algebra work to a **backend**. The active backend is chosen at
*build time* through cargo features. You can mix CPU + GPU work by
offloading a chosen number of transformer layers to the GPU.

## Choosing a backend

| Backend              | Feature             | Default?                  |
| -------------------- | ------------------- | ------------------------- |
| CPU (OpenMP)         | `openmp`            | yes                       |
| Apple Metal          | `metal`             | yes on macOS aarch64      |
| NVIDIA CUDA          | `cuda`              | –                         |
| NVIDIA CUDA (no VMM) | `cuda-no-vmm`       | –                         |
| Vulkan / SPIR-V      | `vulkan`            | –                         |
| AMD ROCm/HIP         | `rocm`              | –                         |
| OpenCL               | `opencl`            | –                         |
| KleidiAI CPU kernels | `kleidiai`          | –                         |
| Dynamic linking      | `dynamic-link`      | –                         |
| System GGML          | `system-ggml`       | –                         |
| Dynamic backends     | `dynamic-backends`  | –                         |

To switch backend, disable the defaults and re-enable what you need:

```toml
[dependencies]
llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
```

## Initializing the backend

`LlamaBackend::init()` is called automatically when you load a model
through the high-level [`Llama`] orchestrator. If you drive the
low-level API directly, hold a [`LlamaBackend`] guard for the entire
lifetime of the model — dropping it tears the backend down.

```rust,no_run
use llama_crab::{LlamaBackend, NumaStrategy};

let _backend = LlamaBackend::init_numa(NumaStrategy::Distribute)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Capability probes are also available on [`LlamaBackend`]:

| Method                      | What it tells you                          |
| --------------------------- | ------------------------------------------ |
| `supports_gpu_offload()`    | Any GPU backend (Metal/CUDA/Vulkan/ROCm).  |
| `supports_mmap()`           | Memory-mapped model loading is available.  |
| `supports_mlock()`          | `mlock` (pin model in RAM) is available.   |
| `supports_rpc()`            | Distributed RPC inference is available.    |

## Layer offload

`LlamaParams::with_n_gpu_layers(n)` controls how many transformer
layers are pushed to the GPU. Pass a large number (`99`) to offload
the whole model; pass `0` to run entirely on CPU.

```rust,no_run
use llama_crab::{Llama, LlamaParams};

// Fully offload a small model to the GPU.
let llama = Llama::load(
    LlamaParams::new("model.gguf")
        .with_n_ctx(2048)
        .with_n_gpu_layers(99),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Layer offload is most useful when:

- the model fits on the GPU (large `n_gpu_layers` → fast),
- the model is bigger than VRAM (partial offload: CPU handles the tail),
- you run on CPU-only machines (`n_gpu_layers = 0`).

## CPU threads

For CPU-only or hybrid runs, control thread counts with:

```rust,no_run
use llama_crab::{Llama, LlamaParams};

let llama = Llama::load(
    LlamaParams::new("model.gguf")
        .with_n_threads(8)        // threads for prompt ingestion
        // .with_n_threads_batch(8) // separate thread count for batches
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

A reasonable starting point is the number of physical cores.

## Mobile builds

Mobile builds use the same Cargo features as desktop builds. Common starting
points are:

```bash
cargo build --profile release-size --no-default-features --features openmp,kleidiai
cargo build --profile release-perf --no-default-features --features metal
cargo build --profile release-perf --no-default-features --features opencl,shared-stdcxx
```

For Android OpenCL, install OpenCL headers and an ICD loader into the NDK
sysroot or provide the standard CMake discovery variables. `OpenCL_LIBRARY`,
`OPENCL_HEADERS_DIR`, and `OPENCL_ICD_LOADER_HEADERS_DIR` are forwarded by the
build script when present. `shared-stdcxx` selects `ANDROID_STL=c++_shared`;
`static-stdcxx` selects `ANDROID_STL=c++_static`. If neither feature is set,
Android keeps the previous `c++_static` default.

The high-level API also provides mobile parameter presets:

```rust,no_run
use llama_crab::{Llama, LlamaParams, MobilePreset};

let llama = Llama::load(
    LlamaParams::new("model.gguf")
        .with_mobile_preset(MobilePreset::Balanced)
        .with_n_ctx(2048),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Call explicit setters after `with_mobile_preset` when you need to override an
individual value.

## Flash attention

Flash attention is opt-in via `LlamaContextParams::with_flash_attn`:

```rust,no_run
use llama_crab::{Llama, LlamaParams};

let llama = Llama::load(
    LlamaParams::new("model.gguf")
        .with_n_ctx(4096)
        .with_n_gpu_layers(99)
        .with_flash_attn(true),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

It reduces memory and accelerates long-context inference on most
modern architectures (Gemma, Llama 3, Qwen2.5, …).

## Where to next?

- [Sampling guide](./sampling.md) — what to do with the logits.
- [Caching & session state](./caching.md) — manually persist and restore KV state.
- [Reference](./reference.md) — full feature matrix.

[`LlamaBackend`]: https://docs.rs/llama-crab/latest/llama_crab/struct.LlamaBackend.html
[`Llama`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html
