# Contributing to llama-crab

Thanks for your interest! 🎉

## Getting started

1. Fork and clone the repo:
   ```bash
   git clone --recursive https://github.com/DominguesM/llama-crab.git
   cd llama-crab
   ```
2. Make sure you have **Rust 1.88+** (see MSRV below), **CMake 3.18+**, **Clang 14+** installed.
3. Build:
   ```bash
   cargo build --workspace --all-features
   ```

## Pull request checklist

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `cargo doc --workspace --all-features --no-deps`
- [ ] New public items are documented (`#[doc = "..."]` or doc comments)
- [ ] CHANGELOG.md is updated

## Code style

- `rustfmt` with default settings
- `clippy::pedantic` is on — suppress with `#[allow(...)]` only when justified
- Prefer strongly typed enums over `i32` constants
- Errors implement `std::error::Error + Send + Sync` via `thiserror`
- `unsafe` is **only** allowed in `llama-crab-sys`; the safe crate is 100% safe
- The root `tsconfig.json` is intentionally empty (`files: []`) so TypeScript
  checks run through package scripts instead of accidentally typechecking the
  workspace root. Use `pnpm typecheck` for JavaScript/TypeScript validation.

## MSRV

The Minimum Supported Rust Version is **1.88** and is pinned by
`rust-toolchain.toml`. Bumping the MSRV is a **breaking change** and
requires a major version bump — keep new code buildable on 1.88.
