# Cargo features

`llama-crab` exposes a rich set of Cargo features so you only compile
what you actually need. The default feature set covers the most
common cases — CPU via OpenMP, plus Metal on Apple Silicon — but
production binaries should always pin the exact feature combination
that matches the target environment.

## Default features

```toml
[dependencies]
llama-crab = "0.1"
```

Expands to:

```toml
features = ["openmp"]
# On `aarch64-apple-darwin`, also enables "metal".
```

This is enough to run every example and most chatbots on a laptop.
For production you almost always want to be explicit:

```toml
[dependencies]
llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
```

## Feature matrix

### Compute backends

| Feature | What it adds | Notes |
| --- | --- | --- |
| `openmp` | CPU backend with OpenMP. | Default. |
| `metal` | Apple GPU backend. | Default on `aarch64-apple-darwin`. |
| `cuda` | NVIDIA CUDA backend. | Mutually exclusive with `cuda-no-vmm`. |
| `cuda-no-vmm` | CUDA without virtual memory management. | Use on systems where CUDA VMM is restricted. |
| `vulkan` | Vulkan / SPIR-V backend. | Works on most GPUs (NVIDIA, AMD, Intel, Apple). |
| `rocm` | AMD ROCm/HIP backend. | Requires a recent ROCm toolchain. |
| `opencl` | OpenCL backend, primarily for Android Adreno and Arm64 devices. | Requires OpenCL headers and an ICD loader. |
| `kleidiai` | KleidiAI CPU kernels for Arm mobile targets. | Pairs with `openmp` or `opencl`. |
| `dynamic-link` | Links llama.cpp as a shared object instead of static. | Cuts build time; requires a prebuilt `libllama.so/.dylib/.dll`. |
| `dynamic-backends` | Loads GGML backends dynamically. | Useful for plugin architectures. |
| `system-ggml` | Uses a system GGML installation instead of the bundled copy. | Skips the GGML build step. |

### Optional subsystems

| Feature | What it adds |
| --- | --- |
| `mtmd` | Multimodal support through `mtmd.h`; enables image and audio helpers. Required for vision. |
| `common` | Builds llama.cpp's `common` utilities used by chat and grammar helpers. Required for JSON-Schema → GBNF and the `grammar` sampler. |
| `llguidance` | Enables the [`llguidance`](https://github.com/microsoft/llguidance) sampler integration. Faster and more flexible than the GBNF sampler for complex grammars. |
| `hf-tokenizer` | Enables Hugging Face `tokenizers` crate integration. Use when you load a model from a `tokenizer.json` instead of the GGUF-embedded tokenizer. |
| `disk-cache` | Enables the persistent `sled`-backed prompt cache. |

### Mobile / Android-only

| Feature | What it adds |
| --- | --- |
| `shared-stdcxx` | Uses `c++_shared` for Android builds. |
| `static-stdcxx` | Uses `c++_static` for Android builds (the historical default). |

These two are **mutually exclusive**. If neither is set, Android keeps
the legacy `c++_static` behaviour.

## Recommended combinations

=== "macOS laptop (Apple Silicon)"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
    ```

=== "Linux server with NVIDIA H100"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
    ```

=== "Linux server with AMD MI300X"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["rocm", "openmp"] }
    ```

=== "iOS app"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["metal"] }
    ```

    Build with the dedicated profile:

    ```bash
    cargo build --profile release-perf --target aarch64-apple-ios \
        --no-default-features --features metal
    ```

=== "Android phone (Snapdragon / Adreno)"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["openmp", "kleidiai", "shared-stdcxx"] }
    ```

    Build with the size-optimised profile:

    ```bash
    cargo build --profile release-size --target aarch64-linux-android \
        --no-default-features --features openmp,kleidiai,shared-stdcxx
    ```

=== "Vision-language workload"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp", "mtmd"] }
    ```

## Detecting which features are active

The compiled `LlamaBackend` exposes a few capability probes you can
call at runtime:

```rust
use llama_crab::LlamaBackend;

let backend = LlamaBackend::init()?;
println!("GPU offload : {}", backend.supports_gpu_offload());
println!("mmap        : {}", backend.supports_mmap());
println!("mlock       : {}", backend.supports_mlock());
println!("RPC         : {}", backend.supports_rpc());
```

These are particularly useful for diagnostics in binaries that ship to
multiple targets.

## What about default features in CI?

CI pins the feature combinations it actually exercises:

| CI matrix row | Features |
| --- | --- |
| `linux-cpu`     | `openmp` |
| `linux-cuda`    | `cuda`, `openmp` |
| `linux-vulkan`  | `vulkan`, `openmp` |
| `linux-rocm`    | `rocm`, `openmp` |
| `macos-metal`   | `metal`, `openmp` |
| `macos-cpu`     | `openmp` |
| `windows-cpu`   | `openmp` |

This guarantees that the code paths each backend exposes keep working
release after release. If you want a backend to be officially
supported in CI, open an issue and propose the matrix addition.

## Where to next?

- [Mobile distribution](../guides/mobile.md) — the iOS / Android
  recipes and the `MobilePreset` defaults.
- [Backends & GPU offload](../guides/backends.md) — how to pick a
  backend and how `n_gpu_layers` works.
- [Cargo features reference](../reference/cargo-features.md) — the
  same table, with the long-form description of each feature.
