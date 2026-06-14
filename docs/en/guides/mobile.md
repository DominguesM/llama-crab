# Mobile distribution

`llama-crab` does not ship prebuilt mobile artifacts — you build the
Rust crate for your target and bundle the produced library or binary
with your app. This page covers the release profiles, the iOS and
Android recipes, the OpenCL + NDK integration, and the runtime
presets.

## Release profiles

The workspace defines two release profiles for mobile packaging:

| Profile | Use case | Trade-offs |
| --- | --- | --- |
| `release-perf` | Maximum runtime performance. | Larger binary, longer link time, thin LTO. |
| `release-size` | Smaller artifact with fat LTO, symbol stripping, `panic = "abort"`. | Best for store-distributed apps. |

```bash
cargo build --profile release-perf
cargo build --profile release-size
```

## iOS

Use the `metal` feature for Apple GPU offload:

```bash
cargo build --profile release-perf \
    --target aarch64-apple-ios \
    --no-default-features --features metal
```

For smaller CPU-only artifacts, disable default features and use
`release-size`:

```bash
cargo build --profile release-size \
    --target aarch64-apple-ios-sim \
    --no-default-features --features openmp
```

Keep model files outside the binary and load them from app storage —
shipping a 4 GB GGUF inside the `.ipa` is rarely a good idea.

## Android CPU

For CPU-first Android builds, start with OpenMP and KleidiAI:

```bash
cargo build --profile release-size \
    --target aarch64-linux-android \
    --no-default-features --features openmp,kleidiai
```

`static-stdcxx` and `shared-stdcxx` select the Android C++ runtime.
They are **mutually exclusive**. If neither feature is set, the
build keeps the historical `c++_static` default.

## Android OpenCL

For Adreno-oriented OpenCL builds, install OpenCL headers and an ICD
loader for the target NDK. Then build with OpenCL enabled:

```bash
cargo build --profile release-perf \
    --target aarch64-linux-android \
    --no-default-features --features opencl,shared-stdcxx
```

The build forwards these environment variables to CMake when set:

| Variable | Purpose |
| --- | --- |
| `OpenCL_LIBRARY` | Path to the OpenCL library. |
| `OPENCL_HEADERS_DIR` | Path to OpenCL headers. |
| `OPENCL_ICD_LOADER_HEADERS_DIR` | Path used when building an ICD loader. |

OpenCL and KleidiAI require target SDKs and device validation. The
default CI only checks Cargo feature wiring with `dynamic-link`; it
does **not** prove that a particular Android SDK, driver, or device
can run the backend.

## Runtime presets

The high-level API exposes [`MobilePreset`], a compact set of
defaults tuned for the most common mobile scenarios:

| Preset | When to use it |
| --- | --- |
| `LowRam` | Old phones, Android Go, watches. 1–2 GB of free RAM. |
| `Balanced` | Modern phones, 4 GB+ of free RAM. |
| `GpuMax` | Devices with a fast GPU (Adreno 7xx+, Apple A-series). |

```rust
use llama_crab::{Llama, LlamaParams, MobilePreset};

let mut llama = Llama::load(
    LlamaParams::new("model.gguf")
        .with_mobile_preset(MobilePreset::Balanced)
        .with_n_ctx(2048),
)?;
```

Call explicit setters after `with_mobile_preset` when you need to
override an individual value:

```rust
use llama_crab::{Llama, LlamaParams, MobilePreset};

let mut llama = Llama::load(
    LlamaParams::new("model.gguf")
        .with_mobile_preset(MobilePreset::LowRam)
        .with_n_ctx(1024)        // override the preset's n_ctx
        .with_n_threads(2),      // override the preset's thread count
)?;
```

The server exposes the same presets through `--mobile-preset low-ram`,
`--mobile-preset balanced`, and `--mobile-preset gpu-max`.

## Packaging checklist

A short checklist for shipping a `llama-crab` binary inside a mobile
app:

- [ ] Pick the right target triple (`aarch64-apple-ios`,
      `aarch64-linux-android`, …).
- [ ] Pick a release profile (`release-perf` for power users,
      `release-size` for the App Store).
- [ ] Bundle the GGUF as a downloadable asset, not a baked-in
      resource.
- [ ] Add a runtime "download model" UI — models are big.
- [ ] Pre-warm the model on a background thread so the first user
      prompt doesn't pay the load cost.
- [ ] Watch memory; on Android, trigger a GC after model load.
- [ ] On iOS, declare the Privacy manifest for any data the app
      sends to the model (the inference itself stays on-device).

## Common pitfalls

| Pitfall | Fix |
| --- | --- |
| Linker error: `cannot find -lomp` | Enable the `openmp` feature and link against the OpenMP runtime available on the target. |
| Linker error: `cannot find -lOpenCL` | Install OpenCL headers and an ICD loader into the NDK sysroot, or use the `OPENCL_*` CMake variables. |
| App rejected by the App Store for "excessive binary size" | Use `release-size` and the `openmp` / `opencl` features only. |
| Crashes during model load on Android Go | Drop the model size or use `MobilePreset::LowRam` with a smaller `n_ctx`. |
| First token takes 5+ seconds | Pre-warm the model on a background thread at app start. |

## Where to next?

- [Backends & GPU offload](backends.md) — pick the right backend for
  the device.
- [Cargo features](../getting-started/cargo-features.md) — the full
  set of feature flags.
- [Server](../server/index.md) — when you want a separate process
  to host the model.

[`MobilePreset`]: https://docs.rs/llama-crab/latest/llama_crab/enum.MobilePreset.html
