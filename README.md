# llama-crab

[![Crates.io](https://img.shields.io/crates/v/llama-crab.svg)](https://crates.io/crates/llama-crab)
[![Crates.io Downloads](https://img.shields.io/crates/d/llama-crab.svg)](https://crates.io/crates/llama-crab)
[![Documentation](https://docs.rs/llama-crab/badge.svg)](https://docs.rs/llama-crab)
[![CI](https://github.com/DominguesM/llama-crab/actions/workflows/ci.yml/badge.svg)](https://github.com/DominguesM/llama-crab/actions/workflows/ci.yml)
[![Coverage](https://github.com/DominguesM/llama-crab/actions/workflows/coverage.yml/badge.svg)](https://github.com/DominguesM/llama-crab/actions/workflows/coverage.yml)
[![Release](https://github.com/DominguesM/llama-crab/actions/workflows/release.yml/badge.svg)](https://github.com/DominguesM/llama-crab/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![MSRV: 1.88](https://img.shields.io/badge/MSRV-1.88-blue.svg)](rust-toolchain.toml)
[![llama.cpp: b9623](https://img.shields.io/badge/llama.cpp-b9623-5c5c5c?logo=github)](https://github.com/ggml-org/llama.cpp/commit/341babcf73f198c78aaa39e4b6ab1a84facb01e7)
[![Hugging Face](https://img.shields.io/badge/Hugging%20Face-dominguesm-ffcc4d?logo=https%3A%2F%2Fhuggingface.co%2Fdatasets%2Fhuggingface%2Fbrand-assets%2Fresolve%2Fmain%2Fhf-logo.svg&logoColor=black)](https://huggingface.co/dominguesm)

<p align="center">
  <img
    src="https://gist.githubusercontent.com/DominguesM/127b9e5614e0e2da6b896fb3da3c8f2d/raw/d5dec07e795979f0a1b43d246a730f4031452113/canarim-crab.png"
    alt="llama-crab logo"
    width="220"
  />
</p>

Safe, ergonomic Rust bindings for [`llama.cpp`](https://github.com/ggml-org/llama.cpp).

`llama-crab` provides:

- Low-level FFI bindings to `llama.cpp`, `ggml`, `gguf` and `mtmd` through `llama-crab-sys`.
- A safe high-level Rust API for model loading, text completion, chat completion and infill.
- Sampling chains, grammar-constrained decoding and JSON-Schema to GBNF conversion.
- Chat templates, tool-call parsing and OpenAI-compatible data structures.
- Embeddings, reranking, prompt cache, session state and speculative decoding.
- Multimodal support through `mtmd` for vision and audio capable GGUF models.
- Hardware backends for CPU, Metal, CUDA, Vulkan and ROCm through Cargo features.

Documentation is available at [docs.rs/llama-crab](https://docs.rs/llama-crab) and in the [mdBook user guide](docs/src/SUMMARY.md).

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
llama-crab = "0.1"
```

By default, `llama-crab` enables CPU OpenMP support and Apple Metal on `aarch64` macOS. To choose backends explicitly, disable default features and enable the ones you need:

```toml
[dependencies]
llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
```

The crate builds the bundled `llama.cpp` sources through CMake. You need:

- Rust 1.88 or newer.
- CMake 3.18 or newer.
- A C and C++ compiler supported by `llama.cpp`.
- A platform SDK when using GPU backends such as Metal, CUDA, Vulkan or ROCm.

## Cargo Features

| Feature | Description |
| --- | --- |
| `openmp` | CPU backend with OpenMP. Enabled by default. |
| `metal` | Apple Metal backend. Enabled by default on `aarch64` macOS. |
| `cuda` | NVIDIA CUDA backend. |
| `cuda-no-vmm` | CUDA backend without virtual memory management. |
| `vulkan` | Vulkan backend. |
| `rocm` | AMD ROCm/HIP backend. |
| `mtmd` | Multimodal support through `mtmd.h`; enables image/audio helpers. |
| `common` | Builds llama.cpp common utilities used by chat and grammar helpers. |
| `llguidance` | Enables the llguidance sampler integration. |
| `hf-tokenizer` | Enables Hugging Face tokenizer support. |
| `disk-cache` | Enables the persistent `sled`-backed prompt cache. |
| `dynamic-link` | Links llama.cpp as a shared object. |
| `dynamic-backends` | Loads GGML backends dynamically. |
| `system-ggml` | Uses a system GGML installation instead of the bundled copy. |

## Basic Usage

Load a GGUF model and generate a text completion:

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

## Chat Completion

Chat completion accepts a list of role-based messages. Built-in templates can be selected explicitly when you need deterministic formatting.

```rust,no_run
use llama_crab::chat::BuiltinTemplate;
use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
use llama_crab::{Llama, LlamaParams, Role};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(LlamaParams::new("models/instruct.gguf").with_n_ctx(4096))?;

    let messages = vec![
        ChatMessage::new(Role::System, "You are a concise assistant."),
        ChatMessage::new(Role::User, "Explain Rust ownership in one paragraph."),
    ];

    let response = create_chat_completion_with(
        &mut llama,
        &messages,
        BuiltinTemplate::ChatMl,
        &[],
        128,
    )?;

    println!("{}", response.content);
    Ok(())
}
```

## JSON Schema and Grammar-Constrained Decoding

`llama-crab` can convert JSON Schema into GBNF grammar and use grammar samplers to constrain model output.

```rust,no_run
use llama_crab::high_level::completion::json_schema_grammar;
use serde_json::json;

let schema = json!({
    "type": "object",
    "properties": {
        "name": { "type": "string" },
        "age": { "type": "integer" }
    },
    "required": ["name", "age"]
});

let grammar = json_schema_grammar(&schema)?;
# let _ = grammar;
# Ok::<(), Box<dyn std::error::Error>>(())
```

See the [`structured`](docs/src/examples/structured.md) example for a complete program.

## Tool Calling

The chat module includes incremental tool-call parsing for common model formats, including ChatML, Mistral, Llama 3, Functionary and plain JSON object output.

```rust,no_run
use llama_crab::chat::tool_call::{ToolFormat, ToolParser};

let mut parser = ToolParser::new(ToolFormat::ChatMl);
let calls = parser.feed("<tool_call>{\"name\":\"get_weather\",\"arguments\":{\"city\":\"Tokyo\"}}</tool_call>");
# let _ = calls;
# Ok::<(), Box<dyn std::error::Error>>(())
```

See [Chat & tool calling](docs/src/chat.md) for supported formats and parser behavior.

## Embeddings and Reranking

Enable embeddings when loading the model, then call `Llama::embed` to get an optionally L2-normalized vector.

```rust,no_run
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(
        LlamaParams::new("models/bge-small.gguf")
            .with_n_ctx(512)
            .with_embeddings(true),
    )?;

    let embedding = llama.embed("Rust is a systems programming language.", true)?;
    println!("dim = {}", embedding.len());

    Ok(())
}
```

See [Embeddings & reranking](docs/src/embeddings.md), [`embeddings`](docs/src/examples/embeddings.md) and [`embedding_search`](docs/src/examples/embedding_search.md).

## Multimodal Models

The `mtmd` feature exposes llama.cpp's multimodal pipeline for GGUF models that use a paired projector.

```toml
[dependencies]
llama-crab = { version = "0.1", features = ["mtmd"] }
```

Supported workflows include:

- Loading a text model and an `mmproj` projector.
- Decoding local images into `MtmdBitmap`.
- Tokenizing text and media together with `MtmdContext`.
- Evaluating multimodal chunks and continuing generation with normal samplers.

See [Multimodal](docs/src/multimodal.md), [`vision`](docs/src/examples/vision.md), [`mtmd`](docs/src/examples/mtmd.md), and the integration tests under [`llama-crab/tests`](llama-crab/tests).

## Speculative Decoding

Prompt-lookup speculative decoding is available through the `speculative` module. It can draft candidate tokens from repeated n-grams in the prompt and verify them with the main model.

See [Speculative decoding](docs/src/speculative.md) and the [`speculative`](docs/src/examples/speculative.md) example.

## Examples

The repository contains runnable example crates under [`examples/`](examples/README.md). The helper script downloads known-good GGUF fixtures on first run.

```bash
./examples/run.sh quickstart
./examples/run.sh chat
./examples/run.sh stateful_chat
./examples/run.sh embeddings
./examples/run.sh embedding_search
./examples/run.sh reranker
./examples/run.sh vision gemma4
./examples/run.sh vision lfm-vl
./examples/run.sh mtmd gemma4
./examples/run.sh tools
./examples/run.sh structured
./examples/run.sh speculative
```

Each example is a standalone Cargo crate and can be copied into another project.

## Documentation

- [API documentation](https://docs.rs/llama-crab)
- [User guide](docs/src/SUMMARY.md)
- [Examples guide](docs/src/examples/index.md)
- [Troubleshooting](docs/src/troubleshooting.md)

To serve the guide locally:

```bash
mdbook serve docs
```

## Crates

| Crate | Description |
| --- | --- |
| [`llama-crab`](https://crates.io/crates/llama-crab) | Safe high-level API and Rust abstractions. |
| [`llama-crab-sys`](https://crates.io/crates/llama-crab-sys) | Low-level FFI package that builds and links llama.cpp. |

Most applications should depend on `llama-crab`. Use `llama-crab-sys` only when you need direct access to raw llama.cpp symbols.

## Development

Clone with submodules:

```bash
git clone --recursive https://github.com/DominguesM/llama-crab.git
cd llama-crab
```

Common checks:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy -p llama-crab --all-features --all-targets
cargo doc -p llama-crab --no-deps --all-features
```

The minimum supported Rust version is 1.88 and is pinned in [`rust-toolchain.toml`](rust-toolchain.toml).

## License

Licensed under the MIT License. See [LICENSE-MIT](LICENSE-MIT).

## Acknowledgements

`llama-crab` builds on [`llama.cpp`](https://github.com/ggml-org/llama.cpp).

Inspired by [`llama-cpp-rs`](https://github.com/utilityai/llama-cpp-rs) and the feature completeness of [`llama-cpp-python`](https://github.com/abetlen/llama-cpp-python).
