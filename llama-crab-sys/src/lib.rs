//! Low-level FFI bindings to `llama.cpp`.
//!
//! Generated at build time via [`bindgen`] over `wrapper.h`, which in turn
//! includes the public C headers of `llama.cpp`, `ggml` and `gguf`.
//!
//! This crate is **unsafe by design**: every public item is a thin
//! `extern "C"` wrapper around a llama.cpp symbol. Use the safe
//! [`llama-crab`](https://docs.rs/llama-crab) crate instead unless
//! you need fine-grained control.
//!
//! ## Features
//!
//! | Feature | Description |
//! |---|---|
//! | `common` | Compile `libcommon.a` (chat templates, JSON schema, OpenAI compat) |
//! | `cuda` | NVIDIA CUDA backend |
//! | `cuda-no-vmm` | CUDA without Virtual Memory Management |
//! | `metal` | Apple Metal (default on macOS aarch64) |
//! | `vulkan` | Vulkan backend |
//! | `rocm` | AMD ROCm/HIP backend |
//! | `openmp` | OpenMP parallel CPU backend (default) |
//! | `dynamic-link` | Link against `libllama` as a shared object |
//! | `system-ggml` | Use GGML from the system instead of the bundled copy |
//! | `mtmd` | Multimodal (vision + audio) helpers |
//! | `llguidance` | `llguidance` sampler (custom C-ABI vtable) |
//! | `dynamic-backends` | Load GGML backends as shared objects at runtime |

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/DominguesM/llama-crab/main/docs/src/assets/logo.png"
)]
#![allow(unknown_lints)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(missing_docs)]
#![allow(clippy::all)]
#![allow(unpredictable_function_pointer_comparisons)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
