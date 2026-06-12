# Changelog

All notable changes to `llama-crab` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
