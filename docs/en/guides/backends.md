# Backends & GPU offload

`llama-crab` is built on top of `llama.cpp`, which delegates the
heavy linear-algebra work to a **backend**. The active backend is
chosen at *build time* through Cargo features; you can mix CPU and
GPU work at *runtime* by offloading a chosen number of transformer
layers to the GPU.

## Choosing a backend

| Backend | Cargo feature | Default? | When to pick it |
| --- | --- | --- | --- |
| CPU (OpenMP) | `openmp` | yes | Always on. Lifts CPU inference to multiple cores. |
| Apple Metal | `metal` | yes on `aarch64-apple-darwin` | Apple Silicon. Best perf-per-watt. |
| NVIDIA CUDA | `cuda` | – | Linux + NVIDIA. Best raw throughput on big GPUs. |
| NVIDIA CUDA (no VMM) | `cuda-no-vmm` | – | CUDA without virtual memory management. |
| Vulkan / SPIR-V | `vulkan` | – | Cross-vendor GPU compute. Falls back to CPU gracefully. |
| AMD ROCm / HIP | `rocm` | – | Linux + AMD. |
| OpenCL | `opencl` | – | Android Adreno and Arm64. |
| KleidiAI CPU kernels | `kleidiai` | – | Arm mobile targets. |
| Dynamic linking | `dynamic-link` | – | Link llama.cpp as a shared library. |
| Dynamic backends | `dynamic-backends` | – | Load GGML backends dynamically. |
| System GGML | `system-ggml` | – | Skip the bundled GGML build, use a system one. |

See the [Cargo features reference](../reference/cargo-features.md)
for the canonical list.

### A recommended Cargo.toml

=== "Apple Silicon (macOS)"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
    ```

=== "Linux + NVIDIA"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
    ```

=== "Linux + AMD"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["rocm", "openmp"] }
    ```

=== "Cross-vendor (Vulkan)"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["vulkan", "openmp"] }
    ```

=== "CPU only"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["openmp"] }
    ```

## Initialising the backend

`LlamaBackend::init()` is called automatically when you load a model
through the high-level [`Llama`] orchestrator. If you drive the
lower-level API directly, hold a [`LlamaBackend`] guard for the
entire lifetime of the model — dropping it tears the backend down.

```rust
use llama_crab::{LlamaBackend, NumaStrategy};

// Default backend.
let _backend = LlamaBackend::init()?;

// NUMA-aware initialisation. Distribute, Isolate, or Numactl.
let _backend = LlamaBackend::init_numa(NumaStrategy::Distribute)?;
```

### Capability probes

The backend exposes a handful of capability probes you can call at
runtime to detect what's available:

| Method | What it tells you |
| --- | --- |
| `supports_gpu_offload()` | Any GPU backend (Metal, CUDA, Vulkan, ROCm) is available. |
| `supports_mmap()` | Memory-mapped model loading is available. |
| `supports_mlock()` | `mlock` (pin model in RAM) is available. |
| `supports_rpc()` | Distributed RPC inference is available. |

```rust
let backend = LlamaBackend::init()?;
if backend.supports_gpu_offload() {
    println!("GPU offload is available");
} else {
    println!("CPU only");
}
```

## Layer offload

`LlamaParams::with_n_gpu_layers(n)` controls how many transformer
layers are pushed to the GPU. Pass a large number (`99`) to offload
the whole model; pass `0` to run entirely on CPU.

```rust
use llama_crab::{Llama, LlamaParams};

// Fully offload a small model to the GPU.
let llama = Llama::load(
    LlamaParams::new("model.gguf")
        .with_n_ctx(2048)
        .with_n_gpu_layers(99),
)?;
```

The "offload" knob is a per-layer counter that walks the model from
the input embedding toward the output. Setting it to `N` means "the
first N layers run on the GPU, the remaining `total - N` layers run
on the CPU".

### When to use partial offload

Layer offload is most useful in three regimes:

1. **Model fits on the GPU** — set `n_gpu_layers` to the number of
   layers in the model. All layers run on the GPU; CPU threads are
   idle.
2. **Model is bigger than VRAM** — set `n_gpu_layers` to the largest
   count that fits in VRAM. The tail of the model runs on the CPU,
   and data crosses the PCIe bus between the layers. The throughput
   drop is graceful (typically 2–4× per crossed layer).
3. **CPU-only machines** — set `n_gpu_layers = 0`. The model runs
   entirely on the CPU using OpenMP threads.

### A quick rule of thumb

| Quant size | 8 GB GPU | 16 GB GPU | 24 GB GPU |
| --- | --- | --- | --- |
| 7B Q4_K_M (~4 GB) | 99 layers | 99 layers | 99 layers |
| 13B Q4_K_M (~7.5 GB) | 99 layers | 99 layers | 99 layers |
| 70B Q4_K_M (~40 GB) | 10–15 layers | 20–25 layers | 35–40 layers |

Numbers depend heavily on the model's vocabulary, head size and
context length. Use them as a starting point, then measure with your
own prompt.

## CPU threads

For CPU-only or hybrid runs, control thread counts with:

```rust
use llama_crab::{Llama, LlamaParams};

let llama = Llama::load(
    LlamaParams::new("model.gguf")
        .with_n_threads(8)         // threads for prompt ingestion
        // .with_n_threads_batch(8) // separate thread count for batches
)?;
```

A reasonable starting point is the number of **physical** cores. On
Apple Silicon, the number of *performance* cores is a better target
than the total core count.

## Flash attention

Flash attention is opt-in via `LlamaContextParams::with_flash_attn`:

```rust
use llama_crab::{Llama, LlamaParams};

let llama = Llama::load(
    LlamaParams::new("model.gguf")
        .with_n_ctx(4096)
        .with_n_gpu_layers(99)
        .with_flash_attn(true),
)?;
```

It reduces memory and accelerates long-context inference on most
modern architectures (Gemma, Llama 3, Qwen2.5, …).

## Multi-GPU

`llama-crab` exposes multi-GPU through `llama.cpp`'s layer-split
model. The `LlamaParams` API exposes a single `n_gpu_layers` knob;
for fine-grained splits across multiple devices, drive the
`llama-crab-sys` API directly. The
[`llama.cpp` documentation](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)
covers the underlying mechanisms in detail.

## When the backend cannot be initialised

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| `BackendNotInitialised` at startup | Lower-level API called without `LlamaBackend::init()`. | Hold a `LlamaBackend` guard for the lifetime of the model. |
| Linker error on Metal | `metal` feature not enabled. | Add `features = ["metal"]` to the dependency. |
| Linker error on CUDA | CUDA toolkit not in `PATH`. | Install the CUDA toolkit and ensure `nvcc` is reachable. |
| OpenCL loader not found | `OPENCL_HEADERS_DIR` / `OPENCL_ICD_LOADER_HEADERS_DIR` not set. | See the [Mobile distribution guide](mobile.md). |

## Where to next?

- [Mobile distribution](mobile.md) — the iOS and Android recipes.
- [Sampling strategies](sampling.md) — what to do with the logits
  once the model produces them.
- [Caching & session state](caching.md) — manually persist and
  restore KV state.

[`Llama`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html
[`LlamaBackend`]: https://docs.rs/llama-crab/latest/llama_crab/struct.LlamaBackend.html
