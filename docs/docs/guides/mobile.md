---
title: Mobile
---

# Mobile

Mobile deployments are constrained by memory, thermals, storage, and startup
time. Prefer small GGUFs, explicit context sizes, and the built-in mobile
presets before hand-tuning every parameter.

## Presets

```rust
use llama_crab::{LlamaParams, MobilePreset};

let params = LlamaParams::new("models/model.gguf")
    .with_mobile_preset(MobilePreset::LowRam);
```

| Preset | Use |
| --- | --- |
| `LowRam` | CPU-only, smaller batches, `n_ctx = 2048`. |
| `Balanced` | Interactive defaults with moderate GPU offload and flash attention. |
| `GpuMax` | More GPU offload and larger batches for capable devices. |

Call explicit setters after the preset to override individual values.

## Server CLI

The HTTP server accepts equivalent names:

```bash
llama-crab-server \
  --model models/model.gguf \
  --mobile-preset low-ram
```

Accepted values are `low-ram`, `balanced`, and `gpu-max`.

## Practical defaults

For a first mobile test:

- Start with a small `Q4_K_M` or smaller quantized GGUF.
- Use `n_ctx = 2048` until prompts require more.
- Keep batch sizes conservative.
- Prefer memory mapping when the platform supports it.
- Add GPU offload only after CPU-only load and generation work.

## Multimodal on mobile

Vision models need both the text GGUF and projector file, and image tokens add
context pressure. Start the server with the matching projector:

```bash
llama-crab-server \
  --model models/LFM2.5-VL-1.6B-Q4_K_M.gguf \
  --mmproj models/mmproj.gguf \
  --mobile-preset low-ram
```

For app integration, validate one image size and one prompt shape first, then
increase context or image complexity only when needed.
