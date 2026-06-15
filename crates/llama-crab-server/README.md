# `llama-crab-server`

OpenAI-compatible HTTP server for local [`llama-crab`](https://crates.io/crates/llama-crab) inference.

Built on top of [`axum`](https://docs.rs/axum) and exposes a worker thread
that owns the model and context.

## Installation

```bash
cargo install llama-crab-server --features mtmd --force
```

For development against a workspace checkout:

```bash
cargo run -p llama-crab-server -- \
  --model models/qwen2.5-0.5b-instruct-q4_k_m.gguf \
  --host 127.0.0.1 \
  --port 8080
```

## Routes

| Route | Description |
| --- | --- |
| `GET /health` | Liveness probe. |
| `GET /v1/models` | List the loaded model. |
| `POST /v1/completions` | OpenAI legacy text completions. |
| `POST /v1/chat/completions` | OpenAI chat completions with streaming. |
| `POST /v1/embeddings` | Embeddings (`float` or `base64`). |
| `POST /v1/rerank`, `POST /v1/reranking` | Rerank. |
| `POST /extras/tokenize`, `/extras/tokenize/count`, `/extras/detokenize` | Tokenizer helpers. |

Multimodal chat is available when the binary is built with `--features mtmd`
and started with `--mmproj <projector.gguf>`.

For the full request schema, sampling fields and structured-output options,
see the [server guide](https://llama-crab.nlp.rocks/server/).

## Resources

- [API reference (docs.rs)](https://docs.rs/llama-crab-server)
- [Workspace README](../../README.md)

## License

Licensed under the [MIT License](../../LICENSE-MIT).
