# Changelog

All notable changes to `llama-crab` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-11

### Added

#### Workspace
- Workspace `llama-crab-sys` (FFI via `bindgen` + `cmake`) + `llama-crab` (100 % safe Rust).
- `llama.cpp` pinned to release tag `b9601` (git submodule).
- MSRV: **1.88.0** (pinned via `rust-toolchain.toml`).
- 9 example crates: `simple`, `chat`, `embeddings`, `reranker`, `speculative`, `tools`, `structured`, `mtmd`, `vision`.
- 3 integration tests covering text and vision paths.
- GitHub Actions: CI matrix (Linux, macOS, Windows × CPU/Metal/Vulkan), auto-bump submodule workflow, release workflow.
- `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `LICENSE-MIT`, `LICENSE-APACHE`.
- `docs/` mdBook (introduction, getting-started, sampling, chat, multimodal, grammars, reference).

#### Backends
- CPU (OpenMP), Apple Metal (default on macOS aarch64), NVIDIA CUDA, Vulkan, AMD ROCm/HIP, dynamic-link, dynamic-backends, system-ggml.

#### Sampling
- 17 strategies: `greedy`, `dist`, `top_k`, `top_p`, `min_p`, `typical`, `temp`, `temp_ext`, `xtc`, `top_n_sigma`, `mirostat` (v1 + v2), `penalties`, `dry`, `adaptive_p`, `logit_bias`, `infill`, `grammar`.
- `SamplerChain` builder (`temp().top_p().min_p().penalties().build()`).

#### Multimodal
- `MtmdContext`, `MtmdBitmap`, `MtmdInputText`, `MtmdInputChunks`, `MtmdInputChunk`.
- `MtmdBitmap::from_file` decodes PNG/JPEG via the `image` crate.
- `examples/vision/` end-to-end CLI and `tests/{gemma4_vision,lfm_vl_vision}.rs` integration tests.

#### Chat & Tools
- `Role`, `ChatMessage` (with `tool_call_id`, `tool_calls`, `name`).
- 14 built-in templates: `chatml`, `mistral-instruct`, `llama-3`, `alpaca`, `vicuna`, `openchat`, `zephyr`, `gemma`, `phi-3`, `command-r`, `deepseek`, `granite`, `oasst_llama`, `plain`.
- `BuiltinTemplate::from_str_ci` + `detect_chat_format` for auto-detection.
- **Jinja2 subset** renderer (if/elif/else, for, set, filters, list/dict, and/or/not/in).
- `ToolDefinition`, `ToolCall`, `ToolParser` (stateful streaming).
- 5 tool-call formats: `ChatMl`, `Mistral`, `Llama3`, `Plain`, `Functionary`.
- `chat::oaicompat` (feature `common`): OpenAI-compat chat template rendering with `tools`, `tool_choice`, `response_format`, `reasoning_format`.

#### JSON-Schema
- Pure-Rust JSON-Schema → GBNF converter (`type`, `properties`, `required`, `items`, `enum`, `const`, `oneOf`/`anyOf`/`allOf`, `$ref`/`definitions`/`$defs`, `format` shortcuts for `date-time`/`email`/`uri`/`uuid`).
- C++ bridge to `common::json_schema_to_grammar` (feature `common`).

#### Caching
- `RamCache` (longest-prefix lookup, `BTreeMap`).
- `DiskCache` (persistent, `sled`-backed; feature `disk-cache`).

#### Tokenization
- `Tokenizer` trait + `LlamaTokenizer<'_>` (delegates to the model).
- `HfTokenizer` (feature `hf-tokenizer`).
- FIM/infill helpers (`FimTokens`, `fim_prompt`).

#### Speculative decoding
- `DraftModel` trait.
- `PromptLookupDecoding` (n-gram prompt lookup).
- `speculative_decode` driver (main ctx + draft model + sampler chain).

#### Other
- `LlamaError` (typed via `thiserror`): model load, context load, decode, encode, embedding, chat, JSON-Schema.
- `LlamaBackend`, `NumaStrategy` (6 variants).
- `LlamaContext::n_batch`, `n_ubatch`, `n_ctx`, `n_seq_max`, `raw_handle()`.
- `state_size`, `state_to_bytes`, `load_state`, `state_save_file`, `state_load_file`.
- `embeddings()`, `embeddings_seq()`, `embeddings_ith()`, `normalize()`.
- `tracing` integration via `send_logs_to_tracing`.
- `scripts/download_models.sh` (HF Hub or curl).

### Tests

- **100 unit / doctest tests** passing (sampling, chat, tool-calling, JSON-Schema, cache, multimodal, FIM, speculative).
- 5 integration tests under `tests/`, all skip cleanly when the model is missing.
- `cargo test --workspace --features mtmd,disk-cache` is the canonical run command.

### Verified

- `cargo build --workspace --release --features mtmd` ✅
- `cargo test --workspace --features mtmd,disk-cache` → **100 passed, 0 failed**.
- `cargo doc -p llama-crab --all-features` → ✅ (no warnings with `RUSTDOCFLAGS=-D warnings`).
- `cargo clippy -p llama-crab-sys --all-targets` → ✅.
- `cargo clippy -p llama-crab --features metal,mtmd,disk-cache --all-targets` → no errors.
- End-to-end model load on Metal backend (Apple M4) verified with
  `lmstudio-community/gemma-4-E4B-it-GGUF` (text) and
  `unsloth/LFM2.5-VL-1.6B-GGUF` (vision).
