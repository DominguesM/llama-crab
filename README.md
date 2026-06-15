<p align="center">
  <img
    src="https://gist.githubusercontent.com/DominguesM/127b9e5614e0e2da6b896fb3da3c8f2d/raw/d5dec07e795979f0a1b43d246a730f4031452113/canarim-crab.png"
    alt="llama-crab logo"
    width="220"
  />
</p>

<p align="center">Safe, ergonomic and complete Rust bindings for <a href="https://github.com/ggml-org/llama.cpp">llama.cpp</a>.</p>

<p align="center">
  <a href="https://crates.io/crates/llama-crab"><img alt="Crates.io" src="https://img.shields.io/crates/v/llama-crab.svg?style=flat-square" /></a>
  <a href="https://docs.rs/llama-crab"><img alt="Documentation" src="https://docs.rs/llama-crab/badge.svg?style=flat-square" /></a>
  <a href="https://github.com/DominguesM/llama-crab/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/DominguesM/llama-crab/ci.yml?style=flat-square&branch=main" /></a>
  <a href="https://github.com/DominguesM/llama-crab/actions/workflows/coverage.yml"><img alt="Coverage" src="https://img.shields.io/github/actions/workflow/status/DominguesM/llama-crab/coverage.yml?style=flat-square&branch=main" /></a>
  <a href=""><img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square" /></a>
</p>

---

### Installation

```bash
# Rust crate
cargo add llama-crab

# HTTP server (optional)
cargo install llama-crab-server --features mtmd --force
```

For backend selection (Metal, CUDA, Vulkan, ROCm, OpenCL, ...), see the
[installation guide](https://llama-crab.nlp.rocks/installation/).

### Quickstart

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

Runnable end-to-end examples live in the separate
[`llama-crab-examples`](https://github.com/DominguesM/llama-crab-examples) repo.

### What is in the box

- Safe high-level API for text completion, chat completion, infill and embeddings.
- Low-level FFI bindings to `llama.cpp`, `ggml`, `gguf` and `mtmd`.
- Sampling chains, grammar-constrained decoding and JSON-Schema to GBNF conversion.
- Tool-call parsing for ChatML, Mistral, Llama 3, Functionary and plain JSON.
- Multimodal support (vision and audio) through `mtmd`.
- Hardware backends for CPU, Metal, CUDA, Vulkan, ROCm, OpenCL and KleidiAI.
- HTTP server (`llama-crab-server`) and Tauri plugin (`tauri-plugin-llama-crab`).
- TypeScript contracts and client (`@llama-crab/core`, `@llama-crab/tauri`).

### Crates and Packages

| Name                                                        | Description                                            |
| ----------------------------------------------------------- | ------------------------------------------------------ |
| [`llama-crab`](crates/llama-crab)                           | Safe high-level API and Rust abstractions. Start here. |
| [`llama-crab-sys`](crates/llama-crab-sys/README.md)         | Low-level FFI bindings to `llama.cpp`.                 |
| [`llama-crab-server`](crates/llama-crab-server)             | OpenAI-compatible HTTP server binary.                  |
| [`tauri-plugin-llama-crab`](crates/tauri-plugin-llama-crab) | Tauri plugin for in-app local inference.               |
| [`@llama-crab/core`](packages/core/README.md)               | OpenAI-like TypeScript contracts and helpers.          |
| [`@llama-crab/tauri`](packages/tauri/README.md)             | TypeScript client for the Tauri plugin.                |

### Documentation

- [User guide](https://llama-crab.nlp.rocks/) — installation, examples, server, multimodal and Tauri guides.
- [API reference (Rust)](https://docs.rs/llama-crab) — published `rustdoc`.
- [Examples](https://llama-crab.nlp.rocks/examples/) — guide for the runnable example crates.
- [Troubleshooting](https://llama-crab.nlp.rocks/troubleshooting/) — common build and runtime issues.

The documentation site is published at <https://llama-crab.nlp.rocks/>.

### Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) before
opening a pull request and follow the [Code of Conduct](CODE_OF_CONDUCT.md).
Bug reports and security issues are tracked through
[GitHub Issues](https://github.com/DominguesM/llama-crab/issues) and
[SECURITY.md](SECURITY.md).

Clone with submodules:

```bash
git clone --recursive https://github.com/DominguesM/llama-crab.git
cd llama-crab
```

### License

Licensed under the [MIT License](LICENSE-MIT).

`llama-crab` builds on [`llama.cpp`](https://github.com/ggml-org/llama.cpp).
