# MSRV & versioning

This page documents the minimum supported Rust version (MSRV), the
`SemVer` policy, and the llama.cpp commit pin.

## MSRV

**`1.88.0`** — pinned via [`rust-toolchain.toml`](https://github.com/DominguesM/llama-crab/blob/main/rust-toolchain.toml).

Bumping the MSRV is a **breaking change** and will trigger a major
version bump.

The MSRV is exercised by the CI matrix on every push to `main`. A
build failure on the lowest supported version is treated as a bug
and fixed before the change is merged.

## `SemVer` policy

`llama-crab` follows [Semantic Versioning 2.0.0](https://semver.org/).
For a public API in a `0.x.y` release, the rules are:

- **Patch (`0.0.y` → `0.0.y+1`)** — backwards-compatible bug fixes.
  Internal refactors, documentation, performance improvements.
- **Minor (`0.x.y` → `0.x+1.0`)** — backwards-compatible new API
  surface. New modules, new methods, new Cargo features. Existing
  code keeps working.
- **Major (`0.x.y` → `1.0.0`)** — backwards-incompatible changes.

The crate is currently in the `0.1.x` series, which means the API
is *expected* to evolve. Breaking changes within `0.1.x` are
documented in the [CHANGELOG](https://github.com/DominguesM/llama-crab/blob/main/CHANGELOG.md)
and the [migration guide](#migration-guide) below.

## llama.cpp pin

`llama-crab` pins `llama.cpp` to a specific commit through a
submodule and a Cargo feature. The exact commit is visible in:

- The README badge (`llama.cpp: <commit>`).
- The submodule pointer in
  [`llama-crab-sys/llama.cpp`](https://github.com/DominguesM/llama-crab/tree/main/llama-crab-sys/llama.cpp).
- The `Cargo.lock` (look for the `llama-cpp-sys-2` dependency).

Two builds of the same `llama-crab` version always produce the same
native library, so the binary is reproducible.

### Bumping the pin

Bumping the `llama.cpp` commit is treated as a **minor** version
bump in `0.1.x`. The CI matrix re-runs every backend and the
integration tests; the bump is merged only when everything is green.

## Cargo.lock

The `Cargo.lock` is committed. For libraries, this is unusual; for
`llama-crab` it is intentional, because the build links against a
pinned native library and we want downstream consumers to see
exactly the same artifacts.

## Release cadence

Releases are cut from `main` whenever a meaningful change has
landed. The criteria are:

- A new public API or a meaningful refinement of an existing one.
- A new Cargo feature, a backend, or a model family.
- A meaningful set of bug fixes.

The release process is automated through the
[`release.yml`](https://github.com/DominguesM/llama-crab/blob/main/.github/workflows/release.yml)
GitHub Actions workflow, which builds every supported target,
publishes to crates.io, and creates a GitHub release.

## Migration guide

This section is updated whenever a breaking change lands in
`0.1.x`. The full history lives in the
[CHANGELOG](https://github.com/DominguesM/llama-crab/blob/main/CHANGELOG.md).

### From `0.1.0` to `0.1.100`

- `LlamaParams::new` now requires a `&str` or `String` (was
  `Into<PathBuf>`). Convert with `.to_string()` or `.into()`.
- `Llama::create_completion` no longer accepts a `temperature: f32`
  argument. Pass it through `CompletionOptions::with_temperature`.
- The `n_threads_batch` field on `LlamaParams` is renamed to
  `with_n_threads_decode`.

### From `0.1.100` to `0.1.200`

- `chat::render_builtin` returns `Result<String, _>` instead of
  `String`. Error is `ChatRenderError::UnknownArchitecture`.
- The `mtmd` feature now requires a Rust 2024 edition build
  environment.

## Where to next?

- [CHANGELOG](https://github.com/DominguesM/llama-crab/blob/main/CHANGELOG.md) —
  the full release history.
- [Crate layout](crate-layout.md) — the source tree.
- [Cargo features](cargo-features.md) — the long-form reference.
