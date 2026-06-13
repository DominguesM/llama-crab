# Getting started

## Install

```toml
# Cargo.toml
[dependencies]
llama-crab = "0.1"
```

The default features enable OpenMP + Metal (macOS). To pick a different
backend, disable the defaults and re-enable what you need:

```toml
[dependencies]
llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
```

### Available features

| Feature            | What it does                                    |
| ------------------ | ----------------------------------------------- |
| `openmp`           | CPU parallel backend                            |
| `metal`            | Apple GPU (default on macOS aarch64)            |
| `cuda`             | NVIDIA CUDA                                     |
| `vulkan`           | Vulkan / SPIR-V                                 |
| `rocm`             | AMD ROCm/HIP                                    |
| `mtmd`             | Vision + audio (multimodal) support             |
| `llguidance`       | `llguidance` grammar sampler                    |
| `hf-tokenizer`     | HuggingFace `tokenizers` crate integration      |
| `disk-cache`       | `sled`-backed persistent KV cache               |
| `dynamic-link`     | Link llama.cpp as a shared object               |
| `dynamic-backends` | Load GGML backends as shared objects            |
| `system-ggml`      | Use the system GGML instead of the bundled copy |

## First program

```rust,no_run
use llama_crab::{Llama, LlamaParams, Role, high_level::chat_completion::ChatMessage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load the model.
    let mut llama = Llama::load(
        LlamaParams::new("models/llama-3.1-8b-instruct-q4_k_m.gguf")
            .with_n_ctx(4096)
            .with_n_gpu_layers(99),
    )?;

    // 2. Plain text completion.
    let resp = llama.create_completion("The capital of France is", 24)?;
    println!("{}", resp.text);

    // 3. Chat completion (uses the plain template by default; pick a
    //    specific one via `create_chat_completion_with`).
    let history = vec![
        ChatMessage::new(Role::System, "You are a concise assistant."),
        ChatMessage::new(Role::User, "What is Rust?"),
    ];
    let resp = llama.create_chat_completion(&history, 128)?;
    println!("assistant> {}", resp.content);

    Ok(())
}
```

## Where to next?

- [Sampling guide](./sampling.md) — choose a sampler chain.
- [Chat & tools](./chat.md) — chat templates, tool calling.
- [Multimodal](./multimodal.md) — vision-language models.
- [JSON-Schema & grammars](./grammars.md) — constrain the output.
- [Examples](../examples/index.html) — runnable programs.
