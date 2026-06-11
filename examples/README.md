# Examples

Runnable, single-file programs demonstrating each public API.

| Example | Cargo command | What it does |
|---|---|---|
| [`simple`](./simple)         | `cargo run --bin simple -- model.gguf` | Plain text completion, greedy sampling |
| [`chat`](./chat)             | `cargo run --bin chat -- model.gguf`  | Single-turn chat completion |
| [`embeddings`](./embeddings) | `cargo run --bin embeddings -- model.gguf "text"` | Tokenize + show token IDs |
| [`reranker`](./reranker)     | `cargo run --bin reranker -- model.gguf` | Load a cross-encoder model |
| [`speculative`](./speculative) | `cargo run --bin speculative` | Demonstrate n-gram prompt-lookup draft |
| [`tools`](./tools)           | `cargo run --bin tools -- model.gguf` | Show tool-calling prompt structure |
| [`structured`](./structured) | `cargo run --bin structured -- model.gguf` | Constrain output to a JSON schema |
| [`mtmd`](./mtmd)             | `cargo run --bin mtmd --features mtmd -- model.gguf mmproj.gguf image.png` | Multimodal image+text query |

Every example skips cleanly when the model file is missing, so you can
copy any one of them as a starting point for your own code.

## Integration tests

See [`../tests/`](../tests) for end-to-end tests that exercise real GGUF
models from the Hugging Face Hub. Use:

```bash
cargo test --workspace --features mtmd
```
