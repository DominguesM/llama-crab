# `llama-crab`

Safe, ergonomic and complete Rust bindings for [`llama.cpp`](https://github.com/ggml-org/llama.cpp).

This is the main crate of the `llama-crab` workspace. Most applications
should depend on this crate; use [`llama-crab-sys`](../llama-crab-sys) only
when you need direct access to raw llama.cpp symbols.

## Quickstart

```toml
[dependencies]
llama-crab = "0.1"
```

```rust,no_run
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(
        LlamaParams::new("models/model.gguf")
            .with_n_ctx(2048)
            .with_n_gpu_layers(99),
    )?;

    let response = llama.create_completion("The capital of France is", 32)?;
    println!("{}", response.text);

    Ok(())
}
```

## Features

- Text completion, chat completion, infill, embeddings and reranking.
- Sampling chains, grammar-constrained decoding, JSON-Schema to GBNF.
- Tool-call parsing for ChatML, Mistral, Llama 3, Functionary and plain JSON.
- Multimodal support (vision and audio) through `mtmd`.
- Hardware backends for CPU, Metal, CUDA, Vulkan, ROCm, OpenCL and KleidiAI.
- Mobile presets and packaging profiles for Android and iOS.

For the full feature table, backend flags and mobile build details, see the
[user guide](https://dominguesm.github.io/llama-crab-docs/).

## Resources

- [API reference (docs.rs)](https://docs.rs/llama-crab)
- [User guide](https://dominguesm.github.io/llama-crab-docs/)
- [Examples repository](https://github.com/DominguesM/llama-crab-examples)
- [Workspace README](../../README.md)

## License

Licensed under the [MIT License](../../LICENSE-MIT).
