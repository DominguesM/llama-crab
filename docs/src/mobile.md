# Mobile Distribution

`llama-crab` does not ship prebuilt mobile artifacts. Build the Rust crate for
your target and bundle the produced library or binary with your app.

## Profiles

The workspace defines two release profiles for packaging:

| Profile | Use case |
| ------- | -------- |
| `release-perf` | Maximum runtime performance with thin LTO. |
| `release-size` | Smaller artifacts with fat LTO, symbol stripping, and `panic = "abort"`. |

Examples:

```bash
cargo build --profile release-perf
cargo build --profile release-size
```

## iOS

Use the `metal` feature for Apple GPU offload:

```bash
cargo build --profile release-perf --target aarch64-apple-ios --no-default-features --features metal
```

For smaller CPU-only artifacts, disable default features and use
`release-size`. Keep model files outside the binary and load them from app
storage.

## Android CPU

For CPU-first Android builds, start with OpenMP and KleidiAI:

```bash
cargo build --profile release-size --target aarch64-linux-android --no-default-features --features openmp,kleidiai
```

`static-stdcxx` and `shared-stdcxx` select the Android C++ runtime. They are
mutually exclusive. If neither feature is set, the build keeps the historical
`c++_static` default.

## Android OpenCL

For Adreno-oriented OpenCL builds, install OpenCL headers and an ICD loader for
the target NDK. Then build with OpenCL enabled:

```bash
cargo build --profile release-perf --target aarch64-linux-android --no-default-features --features opencl,shared-stdcxx
```

The build forwards these environment variables to CMake when set:

| Variable | Purpose |
| -------- | ------- |
| `OpenCL_LIBRARY` | Path to the OpenCL library. |
| `OPENCL_HEADERS_DIR` | Path to OpenCL headers. |
| `OPENCL_ICD_LOADER_HEADERS_DIR` | Path used when building an ICD loader. |

OpenCL and KleidiAI require target SDKs and device validation. The default CI
only checks Cargo feature wiring with `dynamic-link`; it does not prove that a
particular Android SDK, driver, or device can run the backend.

## Runtime Presets

Use `MobilePreset` for a compact starting point and override only the values
your app needs:

```rust,no_run
use llama_crab::{Llama, LlamaParams, MobilePreset};

let mut llama = Llama::load(
    LlamaParams::new("model.gguf")
        .with_mobile_preset(MobilePreset::LowRam)
        .with_n_ctx(1024),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The server exposes the same presets through `--mobile-preset low-ram`,
`--mobile-preset balanced`, and `--mobile-preset gpu-max`.
