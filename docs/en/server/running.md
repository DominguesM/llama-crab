# Running the server

The server binary lives in the `llama-crab-server` crate. It boots in
a few seconds (depending on the model size) and exposes an
OpenAI-compatible HTTP API.

## The basic command

```bash
cargo run -p llama-crab-server --release -- \
  --model models/qwen2.5-0.5b-instruct-q4_k_m.gguf \
  --host 127.0.0.1 \
  --port 8080
```

The binary prints a banner on stderr when it is ready:

```
llama-crab-server listening on http://127.0.0.1:8080
  model : llama-crab
  routes: /health, /v1/models, /v1/completions, /v1/chat/completions,
          /v1/embeddings, /v1/rerank, /extras/tokenize,
          /extras/tokenize/count, /extras/detokenize
  ctrl+c to stop
```

`RUST_LOG=debug` (or any other level) overrides the default `info`
filter.

## Command-line flags

| Flag | Default | Purpose |
| --- | --- | --- |
| `--model <PATH>` | _required_ | GGUF model path. |
| `--model-name <NAME>` | `llama-crab` | id returned by `/v1/models`. |
| `--host <HOST>` | `127.0.0.1` | Bind host. Use `0.0.0.0` to listen on all interfaces. |
| `--port <PORT>` | `8080` | Bind port. |
| `--n-ctx <N>` | `2048` | Context size in tokens. |
| `--n-batch <N>` | `512` | Logical batch size. |
| `--n-threads <N>` | `0` (auto) | CPU thread count; `0` keeps the default. |
| `--n-gpu-layers <N>` | `0` | Number of transformer layers to offload. |
| `--mobile-preset <PRESET>` | – | One of `low-ram`, `balanced`, `gpu-max`. |
| `--embeddings` | `false` | Enable embedding mode. |
| `--reranking` | `false` | Enable rerank endpoints. |
| `--pooling <TYPE>` | `unspecified` | `none`, `mean`, `cls`, `last`, `rank`, `unspecified`. |
| `--mmproj <PATH>` | – | Multimodal projector path. |

## Environment variables

Every flag has an `LLAMA_CRAB_*` environment variable equivalent,
useful in containerised environments and `systemd` units:

| Flag | Environment variable | Purpose |
| --- | --- | --- |
| `--model` | `LLAMA_CRAB_MODEL` | GGUF model path. |
| `--model-name` | `LLAMA_CRAB_MODEL_NAME` | id returned by `/v1/models`. |
| `--n-ctx` | `LLAMA_CRAB_N_CTX` | Context size. |
| `--n-batch` | `LLAMA_CRAB_N_BATCH` | Logical batch size. |
| `--n-threads` | `LLAMA_CRAB_N_THREADS` | CPU thread count; `0` keeps the default. |
| `--n-gpu-layers` | `LLAMA_CRAB_N_GPU_LAYERS` | GPU offload layers. |
| `--mobile-preset` | `LLAMA_CRAB_MOBILE_PRESET` | Mobile defaults: `low-ram`, `balanced`, `gpu-max`. |
| `--embeddings` | `LLAMA_CRAB_EMBEDDINGS` | Enable embedding mode. |
| `--reranking` | `LLAMA_CRAB_RERANKING` | Enable rerank endpoints. |
| `--pooling` | `LLAMA_CRAB_POOLING` | Pooling type. |
| `--mmproj` | `LLAMA_CRAB_MMPROJ` | Multimodal projector path. |

When `--mobile-preset` is set, the other flags only override the
preset's defaults if they are explicitly provided. Without a preset,
the server keeps its previous defaults.

## Common recipes

### Embeddings server

```bash
cargo run -p llama-crab-server --release -- \
  --model models/bge-small-en-v1.5-q4_k_m.gguf \
  --embeddings
```

### Reranking server

```bash
cargo run -p llama-crab-server --release -- \
  --model models/bge-reranker-base-q4_k_m.gguf \
  --reranking \
  --pooling rank
```

### Multimodal chat server

Build the server with the `mtmd` feature and provide the projector:

```bash
cargo run -p llama-crab-server --release --features mtmd -- \
  --model models/LFM2.5-VL-1.6B-Q4_K_M.gguf \
  --mmproj models/LFM2.5-VL-1.6B-mmproj-BF16.gguf
```

### Production-style unit

A `systemd` unit file for a small chat server:

```ini title="/etc/systemd/system/llama-crab.service"
[Unit]
Description=llama-crab-server
After=network.target

[Service]
Type=simple
User=llama
Environment="LLAMA_CRAB_MODEL=/var/lib/llama-crab/qwen2.5-7b-instruct-q4_k_m.gguf"
Environment="LLAMA_CRAB_N_GPU_LAYERS=99"
Environment="LLAMA_CRAB_N_CTX=4096"
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/llama-crab-server --host 0.0.0.0 --port 8080
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### Docker

A minimal Dockerfile:

```dockerfile title="Dockerfile"
FROM rust:1.88-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y cmake build-essential
COPY . .
RUN cargo build --release -p llama-crab-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libgomp1 ca-certificates
COPY --from=builder /app/target/release/llama-crab-server /usr/local/bin/
COPY models/ /var/lib/llama-crab/
ENV LLAMA_CRAB_MODEL=/var/lib/llama-crab/qwen2.5-7b-instruct-q4_k_m.gguf
EXPOSE 8080
CMD ["llama-crab-server", "--host", "0.0.0.0", "--port", "8080"]
```

## Quick health check

```bash
curl http://127.0.0.1:8080/health
# → 200 OK
```

## Where to next?

- [API reference](api.md) — every route, with `curl` examples.
- [Streaming](streaming.md) — Server-Sent Events contract.
- [Structured output](structured.md) — the `response_format`,
  `grammar` and `json_schema` fields.
