# Changelog

The full release history lives in the
[`CHANGELOG.md`](https://github.com/DominguesM/llama-crab/blob/main/CHANGELOG.md)
file in the repository root. This page is a summary of the most
recent releases, with the breaking changes highlighted.

## Recent releases

### `0.1.300` (latest)

- Added the `llama-crab-server` HTTP binary with completions, chat,
  embeddings, reranking, tokenization and SSE streaming endpoints.
- Added high-level streaming completion APIs and the runnable
  `streaming` example.
- Added mobile-oriented presets through `MobilePreset` and
  `LlamaParams::with_mobile_preset`.
- Added the `server_lfm` wrapper example for launching the server with
  LFM text models.
- Migrated the user guide from mdBook to Material for MkDocs with
  English and Portuguese documentation trees.

### `0.1.201`

- Prepared the post-`0.1.2` release line.
- Aligned CI feature matrices with the crate features.
- Scoped coverage to the published crate layout.
- Installed Vulkan shader dependencies in CI.

### `0.1.2`

- Expanded the mdBook guide and runnable example coverage.
- Added one-command example workflows and model download helpers.
- Fixed completion, embedding, grammar, multimodal and example runner
  behavior against the current `llama.cpp` API.

### `0.1.0`

- Initial public release of the `0.1.x` series.
- The safe high-level API on top of `llama-crab-sys`.
- 9 example crates and 3 integration tests.

## Migration recipes

When a breaking change lands in `0.1.x`, the recipe for migrating
your code is documented in the
[MSRV & versioning](../reference/msrv.md) page.

## Where to next?

- [MSRV & versioning](../reference/msrv.md) — the full migration
  guide.
- [GitHub releases](https://github.com/DominguesM/llama-crab/releases) -
  the per-release artifacts and notes.
- [Contributing](contributing.md) — how to send a fix for a bug
  you found in a release.
