---
title: Releases
---

# Releases

Release work must keep crates, TypeScript packages, generated API docs, and
examples aligned.

Before release:

- Verify crate manifests and feature defaults.
- Verify package manifests and generated TypeScript declarations.
- Run packaging dry-runs where possible.
- Rebuild docs from the same source state that will be published.

For docs.rs failures, inspect the published `.crate` artifact when diagnosing a
released version. A fixed local manifest does not change a tarball that has
already been published.
