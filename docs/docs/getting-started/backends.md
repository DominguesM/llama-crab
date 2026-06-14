---
title: Backends
---

# Backends

Backends are selected at compile time with Cargo features. The safe Rust API
then exposes runtime parameters such as context size, thread count, and GPU
offload layers.

Common combinations:

| Target | Features |
| --- | --- |
| CPU-only | `openmp` |
| Apple Silicon | `metal`, `openmp` |
| NVIDIA Linux | `cuda`, `openmp` |
| AMD Linux | `rocm`, `openmp` |
| Cross-vendor GPU | `vulkan`, `openmp` |
| Vision workloads | backend features plus `mtmd` |

Example:

```toml
[dependencies]
llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
```

When debugging docs.rs or Linux CI builds, verify transitive default features on
`llama-crab-sys`. A local manifest fix is not proof that a published crate
tarball has the same dependency shape.
