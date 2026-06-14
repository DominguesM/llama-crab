# Server

`llama-crab-server` is the HTTP entry point for running one local GGUF model behind OpenAI-style JSON APIs. It loads the model once at startup, keeps inference work on a dedicated worker thread, and exposes routes for text completion, chat completion, embeddings, reranking, tokenization, detokenization, model listing, and health checks.

The server is useful when you want a local HTTP boundary instead of linking the Rust crate directly. It is intentionally simple: one process owns one loaded model, and callers select behavior through request fields such as `stream`, `temperature`, `top_p`, `stop`, `grammar`, `json_schema`, or `response_format`.

## Route summary

| Method | Route | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Returns `{ "status": "ok" }`. |
| `GET` | `/v1/models` | Returns the configured model name in an OpenAI-style model list. |
| `POST` | `/v1/completions` | Text completion for one prompt or multiple prompts. |
| `POST` | `/v1/chat/completions` | Chat completion with ChatML by default and optional tool definitions. |
| `POST` | `/v1/embeddings` | Embeddings for one string or an array of strings. |
| `POST` | `/v1/rerank` | Scores documents against a query. Requires startup with `--reranking`. |
| `POST` | `/v1/reranking` | Alias for reranking. |
| `POST` | `/rerank` | Alias for reranking. |
| `POST` | `/reranking` | Alias for reranking. |
| `POST` | `/extras/tokenize` | Converts text to token ids. |
| `POST` | `/extras/tokenize/count` | Returns the token count for text. |
| `POST` | `/extras/detokenize` | Converts token ids back to text. |

## Core constraints

- The `model` field in requests is accepted for compatibility, but the process serves the single model loaded at startup.
- Non-streaming failures return JSON with an `error.message` and `error.type` of `invalid_request`.
- Streaming failures are emitted as SSE `event: error` frames, followed by the final `[DONE]` frame.
- CORS is permissive in the current server.
- Reranking is disabled unless the process starts with `--reranking`.
- Embeddings require a model loaded with embeddings enabled, which the server does when started with `--embeddings` or `--reranking`.

## Where to go next

- Use [Running the Server](./running.md) for startup flags and environment variables.
- Use [OpenAI-Compatible API](./openai-api.md) for request and response shapes.
- Use [Streaming](./streaming.md) for SSE behavior.
- Use [Operations](./operations.md) for process-level guidance.
