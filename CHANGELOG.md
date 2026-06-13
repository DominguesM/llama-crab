# Changelog

All notable changes to `llama-crab` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
  - `src/chat/{parser,template,tool_call,message,oaicompat}.rs`
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
  used by streaming OAI-compat clients.
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

[Unreleased]: https://github.com/DominguesM/llama-crab/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/DominguesM/llama-crab/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/DominguesM/llama-crab/releases/tag/v0.1.0
