# `server_lfm` — `llama-crab-server` with the LFM2.5-VL model

A thin wrapper that starts the `llama-crab-server` HTTP binary pre-wired
for the [Liquid AI LFM2.5-VL 1.6B](https://huggingface.co/unsloth/LFM2.5-VL-1.6B-GGUF)
text GGUF (`Q4_K_M` quantization, ~1 GB). The wrapper can also pass the paired
`mmproj` projector and build `llama-crab-server` with `mtmd` for multimodal
HTTP chat.

## Run it

```bash
./examples/run.sh server_lfm
```

`run.sh` resolves the `lfm-vl` download target (idempotent — skips files
already in `./models/`), then runs this wrapper with the model path. The first
run also builds `llama-crab-server` in `--release`.

For multimodal HTTP chat, use the dedicated target:

```bash
./examples/run.sh multimodal_http
```

Override any default with positional arguments:

```bash
./examples/run.sh server_lfm -- --port 9090 --n-ctx 4096
```

When the server is ready, you'll see:

```
listening on http://127.0.0.1:8080
```

## Routes

| HTTP route                       | Purpose                                         |
| -------------------------------- | ----------------------------------------------- |
| `GET  /health`                   | readiness probe                                 |
| `GET  /v1/models`                | list the configured model                       |
| `POST /v1/completions`           | text completion (supports `stream: true`)       |
| `POST /v1/chat/completions`      | chat completion (supports `stream: true`)       |
| `POST /v1/embeddings`            | embeddings (only when started with `--embeddings`) |
| `POST /v1/rerank`                | reranking (only when started with `--reranking`) |
| `POST /extras/tokenize`          | text → token ids                                |
| `POST /extras/tokenize/count`    | text → token count                              |
| `POST /extras/detokenize`        | token ids → text                                |

Full parameter reference lives in [`docs/src/server.md`](../../docs/src/server.md).

## Call examples

### Health check

```bash
curl -s http://127.0.0.1:8080/health
```

### List the model

```bash
curl -s http://127.0.0.1:8080/v1/models | jq
```

### Text completion

```bash
curl -sN http://127.0.0.1:8080/v1/completions \
  -H 'content-type: application/json' \
  -d '{
    "prompt": "The capital of France is",
    "max_tokens": 16,
    "temperature": 0.0,
    "stream": false
  }' | jq
```

Same request, streamed as SSE:

```bash
curl -sN http://127.0.0.1:8080/v1/completions \
  -H 'content-type: application/json' \
  -d '{
    "prompt": "The capital of France is",
    "max_tokens": 16,
    "temperature": 0.0,
    "stream": true
  }'
```

### Chat completion

```bash
curl -sN http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [
      {"role": "system", "content": "You are a concise assistant. Reply in one sentence."},
      {"role": "user",   "content": "Name three Rust smart pointers."}
    ],
    "max_tokens": 96,
    "temperature": 0.2,
    "template": "chatml",
    "stream": false
  }' | jq
```

Streamed chat completion (chunks arrive in the order documented in
[`docs/src/server.md`](../../docs/src/server.md)):

```bash
curl -sN http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [
      {"role": "user", "content": "Explain Rust ownership briefly."}
    ],
    "max_tokens": 64,
    "template": "chatml",
    "stream": true
  }'
```

### Structured output (JSON-schema constrained decoding)

```bash
curl -sN http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [
      {"role": "user", "content": "Create one fictional person."}
    ],
    "template": "chatml",
    "max_tokens": 96,
    "response_format": {
      "type": "json_object",
      "schema": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "age":  {"type": "integer"}
        },
        "required": ["name", "age"]
      }
    }
  }' | jq
```

### Tool calling

```bash
curl -sN http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [
      {"role": "user", "content": "Weather in Tokyo?"}
    ],
    "template": "chatml",
    "max_tokens": 96,
    "tools": [{
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get weather for a city",
        "parameters": {
          "type": "object",
          "properties": {"city": {"type": "string"}},
          "required": ["city"]
        }
      }
    }],
    "tool_choice": {"type": "function", "function": {"name": "get_weather"}}
  }' | jq
```

### Multimodal chat

Start the server with `./examples/run.sh multimodal_http`, then send a local
image path through an `image_url` content part:

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

### Tokenizer extras

```bash
curl -sN http://127.0.0.1:8080/extras/tokenize \
  -H 'content-type: application/json' \
  -d '{"input": "How many tokens in this query?"}' | jq

curl -sN http://127.0.0.1:8080/extras/tokenize/count \
  -H 'content-type: application/json' \
  -d '{"input": "How many tokens in this query?"}' | jq

curl -sN http://127.0.0.1:8080/extras/detokenize \
  -H 'content-type: application/json' \
  -d '{"tokens": [1, 2, 3]}' | jq
```

## Embeddings

Start the server with `--embeddings` and an embedding model instead — the
LFM2.5-VL text side is not an embedding model. The run script does not
flip the flag for you; pass it through:

```bash
./examples/run.sh server_lfm -- --port 8081
# then in another shell, with a bge model loaded:
#   cargo run --release -p llama-crab-server -- \
#     --model models/bge-small-en-v1.5-q4_k_m.gguf --embeddings
```

Once the server reports `--embeddings` in its startup log, the
`/v1/embeddings` route is active and accepts the same payload shape as
the text completions example.

## See also

- `examples/lfm_vl_vision/` — interactive REPL that uses the multimodal
  APIs directly against the LFM2.5-VL model and its mmproj projector.
- `docs/src/server.md` — full server reference (parameters, SSE contract,
  error shape).
- `docs/src/chat.md` and `docs/src/grammars.md` — chat template and
  JSON-schema details.
