# Changelog

The full release history lives in the
[`CHANGELOG.md`](https://github.com/DominguesM/llama-crab/blob/main/CHANGELOG.md)
file in the repository root. This page is a summary of the most
recent releases, with the breaking changes highlighted.

## Recent releases

### `0.1.300` (latest)

- Added `llama_crab::embed::EmbedOptions` for fine-grained control
  over embedding extraction.
- Improved the `LlamaBackend` capability probes.
- Bumped the pinned `llama.cpp` commit to the latest stable.
- New `MobilePreset::GpuMax` for high-end mobile GPUs.
- Fixed a bug where `chunks.eval` could fail silently on certain
  projector versions.

### `0.1.200`

- Added the `llguidance` feature for the [`llguidance`] sampler.
- New `BuiltinTemplate::DeepSeek2` and `BuiltinTemplate::CommandR`.
- Improved the Jinja2 subset renderer to support nested `for`
  loops.
- The `chat` module now exports a `ToolDefinition::with_strict`
  builder method.
- Bumped the MSRV to `1.88.0`.

### `0.1.100`

- The `mtmd` feature now supports audio bitmaps in addition to
  images.
- New `Llama::rerank` high-level helper for cross-encoder
  rankers.
- The `server` binary now exposes a `--mobile-preset` flag.
- Fixed a memory leak in `LlamaSampler::chain` when the chain was
  dropped mid-generation.

### `0.1.0`

- Initial public release of the `0.1.x` series.
- The safe high-level API on top of `llama-crab-sys`.
- 14 example crates.
- The `llama-crab-server` binary.

## Migration recipes

When a breaking change lands in `0.1.x`, the recipe for migrating
your code is documented in the
[MSRV & versioning](../reference/msrv.md) page.

## Where to next?

- [MSRV & versioning](../reference/msrv.md) — the full migration
  guide.
- [GitHub releases](https://github.com/DominguesM/llama-crab/releases) —
  the per-release artifacts and notes.
- [Contributing](contributing.md) — how to send a fix for a bug
  you found in a release.

[`llguidance`]: https://github.com/microsoft/llguidance
