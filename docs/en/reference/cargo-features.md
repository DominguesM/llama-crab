# Cargo features

The canonical list of Cargo features, with the long-form
description of each one. See the [getting started
guide](../getting-started/cargo-features.md) for a shorter, task-
oriented overview.

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

## Compute backends

| Feature | Description |
| --- | --- |
| `openmp` | CPU backend with OpenMP. Enabled by default. |
| `metal` | Apple Metal backend. Enabled by default on `aarch64-apple-darwin`. |
| `cuda` | NVIDIA CUDA backend. |
| `cuda-no-vmm` | NVIDIA CUDA backend without virtual memory management. |
| `vulkan` | Vulkan / SPIR-V backend. |
| `rocm` | AMD ROCm / HIP backend. |
| `opencl` | OpenCL backend, primarily for Android Adreno and Arm64 devices. |
| `kleidiai` | KleidiAI CPU kernels for Arm mobile targets. |
| `dynamic-link` | Links llama.cpp as a shared object instead of static. |
| `dynamic-backends` | Loads GGML backends dynamically. |
| `system-ggml` | Uses a system GGML installation instead of the bundled copy. |

## Optional subsystems

| Feature | Description |
| --- | --- |
| `mtmd` | Multimodal support through `mtmd.h`; enables image and audio helpers. Required for vision. |
| `common` | Builds llama.cpp's `common` utilities used by chat and grammar helpers. Required for JSON-Schema → GBNF and the `grammar` sampler. |
| `llguidance` | Enables the [`llguidance`](https://github.com/microsoft/llguidance) sampler integration. Faster and more flexible than the GBNF sampler for complex grammars. |
| `hf-tokenizer` | Enables Hugging Face `tokenizers` crate integration. Use when you load a model from a `tokenizer.json` instead of the GGUF-embedded tokenizer. |
| `disk-cache` | Enables the persistent `sled`-backed prompt cache. |

## Mobile / Android-only

| Feature | Description |
| --- | --- |
| `shared-stdcxx` | Uses `c++_shared` for Android builds. |
| `static-stdcxx` | Uses `c++_static` for Android builds. The historical default. |

These two are **mutually exclusive**. If neither is set, Android
keeps the legacy `c++_static` behaviour.

## Mutually exclusive groups

| Group | Pick at most one |
| --- | --- |
| CUDA variant | `cuda`, `cuda-no-vmm` |
| Android C++ runtime | `shared-stdcxx`, `static-stdcxx` |

The crate's `build.rs` will fail the build with a clear error if
two mutually exclusive features are enabled together.

## Recommended combinations

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

=== "Android (Snapdragon / Adreno)"

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

If you want a backend to be officially supported in CI, open an
issue and propose the matrix addition.

## Where to next?

- [Cargo features (getting started)](../getting-started/cargo-features.md) —
  the task-oriented overview.
- [Backends & GPU offload](../guides/backends.md) — the runtime
  configuration.
- [Mobile distribution](../guides/mobile.md) — the iOS and Android
  recipes.
