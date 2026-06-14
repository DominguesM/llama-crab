---
title: Versioning
---

# Versioning

Prefer matching llama-crab package versions across Rust and TypeScript when one
application uses more than one integration surface.

## Release alignment

llama-crab aims to keep Rust crate releases and TypeScript package releases
aligned when public contracts change. For application upgrades:

- Upgrade related Rust crates and TypeScript packages together.
- Read the generated Rust and TypeScript API references for changed signatures.
- Re-run the server, Tauri, embedding, reranking, or multimodal path your app
  actually uses.

## Documentation versioning

The Docusaurus site initially publishes only the current documentation. Add
Docusaurus versioned docs later if users need docs for multiple released
versions at the same time.

Release chores for maintainers are covered in [Releases](../contributing/releases.md).
