# Changelog

All notable changes to `llama-crab` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial workspace skeleton: `llama-crab-sys` (FFI) + `llama-crab` (safe API)
- `llama.cpp` pinned to release tag `b9601`
- GitHub Actions: CI matrix (Linux/macOS/Windows × CPU/Metal/Vulkan), auto-bump submodule workflow, release workflow
- `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`
