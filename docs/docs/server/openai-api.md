# OpenAI-Compatible API

The server uses OpenAI-style routes and response objects where the current runtime supports them. It is not a full OpenAI API implementation; unsupported or invalid combinations are returned as request errors.

## Models

```bash
curl http://127.0.0.1:8080/v1/models
```

The response is a list containing the configured `--model-name`.

## Chat completions

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "local-chat",
    "messages": [
      { "role": "system", "content": "Answer briefly." },
      { "role": "user", "content": "What is llama-crab?" }
    ],
    "max_tokens": 128,
    "temperature": 0.7
  }'
```

Supported message roles are `system`, `user`, `assistant`, `tool`, and `function`. The `function` role is normalized to `tool`. Unknown roles are rejected.

Chat requests support:

| Field | Notes |
| --- | --- |
| `messages` | Required. Content may be a string or content parts. |
| `max_tokens`, `min_tokens` | Generation length controls. Default `max_tokens` is `16`. |
| `temperature`, `top_k`, `top_p`, `min_p`, `typical_p`, `tfs_z` | Sampling controls. |
| `repeat_penalty`, `frequency_penalty`, `presence_penalty`, `penalty_last_n` | Penalty controls. |
| `mirostat_mode`, `mirostat_tau`, `mirostat_eta` | Mirostat controls. |
| `seed` | Optional deterministic sampling seed. |
| `stop` | String or array of strings. |
| `n` | Number of choices. Must be greater than zero. |
| `template` | Built-in chat template name. Defaults to ChatML. |
| `tools`, `tool_choice`, `function_call` | Function-tool metadata and selection validation. |
| `logprobs`, `top_logprobs` | Chat logprobs when supported by the text path. |
| `grammar`, `json_schema`, `response_format`, `grammar_root` | Structured-output controls. |
| `logit_bias`, `logit_bias_type` | Token biasing by `input_ids` or by text `tokens`. |

## Text completions

```bash
curl http://127.0.0.1:8080/v1/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "local",
    "prompt": "Write one sentence about local inference:",
    "max_tokens": 64,
    "stop": ["\n"]
  }'
```

`prompt` may be a single string or an array of strings. Non-streaming completions support `n` and `best_of`; `best_of` must be greater than or equal to `n`.

## Embeddings

Start the server with `--embeddings` or `--reranking`, then call:

```bash
curl http://127.0.0.1:8080/v1/embeddings \
  -H 'content-type: application/json' \
  -d '{
    "model": "local-embed",
    "input": ["first document", "second document"],
    "normalize": true,
    "encoding_format": "float"
  }'
```

`encoding_format` may be `float` or `base64`. The default output is a float array. `normalize` defaults to `true`.

## Reranking

Start with `--reranking`, then call any rerank alias:

```bash
curl http://127.0.0.1:8080/v1/rerank \
  -H 'content-type: application/json' \
  -d '{
    "model": "local-reranker",
    "query": "local inference",
    "documents": ["Runs on your machine", "Hosted API"],
    "top_n": 1
  }'
```

The response sorts results by descending `relevance_score` and includes the original document index.

## Extras

```bash
curl http://127.0.0.1:8080/extras/tokenize \
  -H 'content-type: application/json' \
  -d '{ "input": "hello" }'

curl http://127.0.0.1:8080/extras/tokenize/count \
  -H 'content-type: application/json' \
  -d '{ "input": "hello" }'

curl http://127.0.0.1:8080/extras/detokenize \
  -H 'content-type: application/json' \
  -d '{ "tokens": [1, 2, 3] }'
```

## Structured output

For completions and chat completions, the server can build a grammar from:

- `grammar`: raw grammar text. This takes precedence when non-empty.
- `json_schema`: JSON Schema converted to a grammar.
- `response_format: { "type": "json_object" }`: object grammar, or schema-backed object grammar when `schema` is present.
- `response_format: { "type": "json_schema", "json_schema": { "schema": ... } }`: schema-backed grammar.

`grammar_root` defaults to `root`.
