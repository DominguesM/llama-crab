# Reference

## Crate layout

```
llama-crab/
├── llama-crab-sys/      # FFI (bindgen + cmake)
└── llama-crab/          # 100% safe Rust
    ├── backend          # LlamaBackend + NumaStrategy
    ├── model            # LlamaModel + LlamaModelParams
    ├── context          # LlamaContext + params + embeddings + session
    ├── batch            # LlamaBatch
    ├── sampling         # LlamaSampler + SamplerChain (17 strategies)
    ├── chat             # ChatMessage + templates + tool calling
    ├── speculative      # PromptLookupDecoding + speculative_decode
    ├── multimodal       # MtmdContext + MtmdBitmap (feature mtmd)
    ├── cache            # RamCache + DiskCache
    ├── json_schema      # JSON-Schema → GBNF
    ├── high_level       # Llama orchestrator + create_completion
    └── sampling/        # strategies module
```

## Backends

| Backend | Feature | Default? |
|---------|---------|----------|
| CPU (OpenMP)   | `openmp`  | ✅ |
| Apple Metal    | `metal`   | ✅ on macOS aarch64 |
| NVIDIA CUDA    | `cuda`    | – |
| Vulkan         | `vulkan`  | – |
| AMD ROCm/HIP   | `rocm`    | – |

## Cargo features

| Feature           | Description                                       |
|-------------------|---------------------------------------------------|
| `default`         | `["openmp", "metal"]`                             |
| `cuda`            | NVIDIA CUDA backend                               |
| `cuda-no-vmm`     | CUDA without Virtual Memory Management            |
| `vulkan`          | Vulkan / SPIR-V backend                           |
| `rocm`            | AMD ROCm/HIP backend                              |
| `mtmd`            | Vision + audio (multimodal) support               |
| `llguidance`      | `llguidance` grammar sampler                      |
| `hf-tokenizer`    | HuggingFace `tokenizers` integration              |
| `disk-cache`      | `sled`-backed persistent KV cache                 |
| `dynamic-link`    | Link llama.cpp as a shared object                  |
| `dynamic-backends`| Load GGML backends as shared objects              |
| `system-ggml`     | Use the system GGML instead of the bundled copy   |

## MSRV

`1.88.0` — pinned via `rust-toolchain.toml`. Bumping the MSRV is a
breaking change and will be a major version bump.
