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
- Chat templates and tool-call parsing helpers.
- Embeddings, reranking, manual prompt/session cache APIs and speculative decoding.
- Multimodal support through `mtmd` for vision and audio capable GGUF models.
- Hardware backends for CPU, Metal, CUDA, Vulkan, ROCm, OpenCL and KleidiAI through Cargo features.

Documentation is available at [docs.rs/llama-crab](https://docs.rs/llama-crab) and in the [Docusaurus user guide](https://dominguesm.github.io/llama-crab/) (source in [`docs/`](docs/README.md)).

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
- A platform SDK when using GPU backends such as Metal, CUDA, Vulkan, ROCm or OpenCL.

The workspace also provides packaging-oriented release profiles:

```bash
cargo build --profile release-perf
cargo build --profile release-size
```

## Cargo Features

| Feature            | Description                                                         |
| ------------------ | ------------------------------------------------------------------- |
| `openmp`           | CPU backend with OpenMP. Enabled by default.                        |
| `metal`            | Apple Metal backend. Enabled by default on `aarch64` macOS.         |
| `cuda`             | NVIDIA CUDA backend.                                                |
| `cuda-no-vmm`      | CUDA backend without virtual memory management.                     |
| `vulkan`           | Vulkan backend.                                                     |
| `rocm`             | AMD ROCm/HIP backend.                                               |
| `opencl`           | OpenCL backend, primarily for Android Adreno and Arm64 devices.     |
| `kleidiai`         | KleidiAI CPU kernels for Arm mobile targets.                        |
| `mtmd`             | Multimodal support through `mtmd.h`; enables image/audio helpers.   |
| `common`           | Builds llama.cpp common utilities used by chat and grammar helpers. |
| `llguidance`       | Enables the llguidance sampler integration.                         |
| `hf-tokenizer`     | Enables Hugging Face tokenizer support.                             |
| `disk-cache`       | Enables the persistent `sled`-backed prompt cache.                  |
| `dynamic-link`     | Links llama.cpp as a shared object.                                 |
| `dynamic-backends` | Loads GGML backends dynamically.                                    |
| `system-ggml`      | Uses a system GGML installation instead of the bundled copy.        |
| `shared-stdcxx`    | Uses `c++_shared` for Android builds.                               |
| `static-stdcxx`    | Uses `c++_static` for Android builds.                               |

For mobile packaging details, see [Mobile distribution](https://dominguesm.github.io/llama-crab/guides/mobile/).

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

For mobile targets, use `MobilePreset` as a starting point and override the
fields you need:

```rust,no_run
use llama_crab::{Llama, LlamaParams, MobilePreset};

let mut llama = Llama::load(
    LlamaParams::new("models/model.gguf")
        .with_mobile_preset(MobilePreset::Balanced)
        .with_n_ctx(2048),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
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

See the [structured output and tools guide](https://dominguesm.github.io/llama-crab/examples/structured-and-tools/) for a complete program.

## Tool Calling

The chat module includes incremental tool-call parsing for common model formats, including ChatML, Mistral, Llama 3, Functionary and plain JSON object output.

```rust,no_run
use llama_crab::chat::tool_call::{ToolFormat, ToolParser};

let mut parser = ToolParser::new(ToolFormat::ChatMl);
let calls = parser.feed("<tool_call>{\"name\":\"get_weather\",\"arguments\":{\"city\":\"Tokyo\"}}</tool_call>");
# let _ = calls;
# Ok::<(), Box<dyn std::error::Error>>(())
```

See [Chat & tool calling](https://dominguesm.github.io/llama-crab/rust/chat/) for supported formats and parser behavior.

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

See [Embeddings & reranking](https://dominguesm.github.io/llama-crab/rust/embeddings/) and [embedding examples](https://dominguesm.github.io/llama-crab/examples/embeddings-and-reranking/).

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

See [Multimodal](https://dominguesm.github.io/llama-crab/rust/multimodal/), [multimodal examples](https://dominguesm.github.io/llama-crab/examples/multimodal/), and the integration tests under [`crates/llama-crab/tests`](crates/llama-crab/tests).

## Speculative Decoding

Prompt-lookup speculative decoding is available through the `speculative` module. It can draft candidate tokens from repeated n-grams in the prompt and verify them with the main model.

See the [`speculative`](examples/README.md) example command.

## Streaming

`create_completion_stream` calls a synchronous callback as text becomes
available, while returning the same final `Completion` shape as
`create_completion`. Both high-level helpers clear sequence 0 before
each call; use lower-level context/session APIs if you need manual KV
reuse. The [streaming examples](https://dominguesm.github.io/llama-crab/examples/text-and-chat/) show
the callback loop.

## Server

`llama-crab-server` exposes the high-level API over HTTP with a worker
thread that owns the model and context.

```bash
cargo install llama-crab-server --features mtmd --force

llama-crab-server \
  --model models/qwen2.5-0.5b-instruct-q4_k_m.gguf \
  --host 127.0.0.1 \
  --port 8080
```

From a repository checkout, use `cargo run -p llama-crab-server -- ...`
instead of the installed binary.

Available routes include `/health`, `/v1/models`, `/v1/completions`,
`/v1/chat/completions`, `/v1/embeddings`, `/v1/rerank`, `/v1/reranking`,
`/extras/tokenize`, `/extras/tokenize/count`, and `/extras/detokenize`. Set `"stream": true` on
completion or chat requests to receive server-sent events. Completion and chat
requests accept sampling fields such as `temperature`, `top_k`, `top_p`,
`tfs_z`, `min_p`, penalties, Mirostat settings, `seed`, `min_tokens`, `n`,
`logprobs`, `logit_bias`, and `logit_bias_type`; chat requests also accept
`top_logprobs`, `template`, `tools`, `tool_choice`, and `function_call`;
structured generation can use `grammar`, `json_schema`, or
`response_format`, and text completions support `echo`, `suffix`, and
`best_of`. Embeddings support `encoding_format: "float"` or `"base64"`.
Multimodal chat is available when the server is installed or built with the
`mtmd` feature and started with `--mmproj`. Generation, embedding, rerank, and tokenizer
requests may include `model`; the bundled binary serves the model loaded at startup. See
[Server](https://dominguesm.github.io/llama-crab/server/) for request examples.

## Examples

The repository contains runnable example crates under [`examples/`](examples/README.md). The helper script downloads known-good GGUF fixtures on first run.

```bash
./examples/run.sh quickstart
./examples/run.sh chat
./examples/run.sh stateful_chat
./examples/run.sh embeddings
./examples/run.sh embedding_search
./examples/run.sh rerank
./examples/run.sh reranker
./examples/run.sh vision gemma4
./examples/run.sh vision lfm-vl
./examples/run.sh mtmd gemma4
./examples/run.sh tools
./examples/run.sh tool_calls_qwen
./examples/run.sh multimodal_http
./examples/run.sh structured
./examples/run.sh speculative
./examples/run.sh streaming
```

Each example is a standalone Cargo crate and can be copied into another project.

## Documentation

- [API documentation](https://docs.rs/llama-crab)
- [User guide](https://dominguesm.github.io/llama-crab/)
- [Examples guide](https://dominguesm.github.io/llama-crab/examples/)
- [Troubleshooting](https://dominguesm.github.io/llama-crab/troubleshooting/)

To serve the guide locally:

```bash
pnpm --dir docs install
pnpm --dir docs start
```

## Crates

| Crate                                                       | Description                                            |
| ----------------------------------------------------------- | ------------------------------------------------------ |
| [`llama-crab`](https://crates.io/crates/llama-crab)         | Safe high-level API and Rust abstractions.             |
| [`llama-crab-sys`](https://crates.io/crates/llama-crab-sys) | Low-level FFI package that builds and links llama.cpp. |
| [`llama-crab-server`](crates/llama-crab-server)             | HTTP server binary for local inference.                |

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
