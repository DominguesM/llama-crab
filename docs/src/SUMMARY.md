# Summary

[Introduction](./introduction.md)

---

# User guide

- [Getting started](./getting-started.md)
- [Backends & GPU offload](./backends.md)
- [Sampling guide](./sampling.md)
- [Chat & tool calling](./chat.md)
- [Multimodal (vision + audio)](./multimodal.md)
- [Embeddings & reranking](./embeddings.md)
- [JSON-Schema & GBNF grammars](./grammars.md)
- [Speculative decoding](./speculative.md)
- [Caching & session state](./caching.md)
- [Stateful chat](./stateful_chat.md)
- [Server](./server.md)
- [Reference](./reference.md)
- [Troubleshooting / FAQ](./troubleshooting.md)

---

# Examples

- [Examples overview](./examples/index.md)
- [`simple`](./examples/simple.md) — plain text completion
- [`streaming`](./examples/streaming.md) — high-level token streaming
- [`quickstart`](./examples/quickstart.md) — first end-to-end program
- [`chat`](./examples/chat.md) — multi-turn chat
- [`stateful_chat`](./examples/stateful_chat.md) — interactive REPL with history
- [`vision`](./examples/vision.md) — multimodal image+text
- [`mtmd`](./examples/mtmd.md) — raw `mtmd.h` API
- [`embeddings`](./examples/embeddings.md) — embedding extraction
- [`embedding_search`](./examples/embedding_search.md) — semantic search
- [`reranker`](./examples/reranker.md) — cross-encoder scoring
- [`tools`](./examples/tools.md) — tool / function calling
- [`structured`](./examples/structured.md) — JSON-Schema constrained output
- [`speculative`](./examples/speculative.md) — prompt-lookup draft decoding

---

# For contributors

The crate itself is documented via `cargo doc`; this book is a higher-
level narrative that ties the public API to the underlying concepts.
Run `mdbook serve docs/` to preview locally.
