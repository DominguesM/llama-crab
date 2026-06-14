# llama-crab documentation

This directory contains the Docusaurus documentation site for `llama-crab`.

The site is an English-first Docusaurus documentation set for users, Rust
developers, server operators, Tauri integrators, TypeScript consumers, and
repository contributors.

## Local development

```bash
pnpm --dir docs install
pnpm --dir docs start
```

## Production build

From the repository root:

```bash
pnpm --dir docs build
```

That command builds the Docusaurus static site. Refresh generated API docs
before publishing when Rust or TypeScript public APIs change.

## Generated API docs

- TypeScript API is generated into `docs/docs/api/typescript`.
- Rust API is copied into `docs/static/api/rust`.

Do not hand-edit generated API output.
