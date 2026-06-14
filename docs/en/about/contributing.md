# Contributing

Thanks for considering a contribution to `llama-crab`! This page
walks through the most common ways to get involved.

## Code of Conduct

The project follows the
[Contributor Covenant](https://www.contributor-covenant.org/). By
participating, you agree to uphold its terms. The full text is in
[`CODE_OF_CONDUCT.md`](https://github.com/DominguesM/llama-crab/blob/main/CODE_OF_CONDUCT.md).

## Filing a bug

Search the [GitHub issues] first to avoid duplicates. When you file
a new issue, include:

- A short, descriptive title.
- The exact command you ran and the exact output you got.
- The model identifier (Hugging Face path) and the GGUF size.
- The platform (`aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`,
  `x86_64-pc-windows-msvc`, …) and the Rust version.
- The `llama_crab::LlamaBackend` capability probes, if relevant.

## Proposing a feature

Open an issue with the `enhancement` label. Describe:

- The use case the feature unlocks.
- The API shape you would expect.
- Whether you would be willing to send a PR.

For larger features, the [Discussions] tab is a better place to
gather feedback before opening an issue.

## Sending a pull request

### 1. Fork and clone

```bash
git clone --recursive https://github.com/<you>/llama-crab.git
cd llama-crab
git checkout -b my-feature
```

The `--recursive` is important: the `llama-crab-sys/llama.cpp`
submodule is part of the build.

### 2. Make your changes

The repository follows the standard Rust style. The
`Makefile` exposes the common checks:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy -p llama-crab --all-features --all-targets
cargo doc -p llama-crab --no-deps --all-features
```

If you add a public API, write a rustdoc comment that includes a
runnable example. The CI fails the build if the rustdoc has
broken links or warnings.

### 3. Add a test

The safe API has unit tests in the same module as the code, and
integration tests under `llama-crab/tests/`. The integration tests
skip cleanly when the model is not on disk.

### 4. Update the documentation

If you add a public feature, update the user guide in `docs/`. The
documentation source is Markdown; the build is mkdocs-material.

### 5. Open the PR

Push the branch to your fork and open a PR against `main`. The CI
runs:

- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy -p llama-crab --all-features --all-targets`
- `cargo doc -p llama-crab --no-deps --all-features`
- The matrix build on the supported backends.

A maintainer will review and either merge or request changes.

## Adding a Cargo feature

Adding a new Cargo feature is a public API change. The
convention is:

1. Discuss the feature in an issue first.
2. Add the feature in `llama-crab-sys/build.rs` and the
   `llama-crab` `Cargo.toml`.
3. Update the [Cargo features reference](../reference/cargo-features.md).
4. Add a CI matrix row that exercises the new feature.
5. Document the feature in the relevant guide.

## Adding a new example

See the [Examples index](../examples/index.md) for the boilerplate
and the rules.

## Adding a built-in chat template

1. Add the new `BuiltinTemplate` variant in
   `llama_crab::chat::template`.
2. Add the rendering logic in `render_builtin`.
3. Add a unit test that renders a known message and asserts the
   expected output.
4. Update the
   [built-in chat templates reference](../reference/chat-templates.md).

## Release process

Releases are cut by the maintainers. The flow:

1. Bump the version in `Cargo.toml`.
2. Update the `CHANGELOG.md`.
3. Tag the commit.
4. The `release.yml` workflow builds every supported target,
   publishes to crates.io, and creates a GitHub release.

## Where to next?

- [GitHub issues] — file a bug or a feature request.
- [Discussions] — design questions and ideas.
- [Code of conduct] — the rules of the road.

[GitHub issues]: https://github.com/DominguesM/llama-crab/issues
[Discussions]: https://github.com/DominguesM/llama-crab/discussions
[Code of conduct]: https://github.com/DominguesM/llama-crab/blob/main/CODE_OF_CONDUCT.md
