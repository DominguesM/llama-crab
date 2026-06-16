# Changelog

All notable changes to `llama-crab` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.7] - 2026-06-16

### Fixed

- `llama-crab`: resolved a use-after-move of `LlamaModel` that caused
  SIGSEGV when `Llama` crossed a return boundary. The self-referential
  `&'a LlamaModel` field on `LlamaContext` (papered over by a
  `PhantomData<*mut ()>` + `transmute` in `Llama::load`) has been
  replaced with a heap-allocated `Box<LlamaModel>` and a
  `NonNull<LlamaModel>` raw pointer on the context. `LlamaContext` no
  longer carries a lifetime parameter; all `impl` blocks and the
  `Llama::context_mut` shim were updated accordingly. The
  encode→decode workaround previously required in `embedding.rs` and
  `rerank.rs` is no longer needed.

### Added

- `llama-crab`: regression tests for the use-after-move fix and the
  affected public API paths:
  - `tests/embedding_regression.rs` — `embeddings_seq`,
    `embeddings_ith`, `logits_ith`, `sampled_probs_ith` and re-entrant
    `embed()`.
  - `tests/rerank_api.rs` — `Llama::rerank` cross-encoder scoring
    against `bge-reranker-base`, plus an empty-documents edge case.
  - `tests/infill_api.rs` — `complete_infill` and re-entrant infill
    against Qwen2.5 0.5B Instruct.
  - `tests/streaming_api.rs` — `create_completion_stream` token
    collection and `StreamControl::Stop` early termination.
  - `QWEN_DEFAULT_PATH` and `RERANK_DEFAULT_PATH` constants in
    `tests/common.rs` following the existing `resolve_path` convention.

### Changed

- `llama-crab` workspace: replaced the explicit 4-crate `members`
  list with a `crates/*` glob. All current crates live under
  `crates/`, so the glob is equivalent. `resolver` and `edition` are
  kept at `2` and `2021` to avoid the FFI bindings breakage that
  edition 2024 would cause in `llama-crab-sys` (bindgen generates
  `extern "C"` blocks without `unsafe`, which is a hard error in
  edition 2024).

## [0.1.6] - 2026-06-15

### Added

- `llama-crab`: added optional Hugging Face Hub model resolution behind the
  `hf-hub` feature. `Llama::load` can now resolve HF repository IDs, download
  GGUF files through `hf-hub`'s sync API and then load the cached model.
- `llama-crab`: added the public `hf` module with `HfRepo`, downloader traits
  and test helpers, plus `LlamaError::ModelDownload` for download and
  resolution failures.
- `llama-crab`: added `LlamaParams` builders for Hugging Face inputs:
  `with_hf_filename`, `with_hf_revision`, `with_hf_token`,
  `with_hf_cache_dir` and `with_hf_endpoint`.
- `llama-crab-server`: added opt-in Hugging Face support through the
  `hf-hub` cargo feature and the `--hf-filename` CLI flag
  (`LLAMA_CRAB_HF_FILENAME`) so server users can pass HF repo IDs as model
  sources.
- Added Hugging Face documentation for the Rust crate and server, plus an
  env-gated integration test for the HF download/cache path.

### Changed

- `tauri-plugin-llama-crab` now enables `llama-crab`'s `hf-hub` feature by
  default, allowing Tauri apps to load HF repo IDs through `load_model`
  without additional plugin-side feature wiring.
- CI/CD workflows now use improved caching and consolidated required-check
  coverage for faster release validation.

### Fixed

- Fixed strict rustdoc failures caused by private intra-doc links in the new
  Hugging Face resolver internals.
- Reformatted the new Hugging Face integration tests and helpers so the
  release passes the workspace rustfmt checks.

## [0.1.5] - 2026-06-15

### Changed

- Moved the documentation site out of this repository. The site is now
  published at <https://llama-crab.nlp.rocks/> instead of the previous
  GitHub Pages URLs. The `docs/` folder and the
  `Publish docs site` GitHub Actions workflow have been removed from this
  repo. README files and crate-level docs throughout this workspace were
  updated to point at the new URL.

### Added

- `tauri-plugin-llama-crab`: added a `Config` struct and `init_with_config`
  entry point so consumers can apply plugin-wide defaults (n_ctx, n_batch,
  n_ubatch, n_threads, n_threads_batch, n_gpu_layers, default model name)
  at startup. Anything left as `None` lets the per-request field win, with
  the `llama-crab` defaults as the final fallback.
- `tauri-plugin-llama-crab`: added the `mtmd` cargo feature. When enabled,
  `load_model` can take an `mmproj_path` and the chat pipeline runs
  multimodal (vision) inference through `llama.cpp`'s `mtmd` projector.
  Image inputs are accepted as `data:image/...;base64,...` URLs and as
  local file paths.
- `tauri-plugin-llama-crab`: added granular `PluginError` kinds
  (`workerSpawnFailed`, `workerDisconnected`, `workerPanicked`,
  `multimodalNotEnabled`, `multimodalSetup`, `mediaDecode`) so the
  TypeScript client can distinguish failure modes instead of collapsing
  every error into `worker`.

### Changed

- `tauri-plugin-llama-crab`: `JoinError` from `spawn_blocking` now maps
  to `workerPanicked`; `mpsc::RecvError` maps to `workerDisconnected`;
  thread-spawn failures map to `workerSpawnFailed`.
- `@llama-crab/tauri`: the Support Matrix entry for multimodal now
  reflects that the Rust plugin must be built with the `mtmd` cargo
  feature for image parts to be processed.

## [0.1.4] - 2026-06-14

### Added

- Added high-level streaming completion APIs, including
  `create_completion_stream`, `create_completion_stream_with_sampler`,
  `CompletionChunk`, `StreamControl` and richer completion logprob
  metadata.
- Added `llama-crab-server`, an HTTP server binary for local inference
  with completions, chat completions, embeddings, reranking,
  tokenization, detokenization, SSE streaming and optional multimodal
  chat support.
- Added OpenAI-style high-level convenience helpers for text, chat and
  embeddings with token accounting.
- Added the `server_lfm` example wrapper and an `lfm-text` download
  target for launching the HTTP server with LFM text models.
- Added the `streaming` example to demonstrate callback-driven text
  generation.
- Added `tauri-plugin-llama-crab`, a Tauri IPC runtime for loading
  GGUF models and exposing OpenAI-like chat, completion, embedding,
  rerank, tokenization and model-management commands.
- Added the `@llama-crab/core` and `@llama-crab/tauri` TypeScript
  packages with shared OpenAI-like contracts, request mappers and a
  Tauri client.
- Added the `tauri-chat-lfm` desktop example and smoke coverage for
  the Tauri chat workflow.
- Added mobile-oriented runtime presets through `MobilePreset` and
  `LlamaParams::with_mobile_preset`.
- Added broader tool-call streaming support, including OpenAI-style
  tool-call deltas.
- Added documentation deployment for the project guide.

### Changed

- Migrated the user guide from mdBook/MkDocs-era documentation to
  Docusaurus, with expanded server, mobile, Tauri, TypeScript,
  streaming, chat, embeddings and grammar coverage.
- Reorganized the repository into `crates/` and `packages/` workspaces
  so Rust crates, TypeScript packages and examples share one release
  surface.
- README files now point users to the project guide hosted at
  <https://llama-crab.nlp.rocks/>.
- CI and release workflows now build, test and publish
  `llama-crab-server`, `tauri-plugin-llama-crab` and TypeScript
  packages alongside the library crates.
- CI workflows now run through manual dispatch instead of push triggers,
  and documentation jobs use nightly Cargo where required.
- The `hf-tokenizer` dependency now enables the `onig` feature for
  tokenizer compatibility.
- Rustdoc crate logos now reference the current Canarim Crab asset.

### Fixed

- Removed unused placeholder OpenAI-compat wrapper bindings from
  `llama-crab-sys` and the old chat module export.
- Gated the Metal backend build configuration to macOS targets.
- Hardened documentation builds and docs deployment workflow behavior.
- Cleaned up server and example runner support for the new server and
  mobile workflows.

## [0.1.201] - 2026-06-13

### Changed

- Prepared the `0.1.201` release after the `0.1.2` documentation and
  runtime fixes.
- CI feature matrices now align with the actual crate feature set.
- Coverage workflow scope now matches the published crate layout.

### Fixed

- CI now installs Vulkan shader dependencies, including the shader
  compiler required by Vulkan-enabled builds.
- Removed redundant casts from the vision CI tests.

## [0.1.2] - 2026-06-13

### Added

- Expanded mdBook user guide coverage with new chapters for backends,
  embeddings and reranking, speculative decoding, caching and session
  state, stateful chat and troubleshooting.
- Added dedicated mdBook pages for the `quickstart`, `stateful_chat`,
  `embeddings`, `embedding_search`, `reranker`, `mtmd` and
  `speculative` examples, plus a consolidated examples overview.
- New one-command examples workflow with `examples/run.sh`, including
  automatic model resolution for text, embedding and multimodal demos.
- New runnable examples:
  - `quickstart` for the smallest end-to-end text demo.
  - `stateful_chat` for multi-turn chat with history commands.
  - `embedding_search` for BGE-small semantic ranking.
  - `lfm_vl_vision` for LFM2.5-VL image question answering.
- Top-level `Makefile` convenience targets for building, checking,
  downloading models, running examples and cleaning local artifacts.
- Script smoke test coverage for the example runner, model downloader
  and cleanup script.

### Changed

- Release workflow now waits for the `llama-crab-sys` crate to become
  visible in the crates.io index before publishing `llama-crab`.
- Automated `llama.cpp` updates now run weekly, record the resolved
  upstream tag in the workflow environment and create PRs with the
  target tag in the title.
- README and contributor docs now point users to the mdBook guide and
  document the Rust 1.88 MSRV.
- Crate metadata now uses supported crates.io categories and both
  published crates include README content in their package metadata.
- Project license changed from dual `MIT OR Apache-2.0` to MIT-only.
- Example documentation now describes the downloadable GGUF model set
  and the new one-command workflow.
- Download workflow now prefers the Hugging Face CLI (`hf download`)
  instead of raw `curl`, with a Python module fallback for environments
  where the `hf` executable is broken.
- `make quickstart`, `make stateful-chat`, `make vision-*` and
  `make embedding-search` now download only the models required by the
  requested example instead of fetching the full model set.
- Rust sources and tests were formatted consistently with `cargo fmt`.

### Fixed

- High-level completion now tokenizes with model-special tokens enabled
  and stops on either EOS or EOT.
- Embedding generation now clears sequence 0 before encoding, requests
  embeddings for every token in the batch and reads the sequence-level
  embedding output.
- Grammar sampler initialization now passes the model vocabulary handle
  required by the current `llama.cpp` API.
- JSON-Schema grammar generation now escapes object property names
  correctly and uses a smaller default unbounded string length.
- Vision examples, docs and integration tests now feed generated tokens
  back into the context after multimodal prompt evaluation and sample
  from the current logits with `-1`.
- Gemma 4 and LFM2.5-VL vision tests now use updated projector fixture
  names and resolve fixtures from the workspace parent when needed.
- Tool-call documentation now escapes literal pipe tokens in markdown
  tables so mdBook does not split model control tokens into extra
  columns.
- Integration tests for `llama-crab` now live inside the crate package,
  so `cargo package` no longer ignores tests declared outside the
  published crate root.
- Text completion, FIM and multimodal sampling now clear sequence 0
  before each high-level call and sample from the current batch logits
  after the initial decode.
- `LlamaModel::detokenize` now uses lossy UTF-8 conversion, avoiding
  errors on token byte sequences that are not valid UTF-8 by themselves.
- Multimodal prompts can now use `default_media_marker()` to place image
  tokens reliably before `mtmd_tokenize`.
- `examples/run.sh` now passes downloaded model, projector and fixture
  paths to examples that require positional arguments.
- `scripts/clean.sh` no longer errors when optional cleanup path arrays
  are empty.
- Removed the unsupported `unreachable-docs` lint from
  `llama-crab-sys`, eliminating the Rust 1.88 warning.
- `.DS_Store` and downloaded model artifacts are ignored.

## [0.1.1] - 2026-06-11

### Changed

- **Module layout matches `PLAN.md` exactly**. Sub-modules added:
  - `src/sampling/{strategies,grammar,chain,custom}.rs`
  - `src/multimodal/{context,bitmap,chunks}.rs`
  - `src/chat/{parser,template,tool_call,message}.rs`
  - `src/model/{kv_overrides,buft_overrides,vocab,params}.rs`
  - `src/context/{embeddings,kv_cache,session,sampling_state,params}.rs`
  - `src/high_level/{completion,chat_completion,embedding,rerank,infill,tokenizer,hf_tokenizer}.rs`
  - `src/json_schema/`
  - `src/logit_bias.rs`
- Tokenizers moved into `src/high_level/` per the plan.

### Added

- New `KvOverrides` and `BufferTypeOverride` builders (model-side).
- New `LlamaContext::seq_rm`, `seq_cp`, `seq_keep`, `seq_add`, `seq_div`
  plus `seq_pos_min` / `seq_pos_max` (KV cache management).
- New `LlamaContext::logits_ith`, `sampled_token_ith`, `sampled_probs_ith`
  (sampling-state introspection).
- `chat::parser::ChatParseState` — incremental JSON-object parser
  used by streaming clients.
- `high_level::embedding::Llama::embed(text, normalize)` — convenience
  helper around the encode + extract_embeddings pipeline.
- `high_level::rerank::Llama::rerank(query, docs)` — cross-encoder
  re-ranking driver.
- `high_level::infill::Llama::complete_infill(prefix, suffix)` — FIM
  code-completion driver.
- `coverage.yml` CI workflow using `cargo-llvm-cov` with an 80 % gate
  on lib coverage.
- `docs/src/examples/{simple,chat,vision,tools,structured}.md`
  with runnable snippets.

### Fixed

- `sampling` module now has all 17 strategies as named constructors
  (re-exported via `LlamaSampler::…`).
- `MtmdBitmap::from_file` now decodes via the `image` crate (no
  need for an intermediate `Vec<u8>` buffer).
- `LlamaContext::raw_handle()` is now public (was `pub(crate)`) to
  allow multimodal interop without `unsafe impl Deref`.
- `cache::RamCache::lookup` correctly finds the longest matching
  prefix (was iterating over the wrong variable).
- `chat::template::render_template` now correctly handles the
  `else` / `elif` branches of `if` blocks (was emitting them
  unconditionally).
- `json_schema::schema_to_grammar` properly registers `$ref`
  definitions *before* dereferencing them (was failing on top-level
  `$ref` schemas).

### Quality

- **136 tests passing, 0 failures** (was 100).
- `cargo test --workspace --features mtmd,disk-cache` → 136 passed.
- `cargo doc -p llama-crab --all-features` (RUSTDOCFLAGS=-D warnings) → clean.
- `cargo clippy -p llama-crab --all-targets -- -D warnings` → no errors.
- `cargo build --workspace --release --features mtmd,disk-cache` → 17s.
- Per-module coverage on pure-logic modules (cargo-llvm-cov):
  - `cache.rs` 92 %, `chat/message.rs` 97 %, `chat/parser.rs` 92 %,
    `json_schema/mod.rs` 90 %, `model/kv_overrides.rs` 91 %,
    `logit_bias.rs` 100 %.

### Notes

- **MSRV: 1.88** (bumped from 1.80 in v0.1.0 because of `hashbrown 0.17`
  and `image 0.25` requirements; documented here for visibility).

## [0.1.0] - 2026-06-11

### Added

- Initial workspace: `llama-crab-sys` (FFI via bindgen + cmake) +
  `llama-crab` (safe API).
- 17 sampling strategies, 14 chat templates, Jinja2-subset renderer,
  tool-calling parsers, multimodal vision, FIM, speculative decoding,
  JSON-Schema → GBNF, RAM/Disk cache.
- 9 example crates and 3 integration tests covering Gemma 4 and
  LFM2.5-VL.

[Unreleased]: https://github.com/DominguesM/llama-crab/compare/v0.1.5...HEAD
[0.1.5]: https://github.com/DominguesM/llama-crab/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/DominguesM/llama-crab/compare/v0.1.201...v0.1.4
[0.1.201]: https://github.com/DominguesM/llama-crab/compare/v0.1.2...v0.1.201
[0.1.2]: https://github.com/DominguesM/llama-crab/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/DominguesM/llama-crab/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/DominguesM/llama-crab/releases/tag/v0.1.0
