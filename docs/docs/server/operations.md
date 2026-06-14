# Operations

`llama-crab-server` is a single-model process. Operationally, treat each process as one loaded model with one worker thread receiving inference jobs from the HTTP layer.

## Process model

- Model loading happens before the HTTP listener is started.
- The Axum app accepts requests and sends jobs to a standard Rust channel.
- The worker thread owns the `Llama` instance and runs completion, chat, embedding, reranking, and tokenization work.
- If model loading fails, the worker exits and startup cannot serve useful inference.

## Health and readiness

Use `/health` for a lightweight liveness check:

```bash
curl -f http://127.0.0.1:8080/health
```

Use `/v1/models` to verify the configured model name:

```bash
curl -f http://127.0.0.1:8080/v1/models
```

The health route does not run inference. For deployment readiness, also run a small completion, embedding, or rerank request that matches the model role.

## Logging

The server initializes `tracing_subscriber` with `RUST_LOG` support. Without `RUST_LOG`, it uses `info`.

```bash
RUST_LOG=info llama-crab-server --model /models/model.gguf
```

Useful startup output includes the listening URL, model name, and route list.

## Capacity planning

The current server is not a multi-model router. To serve multiple models, run multiple server processes on different ports and route at a higher layer.

Tune memory and latency with startup parameters:

- `--n-ctx` for context size.
- `--n-batch` for prompt processing batch size.
- `--n-threads` for CPU thread count.
- `--n-gpu-layers` for GPU offload.
- `--mobile-preset` for preset resource tradeoffs.
- `--pooling` for embedding and reranking models.

## Security posture

The server is designed for local inference workflows. The code currently applies a permissive CORS layer. Bind to `127.0.0.1` unless you intentionally expose it:

```bash
llama-crab-server --host 127.0.0.1 --model /models/model.gguf
```

If you bind to a network interface, put authentication, TLS, rate limiting, and request-size controls in front of the process.

## Error handling

Non-streaming route handlers return:

```json
{
  "error": {
    "message": "description",
    "type": "invalid_request"
  }
}
```

Most model and request validation failures are returned as `400 Bad Request`. Channel send and worker response failures return `500 Internal Server Error`.
