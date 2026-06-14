# Running the Server

Install the server binary from crates.io:

```bash title="Install the server"
cargo install llama-crab-server --features mtmd --force
```

The `mtmd` feature enables multimodal request handling. If you only serve text,
embedding, or reranking models, you can omit `--features mtmd`.

After installation, run the binary with a GGUF model path:

```bash title="Run the server"
llama-crab-server --model /path/to/model.gguf
```

By default it listens on `127.0.0.1:8080` and advertises the model as `llama-crab`.

```bash title="Health and model discovery"
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/v1/models
```

## Common startup options

| CLI flag | Environment variable | Default | Notes |
| --- | --- | --- | --- |
| `--model` | `LLAMA_CRAB_MODEL` | Required | Path to the GGUF model. |
| `--host` | `LLAMA_CRAB_HOST` | `127.0.0.1` | Bind host. |
| `--port` | `LLAMA_CRAB_PORT` | `8080` | Bind port. |
| `--model-name` | `LLAMA_CRAB_MODEL_NAME` | `llama-crab` | Value returned in responses. |
| `--n-ctx` | `LLAMA_CRAB_N_CTX` | `2048` without mobile preset | Context size. |
| `--n-batch` | `LLAMA_CRAB_N_BATCH` | `512` without mobile preset | Batch size. |
| `--n-threads` | `LLAMA_CRAB_N_THREADS` | `0` without mobile preset | Positive values set decode and, with mobile presets, batch threads. |
| `--n-gpu-layers` | `LLAMA_CRAB_N_GPU_LAYERS` | `0` without mobile preset | Number of layers to offload. |
| `--mobile-preset` | `LLAMA_CRAB_MOBILE_PRESET` | None | One of `low-ram`, `balanced`, `gpu-max`. |
| `--mmproj` | `LLAMA_CRAB_MMPROJ` | None | Multimodal projector path. |
| `--embeddings` | `LLAMA_CRAB_EMBEDDINGS` | `false` | Enables embedding mode at load time. |
| `--reranking` | `LLAMA_CRAB_RERANKING` | `false` | Enables embedding mode and the rerank endpoint. |
| `--pooling` | `LLAMA_CRAB_POOLING` | `unspecified` | One of `none`, `mean`, `cls`, `last`, `rank`, `unspecified`. |

## Examples

Run a chat model with more context and GPU offload:

```bash
llama-crab-server \
  --model /models/chat.gguf \
  --model-name local-chat \
  --n-ctx 4096 \
  --n-gpu-layers 32
```

Run an embedding model:

```bash
llama-crab-server \
  --model /models/embed.gguf \
  --model-name local-embed \
  --embeddings \
  --pooling mean
```

Run a reranker:

```bash
llama-crab-server \
  --model /models/reranker.gguf \
  --model-name local-reranker \
  --reranking \
  --pooling rank
```

Run the installed binary with multimodal support and pass an mmproj file:

```bash
llama-crab-server \
  --model /models/vision.gguf \
  --mmproj /models/mmproj.gguf
```

Multimodal chat content requires both the `mtmd` Cargo feature and `--mmproj`.
Source checkout commands are covered in [Development](../contributing/development.md).
