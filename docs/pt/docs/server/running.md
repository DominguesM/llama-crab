# Executando o servidor

O binário do servidor vive no crate `llama-crab-server`. Ele sobe
em alguns segundos (dependendo do tamanho do modelo) e expõe uma
API HTTP compatível com OpenAI.

## O comando básico

```bash
cargo run -p llama-crab-server --release -- \
  --model models/qwen2.5-0.5b-instruct-q4_k_m.gguf \
  --host 127.0.0.1 \
  --port 8080
```

O binário imprime um banner no stderr quando está pronto:

```
llama-crab-server listening on http://127.0.0.1:8080
  model : llama-crab
  routes: /health, /v1/models, /v1/completions, /v1/chat/completions,
          /v1/embeddings, /v1/rerank, /extras/tokenize,
          /extras/tokenize/count, /extras/detokenize
  ctrl+c to stop
```

`RUST_LOG=debug` (ou qualquer outro nível) sobrescreve o filtro
padrão `info`.

## Flags de linha de comando

| Flag | Padrão | Propósito |
| --- | --- | --- |
| `--model <PATH>` | _obrigatório_ | Caminho do modelo GGUF. |
| `--model-name <NAME>` | `llama-crab` | id retornado por `/v1/models`. |
| `--host <HOST>` | `127.0.0.1` | Host de bind. Use `0.0.0.0` para escutar em todas as interfaces. |
| `--port <PORT>` | `8080` | Porta de bind. |
| `--n-ctx <N>` | `2048` | Tamanho do contexto em tokens. |
| `--n-batch <N>` | `512` | Tamanho lógico do batch. |
| `--n-threads <N>` | `0` (auto) | Contagem de threads de CPU; `0` mantém o padrão. |
| `--n-gpu-layers <N>` | `0` | Número de camadas do transformer a descarregar. |
| `--mobile-preset <PRESET>` | – | Um de `low-ram`, `balanced`, `gpu-max`. |
| `--embeddings` | `false` | Habilita modo de embedding. |
| `--reranking` | `false` | Habilita endpoints de rerank. |
| `--pooling <TYPE>` | `unspecified` | `none`, `mean`, `cls`, `last`, `rank`, `unspecified`. |
| `--mmproj <PATH>` | – | Caminho do projetor multimodal. |

## Variáveis de ambiente

Cada flag tem um equivalente em variável de ambiente `LLAMA_CRAB_*`,
útil em ambientes containerizados e unidades `systemd`:

| Flag | Variável de ambiente | Propósito |
| --- | --- | --- |
| `--model` | `LLAMA_CRAB_MODEL` | Caminho do modelo GGUF. |
| `--model-name` | `LLAMA_CRAB_MODEL_NAME` | id retornado por `/v1/models`. |
| `--n-ctx` | `LLAMA_CRAB_N_CTX` | Tamanho do contexto. |
| `--n-batch` | `LLAMA_CRAB_N_BATCH` | Tamanho lógico do batch. |
| `--n-threads` | `LLAMA_CRAB_N_THREADS` | Contagem de threads de CPU; `0` mantém o padrão. |
| `--n-gpu-layers` | `LLAMA_CRAB_N_GPU_LAYERS` | Camadas com offload de GPU. |
| `--mobile-preset` | `LLAMA_CRAB_MOBILE_PRESET` | Padrões mobile: `low-ram`, `balanced`, `gpu-max`. |
| `--embeddings` | `LLAMA_CRAB_EMBEDDINGS` | Habilita modo de embedding. |
| `--reranking` | `LLAMA_CRAB_RERANKING` | Habilita endpoints de rerank. |
| `--pooling` | `LLAMA_CRAB_POOLING` | Tipo de pooling. |
| `--mmproj` | `LLAMA_CRAB_MMPROJ` | Caminho do projetor multimodal. |

Quando `--mobile-preset` é definido, as outras flags só
sobrescrevem os padrões do preset se forem explicitamente
fornecidas. Sem um preset, o servidor mantém seus padrões
anteriores.

## Receitas comuns

### Servidor de embeddings

```bash
cargo run -p llama-crab-server --release -- \
  --model models/bge-small-en-v1.5-q4_k_m.gguf \
  --embeddings
```

### Servidor de reranking

```bash
cargo run -p llama-crab-server --release -- \
  --model models/bge-reranker-base-q4_k_m.gguf \
  --reranking \
  --pooling rank
```

### Servidor de chat multimodal

Compile o servidor com a feature `mtmd` e forneça o projetor:

```bash
cargo run -p llama-crab-server --release --features mtmd -- \
  --model models/LFM2.5-VL-1.6B-Q4_K_M.gguf \
  --mmproj models/LFM2.5-VL-1.6B-mmproj-BF16.gguf
```

### Unidade estilo produção

Um arquivo de unidade `systemd` para um pequeno servidor de chat:

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

Um Dockerfile mínimo:

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

## Verificação rápida de saúde

```bash
curl http://127.0.0.1:8080/health
# → 200 OK
```

## Por onde ir a partir daqui

- [Referência da API](api.md) — cada rota, com exemplos `curl`.
- [Streaming](streaming.md) — contrato de Server-Sent Events.
- [Saída estruturada](structured.md) — os campos `response_format`,
  `grammar` e `json_schema`.
