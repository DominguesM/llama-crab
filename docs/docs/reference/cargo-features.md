---
title: Cargo features
---

# Cargo features

Feature flags let you select native backends and optional capabilities when
adding llama-crab to an application.

## `llama-crab`

Current high-level feature groups:

| Feature | Purpose |
| --- | --- |
| `openmp` | CPU backend through `llama-crab-sys/openmp`. |
| `metal` | Apple Metal backend. |
| `cuda` | NVIDIA CUDA backend. |
| `cuda-no-vmm` | CUDA without virtual memory management. |
| `vulkan` | Vulkan backend. |
| `rocm` | AMD ROCm backend. |
| `dynamic-link` | Dynamic native linking. |
| `dynamic-backends` | Dynamic GGML backend loading. |
| `common` | llama.cpp common utilities used by chat and grammar helpers. |
| `mtmd` | Multimodal support through mtmd. |
| `llguidance` | llguidance sampler integration. |
| `hf-tokenizer` | Hugging Face tokenizer integration. |
| `disk-cache` | Persistent prompt cache support. |

Do not assume platform-specific defaults. Pin the features you need in your
application manifest and confirm them against the generated Rust API reference
or the crate metadata for the release you consume.

## Server features

`llama-crab-server` keeps its default feature set empty. Enable `mtmd` when the
server needs image or audio-capable multimodal requests.
