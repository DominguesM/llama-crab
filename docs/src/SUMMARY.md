# Summary

[Introduction](./introduction.md)

---

# User guide

- [Getting started](./getting-started.md)
- [Sampling guide](./sampling.md)
- [Chat & tool calling](./chat.md)
- [Multimodal (vision + audio)](./multimodal.md)
- [JSON-Schema & GBNF grammars](./grammars.md)
- [Reference](./reference.md)

---

# Examples

- [`simple`](./examples/simple.md) — plain text completion
- [`chat`](./examples/chat.md) — multi-turn chat
- [`vision`](./examples/vision.md) — multimodal image+text
- [`tools`](./examples/tools.md) — tool / function calling
- [`structured`](./examples/structured.md) — JSON-Schema constrained output

---

# For contributors

The crate itself is documented via `cargo doc`; this book is a higher-
level narrative that ties the public API to the underlying concepts.
Run `mdbook serve docs/` to preview locally.
