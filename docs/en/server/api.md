# API reference

The server exposes an OpenAI-compatible HTTP API. This page
documents every route, the request shape, the response shape, and
the status codes. Worked `curl` examples are included for each
route.

## Routes

| HTTP route | Method | Rust entry point |
| --- | --- | --- |
| `/health` | `GET` | Readiness probe. |
| `/v1/models` | `GET` | Configured model name. |
| `/v1/completions` | `POST` | `Llama::create_completion_with_options`. |
| `/v1/chat/completions` | `POST` | `Llama::create_chat_completion_stream_with`. |
| `/v1/embeddings` | `POST` | `Llama::embed_texts`. |
| `/v1/rerank` | `POST` | `Llama::rerank`. |
| `/v1/reranking` | `POST` | Alias for `/v1/rerank`. |
| `/rerank` | `POST` | Alias for `/v1/rerank`. |
| `/reranking` | `POST` | Alias for `/v1/rerank`. |
| `/extras/tokenize` | `POST` | `LlamaModel::tokenize`. |
| `/extras/tokenize/count` | `POST` | `LlamaModel::tokenize`. |
| `/extras/detokenize` | `POST` | `LlamaModel::detokenize`. |

Set `"stream": true` on completion or chat requests to receive
Server-Sent Events. Text completion chunks carry `choices[].text`;
chat chunks carry `choices[].delta.role` and `choices[].delta.content`.
Normal streams finish with `data: [DONE]`.

## `GET /health`

Readiness probe. Returns `200 OK` once the model is loaded.

```bash
curl http://127.0.0.1:8080/health
```

## `GET /v1/models`

Returns the model configured for this process:

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

## `POST /v1/completions`

Plain text completion.

=== "Single prompt"

    ```bash
    curl http://127.0.0.1:8080/v1/completions \
      -H 'content-type: application/json' \
      -d '{
        "prompt": "The capital of France is",
        "max_tokens": 16,
        "temperature": 0.7,
        "top_p": 0.9,
        "echo": false,
        "logit_bias": {"42": -100.0}
      }'
    ```

=== "Multiple prompts"

    ```bash
    curl http://127.0.0.1:8080/v1/completions \
      -H 'content-type: application/json' \
      -d '{
        "prompt": [
          "The capital of France is",
          "The capital of Japan is"
        ],
        "max_tokens": 8
      }'
    ```

### Request fields

| Field | Default | Description |
| --- | --- | --- |
| `prompt` | _required_ | A string or an array of strings. |
| `max_tokens` | `16` | Maximum number of tokens to generate. |
| `min_tokens` | `0` | Minimum number of tokens to generate. |
| `temperature` | `0.8` | `0.0` selects greedy decoding. |
| `top_k` | `40` | Top-K sampling. |
| `top_p` | `0.95` | Top-P sampling. |
| `tfs_z` | `1.0` | Tail-free sampling. |
| `min_p` | `0.05` | Min-P sampling. |
| `typical_p` | `1.0` | Locally-typical sampling. |
| `min_keep` | `1` | Minimum tokens to keep after filtering. |
| `repeat_penalty` | `1.0` | Repetition penalty. |
| `frequency_penalty` | `0.0` | Frequency penalty. |
| `presence_penalty` | `0.0` | Presence penalty. |
| `penalty_last_n` | `64` | Tokens to consider for penalties. |
| `mirostat_mode` | `0` | Mirostat mode (`0`, `1`, `2`). |
| `mirostat_tau` | `5.0` | Mirostat target perplexity. |
| `mirostat_eta` | `0.1` | Mirostat learning rate. |
| `seed` | random | RNG seed. |
| `logit_bias` | `{}` | Token id → additive logit bias. |
| `logit_bias_type` | `input_ids` | `input_ids` or `tokens`. |
| `grammar` | – | Raw GBNF grammar. |
| `json_schema` | – | JSON Schema (converted to GBNF). |
| `response_format` | – | `text`, `json_object`, or `json_schema`. |
| `grammar_root` | `root` | Root rule of the GBNF grammar. |
| `stop` | `[]` | String or list of strings. |
| `stream` | `false` | Server-Sent Events. |
| `echo` | `false` | Echo the prompt in the response. |
| `suffix` | – | Suffix appended after the prompt. |
| `best_of` | `n` | Number of internal candidates for `n`. |
| `logprobs` | `false` | Per-token log probabilities. |
| `top_logprobs` | `0` | Top-K logprobs per token. |
| `n` | `1` | Number of choices to return. |
| `model` | ignored | Included for OpenAI client compatibility. |
| `user` | ignored | Included for OpenAI client compatibility. |

## `POST /v1/chat/completions`

Multi-turn chat completion.

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [
      { "role": "user", "content": "Explain Rust ownership briefly." }
    ],
    "max_tokens": 64,
    "template": "chatml"
  }'
```

### Chat with tools

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [
      { "role": "user", "content": "Weather in Tokyo?" }
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
          "properties": { "city": { "type": "string" } },
          "required": ["city"]
        }
      }
    }],
    "tool_choice": {
      "type": "function",
      "function": { "name": "get_weather" }
    }
  }'
```

### Multimodal chat

`image_url.url` must be a local path or `file://` URL. The server
must be built with `--features mtmd` and started with `--mmproj`:

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [{
      "role": "user",
      "content": [
        { "type": "text", "text": "Describe this image in one sentence." },
        { "type": "image_url", "image_url": { "url": "tests/fixtures/test_image.png" } }
      ]
    }],
    "max_tokens": 64,
    "template": "chatml"
  }'
```

### Chat with prior tool calls

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
      "content": "{\"temperature\": 22}"
    }
  ],
  "template": "chatml"
}
```

### Chat request fields

Chat requests accept the same generation fields as text completion,
plus:

| Field | Default | Description |
| --- | --- | --- |
| `messages` | _required_ | List of `{role, content}` messages. |
| `template` | `plain` | `plain`, `chatml`, `llama3`, `mistral`, `gemma`, … |
| `tools` | `[]` | List of `function` definitions. |
| `tool_choice` | `auto` | `none`, `auto`, or a specific function. |
| `function_call` | – | Legacy OpenAI parameter. |
| `top_logprobs` | `0` | Top-K logprobs per token. |
| `logprobs` | `false` | Per-token log probabilities. |

Chat `content` may be a string, `null`, or an array of content
parts. Text parts are concatenated in order. `image_url` parts are
evaluated with `mtmd` when the server is built with the `mtmd`
feature and started with `--mmproj`. `audio_url` and `video_url`
parse for request compatibility but are not yet evaluated by the
server generation path.

## `POST /v1/embeddings`

```bash
curl http://127.0.0.1:8080/v1/embeddings \
  -H 'content-type: application/json' \
  -d '{
    "input": ["Rust is memory-safe.", "Paris is in France."],
    "normalize": true
  }'
```

Set `encoding_format` to `base64` to return each embedding as a
single base64 string containing little-endian `f32` bytes:

```bash
curl http://127.0.0.1:8080/v1/embeddings \
  -H 'content-type: application/json' \
  -d '{
    "input": "Rust",
    "encoding_format": "base64"
  }'
```

## `POST /v1/rerank`

```bash
curl http://127.0.0.1:8080/v1/rerank \
  -H 'content-type: application/json' \
  -d '{
    "query": "safe systems programming language",
    "documents": [
      "Rust is a memory-safe systems programming language.",
      "Paris is the capital city of France.",
      "Bananas are yellow fruit."
    ],
    "top_n": 2
  }'
```

## `POST /extras/tokenize` and `/extras/tokenize/count`

```bash
curl http://127.0.0.1:8080/extras/tokenize \
  -H 'content-type: application/json' \
  -d '{"input": "How many tokens in this query?"}'

curl http://127.0.0.1:8080/extras/tokenize/count \
  -H 'content-type: application/json' \
  -d '{"input": "How many tokens in this query?"}'
```

## `POST /extras/detokenize`

```bash
curl http://127.0.0.1:8080/extras/detokenize \
  -H 'content-type: application/json' \
  -d '{"tokens": [1, 2, 3]}'
```

## Status codes

| Code | When |
| --- | --- |
| `200 OK` | Success. |
| `400 Bad Request` | Malformed JSON, unknown field, invalid template, schema that fails to compile. |
| `404 Not Found` | Unknown route. |
| `422 Unprocessable Entity` | The model rejected the request (e.g. `tool_choice` names an unknown function). |
| `500 Internal Server Error` | An internal error (rare; usually means the model is not loaded). |
| `503 Service Unavailable` | The model is still loading. |

## Where to next?

- [Streaming](streaming.md) — the Server-Sent Events contract.
- [Structured output](structured.md) — `response_format`,
  `grammar` and `json_schema`.
- [Running the server](running.md) — boot flags and presets.
