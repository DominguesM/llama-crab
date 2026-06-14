---
title: Server Examples
---

# Server examples

`crates/llama-crab-server` exposes local OpenAI-compatible HTTP routes backed by
one loaded GGUF model.

## Install the server

For normal use outside this repository, install the binary from crates.io:

```bash
cargo install llama-crab-server --features mtmd --force
```

Then run it directly:

```bash
llama-crab-server --model models/qwen2.5-0.5b-instruct-q4_k_m.gguf
```

The examples below are repository wrappers for local development and smoke
testing.

## LFM server wrapper

```bash
./examples/run.sh server_lfm
```

This starts the server with the LFM2.5-VL text GGUF. Pass server flags after
`--`:

```bash
./examples/run.sh server_lfm -- --port 9090 --n-ctx 4096
```

The server defaults to `127.0.0.1:8080`.

## Useful routes

| Route | Purpose |
| --- | --- |
| `GET /health` | Readiness probe. |
| `GET /v1/models` | List the configured model. |
| `POST /v1/completions` | Text completions, including SSE streaming. |
| `POST /v1/chat/completions` | Chat completions, including SSE streaming. |
| `POST /v1/embeddings` | Embeddings when started with `--embeddings`. |
| `POST /v1/rerank` | Reranking when started with `--reranking`. |
| `POST /extras/tokenize` | Tokenize text. |
| `POST /extras/tokenize/count` | Count tokens. |
| `POST /extras/detokenize` | Convert token IDs back to text. |

## Chat request

```bash
curl -sN http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [
      {"role": "system", "content": "You are concise."},
      {"role": "user", "content": "Name three Rust smart pointers."}
    ],
    "template": "chatml",
    "max_tokens": 96,
    "temperature": 0.2
  }' | jq
```

Set `"stream": true` to receive SSE chunks.

## Multimodal request

Start:

```bash
./examples/run.sh multimodal_http
```

Call:

```bash
curl -sN http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [{
      "role": "user",
      "content": [
        {"type": "text", "text": "Describe this image in one sentence."},
        {"type": "image_url", "image_url": {"url": "tests/fixtures/test_image.png"}}
      ]
    }],
    "template": "chatml",
    "max_tokens": 64
  }' | jq
```

## Embedding and rerank modes

Embedding mode must be started with an embedding model:

```bash
llama-crab-server \
  --model models/bge-small-en-v1.5-q4_k_m.gguf \
  --embeddings
```

Rerank mode is wrapped by:

```bash
./examples/run.sh rerank
```
