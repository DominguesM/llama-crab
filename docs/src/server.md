# Server

`llama-crab-server` is a thin HTTP binary built on top of the safe
`llama-crab` API. It keeps inference inside the Rust binding and uses a worker
thread to own the model and context.

## Runtime shape

Keep model inference on a dedicated worker thread and send requests to it
through channels. `Llama` owns a native model and context and is intentionally
not shared freely across threads. The included binary uses this layout:

- One worker owns one `Llama` instance and processes requests sequentially.
- The HTTP router validates requests and forwards inference jobs to the worker.
- Streaming routes forward decoded chunks back to the HTTP task over a channel.

You can run several server processes or extend the crate with several workers
when you need parallel throughput.

## Routes

| HTTP route | Rust entry point |
| ---------- | ---------------- |
| `GET /health` | readiness probe |
| `GET /v1/models` | configured model name |
| `POST /v1/completions` | `Llama::create_completion_with_options` |
| `POST /v1/chat/completions` | `Llama::create_chat_completion_stream_with` |
| `POST /v1/embeddings` | `Llama::embed_texts` |
| `POST /v1/rerank` | `Llama::rerank` |
| `POST /v1/reranking` | alias for `/v1/rerank` |
| `POST /rerank` | alias for `/v1/rerank` |
| `POST /reranking` | alias for `/v1/rerank` |
| `POST /extras/tokenize` | `LlamaModel::tokenize` |
| `POST /extras/tokenize/count` | `LlamaModel::tokenize` |
| `POST /extras/detokenize` | `LlamaModel::detokenize` |

Set `"stream": true` on completion or chat requests to receive server-sent
events. Text completion chunks carry `choices[].text`; chat chunks carry
`choices[].delta.role` and `choices[].delta.content`. Normal streams finish
with `data: [DONE]`.

### Chat stream contract

`POST /v1/chat/completions` with `"stream": true` emits chunks in this order:

1. A first chunk with `choices[0].delta` containing `{"role": "assistant"}`
   and no content. This frame is sent only after the server has finished
   validating `options`, the chat prompt, and the sampler, so a malformed
   request never produces a valid role frame.
2. Zero or more content chunks with `choices[0].delta.content` set to the
   text decoded in that step and `choices[0].delta.role` omitted.
3. A terminal chunk with `choices[0].delta` equal to `{}` and
   `choices[0].finish_reason` set to `"stop"`, `"length"`, or `"tool_calls"`.
4. A final `data: [DONE]` SSE frame.

If validation fails before generation, the stream ends with an `error` SSE
event carrying the validation message and no role frame is emitted.

`GET /v1/models` returns the model configured for this process:

```json
{
  "object": "list",
  "data": [
    {
      "id": "local-model",
      "object": "model",
      "owned_by": "me",
      "permissions": []
    }
  ]
}
```

## Running

```bash
cargo run -p llama-crab-server -- \
  --model models/qwen2.5-0.5b-instruct-q4_k_m.gguf \
  --host 127.0.0.1 \
  --port 8080 \
  --n-ctx 2048 \
  --mobile-preset balanced
```

When the server is ready, a short banner is printed on stderr:

```
llama-crab-server listening on http://127.0.0.1:8080
  model : llama-crab
  routes: /health, /v1/models, /v1/completions, /v1/chat/completions,
          /v1/embeddings, /v1/rerank, /extras/tokenize,
          /extras/tokenize/count, /extras/detokenize
  ctrl+c to stop
```

The URL, model name, route list, and the same line as a `tracing::info!`
event let you confirm the server is up before sending the first
request. Set `RUST_LOG=debug` (or any other level) before launch to
override the default `info` filter.

Useful server flags also have `LLAMA_CRAB_*` environment variable equivalents:

| Flag | Environment variable | Purpose |
| ---- | -------------------- | ------- |
| `--model` | `LLAMA_CRAB_MODEL` | GGUF model path |
| `--model-name` | `LLAMA_CRAB_MODEL_NAME` | id returned by `/v1/models` |
| `--n-ctx` | `LLAMA_CRAB_N_CTX` | context size |
| `--n-batch` | `LLAMA_CRAB_N_BATCH` | logical batch size |
| `--n-threads` | `LLAMA_CRAB_N_THREADS` | CPU thread count; `0` keeps the default |
| `--n-gpu-layers` | `LLAMA_CRAB_N_GPU_LAYERS` | GPU offload layers |
| `--mobile-preset` | `LLAMA_CRAB_MOBILE_PRESET` | mobile defaults: `low-ram`, `balanced`, or `gpu-max` |
| `--embeddings` | `LLAMA_CRAB_EMBEDDINGS` | enable embedding mode |
| `--reranking` | `LLAMA_CRAB_RERANKING` | enable rerank endpoints |
| `--pooling` | `LLAMA_CRAB_POOLING` | pooling type: `none`, `mean`, `cls`, `last`, `rank`, or `unspecified` |
| `--mmproj` | `LLAMA_CRAB_MMPROJ` | multimodal projector path |

When `--mobile-preset` is set, `--n-ctx`, `--n-batch`, `--n-threads`, and
`--n-gpu-layers` override the selected preset only if they are explicitly
provided. Without a preset, the server keeps its previous defaults.

For embeddings, start the server with `--embeddings` and use an embedding
model:

```bash
cargo run -p llama-crab-server -- \
  --model models/bge-small-en-v1.5-q4_k_m.gguf \
  --embeddings
```

For reranking, start the server with `--reranking` and a model loaded with rank
pooling:

```bash
cargo run -p llama-crab-server -- \
  --model models/bge-reranker-base-q4_k_m.gguf \
  --reranking \
  --pooling rank
```

For multimodal chat, build the server with `mtmd` and provide the projector:

```bash
cargo run -p llama-crab-server --features mtmd -- \
  --model models/LFM2.5-VL-1.6B-Q4_K_M.gguf \
  --mmproj models/LFM2.5-VL-1.6B-mmproj-BF16.gguf
```

## Requests

Text generation:

```bash
curl http://127.0.0.1:8080/v1/completions \
  -H 'content-type: application/json' \
  -d '{"prompt":"The capital of France is","max_tokens":16,"temperature":0.7,"top_p":0.9,"echo":false,"logit_bias":{"42":-100.0}}'
```

Text generation for several prompts in one request:

```bash
curl http://127.0.0.1:8080/v1/completions \
  -H 'content-type: application/json' \
  -d '{"prompt":["The capital of France is","The capital of Japan is"],"max_tokens":8}'
```

Chat generation:

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"messages":[{"role":"user","content":"Explain Rust ownership briefly."}],"max_tokens":64,"template":"chatml"}'
```

Chat generation with tools:

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages":[{"role":"user","content":"Weather in Tokyo?"}],
    "template":"chatml",
    "max_tokens":96,
    "tools":[{
      "type":"function",
      "function":{
        "name":"get_weather",
        "description":"Get weather for a city",
        "parameters":{
          "type":"object",
          "properties":{"city":{"type":"string"}},
          "required":["city"]
        }
      }
    }],
    "tool_choice":{"type":"function","function":{"name":"get_weather"}}
  }'
```

Multimodal chat accepts OpenAI-style content parts. `image_url.url` currently
must be a local path or `file://` URL, and the server must be built with
`--features mtmd` and started with `--mmproj`:

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages":[{
      "role":"user",
      "content":[
        {"type":"text","text":"Describe this image in one sentence."},
        {"type":"image_url","image_url":{"url":"tests/fixtures/test_image.png"}}
      ]
    }],
    "max_tokens":64,
    "template":"chatml"
  }'
```

Embeddings:

```bash
curl http://127.0.0.1:8080/v1/embeddings \
  -H 'content-type: application/json' \
  -d '{"input":["Rust is memory-safe.","Paris is in France."],"normalize":true}'
```

Set `encoding_format` to `base64` to return each embedding as one base64 string
containing little-endian `f32` bytes:

```bash
curl http://127.0.0.1:8080/v1/embeddings \
  -H 'content-type: application/json' \
  -d '{"input":"Rust","encoding_format":"base64"}'
```

Rerank:

```bash
curl http://127.0.0.1:8080/v1/rerank \
  -H 'content-type: application/json' \
  -d '{
    "query":"safe systems programming language",
    "documents":[
      "Rust is a memory-safe systems programming language.",
      "Paris is the capital city of France.",
      "Bananas are yellow fruit."
    ],
    "top_n":2
  }'
```

Tokenizer extras:

```bash
curl http://127.0.0.1:8080/extras/tokenize \
  -H 'content-type: application/json' \
  -d '{"input":"How many tokens in this query?"}'

curl http://127.0.0.1:8080/extras/tokenize/count \
  -H 'content-type: application/json' \
  -d '{"input":"How many tokens in this query?"}'

curl http://127.0.0.1:8080/extras/detokenize \
  -H 'content-type: application/json' \
  -d '{"tokens":[1, 2, 3]}'
```

## Generation parameters

Completion and chat requests accept the same generation fields:

| Field | Default |
| ----- | ------- |
| `model` | ignored by the bundled single-model binary |
| `user` | ignored |
| `max_tokens` | `16` |
| `min_tokens` | `0` |
| `logprobs` | none for text completions, `false` for chat |
| `top_logprobs` | none for chat |
| `n` | `1` |
| `stop` | `[]`; accepts a string or a list of strings |
| `stream` | `false` |
| `echo` | `false` for text completions |
| `suffix` | none for text completions |
| `temperature` | `0.8` for text, `0.2` for chat |
| `top_k` | `40` |
| `top_p` | `0.95` |
| `tfs_z` | `1.0` |
| `min_p` | `0.05` |
| `typical_p` | `1.0` |
| `min_keep` | `1` |
| `repeat_penalty` | `1.0` |
| `frequency_penalty` | `0.0` |
| `presence_penalty` | `0.0` |
| `penalty_last_n` | `64` |
| `mirostat_mode` | `0` |
| `mirostat_tau` | `5.0` |
| `mirostat_eta` | `0.1` |
| `seed` | random |
| `logit_bias` | `{}` |
| `logit_bias_type` | `input_ids` |
| `grammar` | none |
| `json_schema` | none |
| `response_format` | none |
| `grammar_root` | `root` |

Text completion requests also accept `best_of`; it defaults to `n`, must be
greater than or equal to `n`, and controls how many internal completions are
generated before returning the public choice count. When `best_of` is greater
than `n`, candidates are ranked by the average log probability of their
generated tokens.

Set `logprobs` on text completion requests to include per-token `tokens`,
`text_offset`, `token_logprobs`, and `top_logprobs` in each returned choice.
The value controls how many top candidates are retained per generated token;
the selected token is always present in its `top_logprobs` map. Streaming text
completion chunks include the same shape for the token emitted by that chunk.

For chat requests, set `"logprobs": true` and optionally set
`"top_logprobs"` to include per-token `content` logprobs on each returned
assistant message. Streaming chat chunks include the same per-token `content`
logprob shape when requested.

Chat requests accept `tools` with `type: "function"` entries. Function tools
are rendered into the selected chat template before generation so instruct
models can emit the format they were trained to use. `tool_choice` and
`function_call` may name a declared function; unknown names are rejected before
generation. Tool execution is application-owned: the server returns structured
tool calls when the model emits them and does not invoke functions.

Chat history may include assistant tool calls and tool results:

```json
{
  "messages": [
    { "role": "user", "content": "Weather in Tokyo?" },
    {
      "role": "assistant",
      "tool_calls": [
        {
          "id": "call_weather",
          "type": "function",
          "function": {
            "name": "get_weather",
            "arguments": "{\"city\":\"Tokyo\"}"
          }
        }
      ]
    },
    {
      "role": "tool",
      "tool_call_id": "call_weather",
      "content": "{\"temperature\":22}"
    }
  ],
  "template": "chatml"
}
```

Chat `content` may be a string, `null`, or an array of content parts. Text
parts are concatenated in order. `image_url` parts are evaluated with `mtmd`
when the server is built with the `mtmd` feature and started with `--mmproj`;
without `--mmproj`, media requests fail before generation. `audio_url` and
`video_url` parse for request compatibility but are not yet evaluated by the
server generation path.

`temperature: 0.0` selects greedy decoding. Negative temperature skips the
probability filters and samples directly from the model distribution. Mirostat
mode `1` or `2` replaces the top-k/top-p/min-p/temperature chain with the
corresponding adaptive sampler.

Chat requests also accept `template`. Supported values are the built-in
template names exposed by `BuiltinTemplate`, such as `plain`, `chatml`,
`llama3`, `mistral`, and `gemma`.

Embedding and tokenizer requests may also include `model`. The bundled binary
uses the model loaded at startup; run a separate process for each served model.

`logit_bias` is a JSON object mapping token ids to additive logit values by
default. Large negative values suppress a token; positive values make it more
likely. Set `"logit_bias_type":"tokens"` to map text fragments instead; the
server tokenizes each key without a BOS token and applies the same bias to every
produced token id.

## Structured output

For structured output, pass a GBNF grammar string through `grammar`, a JSON
Schema object through `json_schema`, or a `response_format` object.
`response_format` accepts `{"type":"text"}`, `{"type":"json_object"}`, or
`{"type":"json_schema","json_schema":{"schema":...}}`; when a schema is
present, the server converts it to GBNF. The grammar sampler is chained before
the request sampling strategy.

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages":[{"role":"user","content":"Create one fictional person."}],
    "template":"chatml",
    "max_tokens":96,
    "response_format":{
      "type":"json_object",
      "schema":{
        "type":"object",
        "properties":{
          "name":{"type":"string"},
          "age":{"type":"integer"}
        },
        "required":["name","age"]
      }
    }
  }'
```

## Scope

The server crate stays thin: configuration, HTTP routing, request/response
structs, worker lifecycle, streaming transport, and errors. Inference behavior
remains in `llama-crab` so CLI, library, and server users exercise the same
implementation.
