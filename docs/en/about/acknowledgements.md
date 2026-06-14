# Acknowledgements

`llama-crab` would not exist without the work of the people and
projects listed below. Thank you.

## The foundation

- **[llama.cpp](https://github.com/ggml-org/llama.cpp)** —
  Georgi Gerganov and the `ggml-org` team. The C/C++ inference
  engine that `llama-crab` wraps. `llama.cpp` is the reason we can
  run large language models on consumer hardware.
- **[GGML](https://github.com/ggml-org/ggml)** — the tensor library
  that powers every backend.

## The Rust ecosystem

`llama-crab` stands on the shoulders of a long list of Rust
projects. Highlights:

- **`bindgen`** — auto-generation of the FFI bindings in
  `llama-crab-sys`.
- **`cmake`** and **`cc`** — the C/C++ build glue.
- **`serde` and `serde_json`** — request/response types, tool
  definitions, the JSON-Schema converter.
- **`anyhow` and `thiserror`** — error handling.
- **`tokio` and `axum`** — the HTTP server.
- **`tracing` and `tracing-subscriber`** — structured logging.
- **`sled`** — the on-disk prompt cache.

A full list lives in the workspace `Cargo.lock`.

## The models

The examples in this repository are tested against open-weights
models from the Hugging Face Hub. Thank you to:

- **Alibaba (Qwen team)** — Qwen 2 / 2.5.
- **Meta (Llama team)** — Llama 3 / 3.1 / 3.2 / 3.3.
- **Google (Gemma team)** — Gemma 2 / 3 / 4.
- **Mistral AI** — Mistral and Mixtral.
- **Microsoft (Phi team)** — Phi-3.
- **DeepSeek AI** — DeepSeek-V2 / V2.5.
- **Liquid AI** — LFM2.5-VL.
- **Beijing Academy of Artificial Intelligence (BGE team)** —
  BGE embeddings and rerankers.
- **Cohere** — Command R / R+.

## The tools

- **[Material for MkDocs](https://squidfunk.github.io/mkdocs-material/)** —
  the documentation site theme.
- **[Pymdown Extensions](https://facelessuser.github.io/pymdown-extensions/)** —
  the Markdown extensions used by the docs.
- **[mdBook](https://rust-lang.github.io/mdBook/)** — the previous
  documentation tool. Thank you for the years of service.

## The community

Thanks to every contributor who has filed an issue, sent a PR, or
helped someone in the discussions. The full list lives in the
[contributors graph](https://github.com/DominguesM/llama-crab/graphs/contributors).

## Where to next?

- [License](license.md) — the full text.
- [Contributing](contributing.md) — how to send a fix for a bug
  you found.
- [Home](../index.md) — back to the documentation home.
