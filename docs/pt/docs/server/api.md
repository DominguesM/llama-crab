# Referência da API

O servidor expõe uma API HTTP compatível com OpenAI. Esta página
documenta cada rota, o formato da requisição, o formato da resposta
e os códigos de status. Exemplos `curl` trabalhados estão
incluídos para cada rota.

## Rotas

| Rota HTTP | Método | Ponto de entrada em Rust |
| --- | --- | --- |
| `/health` | `GET` | Probe de prontidão. |
| `/v1/models` | `GET` | Nome do modelo configurado. |
| `/v1/completions` | `POST` | `Llama::create_completion_with_options`. |
| `/v1/chat/completions` | `POST` | `Llama::create_chat_completion_stream_with`. |
| `/v1/embeddings` | `POST` | `Llama::embed_texts`. |
| `/v1/rerank` | `POST` | `Llama::rerank`. |
| `/v1/reranking` | `POST` | Alias para `/v1/rerank`. |
| `/rerank` | `POST` | Alias para `/v1/rerank`. |
| `/reranking` | `POST` | Alias para `/v1/rerank`. |
| `/extras/tokenize` | `POST` | `LlamaModel::tokenize`. |
| `/extras/tokenize/count` | `POST` | `LlamaModel::tokenize`. |
| `/extras/detokenize` | `POST` | `LlamaModel::detokenize`. |

Defina `"stream": true` em requisições de completion ou chat para
receber Server-Sent Events. Chunks de completion de texto carregam
`choices[].text`; chunks de chat carregam `choices[].delta.role` e
`choices[].delta.content`. Streams normais terminam com
`data: [DONE]`.

## `GET /health`

Probe de prontidão. Retorna `200 OK` assim que o modelo é
carregado.

```bash
curl http://127.0.0.1:8080/health
```

## `GET /v1/models`

Retorna o modelo configurado para este processo:

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

Completion de texto simples.

=== "Prompt único"

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

=== "Múltiplos prompts"

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

### Campos da requisição

| Campo | Padrão | Descrição |
| --- | --- | --- |
| `prompt` | _obrigatório_ | Uma string ou array de strings. |
| `max_tokens` | `16` | Número máximo de tokens a serem gerados. |
| `min_tokens` | `0` | Número mínimo de tokens a serem gerados. |
| `temperature` | `0.8` | `0.0` seleciona decodificação greedy. |
| `top_k` | `40` | Amostragem Top-K. |
| `top_p` | `0.95` | Amostragem Top-P. |
| `tfs_z` | `1.0` | Amostragem tail-free. |
| `min_p` | `0.05` | Amostragem Min-P. |
| `typical_p` | `1.0` | Amostragem localmente típica. |
| `min_keep` | `1` | Tokens mínimos a manter após filtragem. |
| `repeat_penalty` | `1.0` | Penalidade de repetição. |
| `frequency_penalty` | `0.0` | Penalidade de frequência. |
| `presence_penalty` | `0.0` | Penalidade de presença. |
| `penalty_last_n` | `64` | Tokens a considerar para penalidades. |
| `mirostat_mode` | `0` | Modo Mirostat (`0`, `1`, `2`). |
| `mirostat_tau` | `5.0` | Perplexidade alvo do Mirostat. |
| `mirostat_eta` | `0.1` | Taxa de aprendizado do Mirostat. |
| `seed` | random | Semente do RNG. |
| `logit_bias` | `{}` | Id de token → bias de logit aditivo. |
| `logit_bias_type` | `input_ids` | `input_ids` ou `tokens`. |
| `grammar` | – | Gramática GBNF bruta. |
| `json_schema` | – | JSON Schema (convertido para GBNF). |
| `response_format` | – | `text`, `json_object` ou `json_schema`. |
| `grammar_root` | `root` | Regra raiz da gramática GBNF. |
| `stop` | `[]` | String ou lista de strings. |
| `stream` | `false` | Server-Sent Events. |
| `echo` | `false` | Ecoa o prompt na resposta. |
| `suffix` | – | Sufixo anexado após o prompt. |
| `best_of` | `n` | Número de candidatos internos para `n`. |
| `logprobs` | `false` | Log-probabilidades por token. |
| `top_logprobs` | `0` | Top-K logprobs por token. |
| `n` | `1` | Número de escolhas a retornar. |
| `model` | ignorado | Incluído para compatibilidade com cliente OpenAI. |
| `user` | ignorado | Incluído para compatibilidade com cliente OpenAI. |

## `POST /v1/chat/completions`

Completion de chat multi-turno.

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

### Chat com tools

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

### Chat multimodal

`image_url.url` deve ser um caminho local ou URL `file://`. O
servidor deve ser compilado com `--features mtmd` e iniciado com
`--mmproj`:

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

### Chat com tool calls anteriores

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

### Campos da requisição de chat

Requisições de chat aceitam os mesmos campos de geração que
completion de texto, mais:

| Campo | Padrão | Descrição |
| --- | --- | --- |
| `messages` | _obrigatório_ | Lista de mensagens `{role, content}`. |
| `template` | `plain` | `plain`, `chatml`, `llama3`, `mistral`, `gemma`, … |
| `tools` | `[]` | Lista de definições de `function`. |
| `tool_choice` | `auto` | `none`, `auto` ou uma função específica. |
| `function_call` | – | Parâmetro legado da OpenAI. |
| `top_logprobs` | `0` | Top-K logprobs por token. |
| `logprobs` | `false` | Log-probabilidades por token. |

O `content` do chat pode ser uma string, `null` ou um array de
partes de conteúdo. Partes de texto são concatenadas em ordem.
Partes `image_url` são avaliadas com `mtmd` quando o servidor é
compilado com a feature `mtmd` e iniciado com `--mmproj`.
`audio_url` e `video_url` fazem parse para compatibilidade de
requisição mas ainda não são avaliados pelo caminho de geração
do servidor.

## `POST /v1/embeddings`

```bash
curl http://127.0.0.1:8080/v1/embeddings \
  -H 'content-type: application/json' \
  -d '{
    "input": ["Rust is memory-safe.", "Paris is in France."],
    "normalize": true
  }'
```

Defina `encoding_format` como `base64` para retornar cada embedding
como uma única string base64 contendo bytes `f32` little-endian:

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

## `POST /extras/tokenize` e `/extras/tokenize/count`

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

## Códigos de status

| Código | Quando |
| --- | --- |
| `200 OK` | Sucesso. |
| `400 Bad Request` | JSON malformado, campo desconhecido, template inválido, schema que falha ao compilar. |
| `404 Not Found` | Rota desconhecida. |
| `422 Unprocessable Entity` | O modelo rejeitou a requisição (ex. `tool_choice` nomeia uma função desconhecida). |
| `500 Internal Server Error` | Um erro interno (raro; geralmente significa que o modelo não está carregado). |
| `503 Service Unavailable` | O modelo ainda está carregando. |

## Por onde ir a partir daqui

- [Streaming](streaming.md) — o contrato de Server-Sent Events.
- [Saída estruturada](structured.md) — `response_format`,
  `grammar` e `json_schema`.
- [Executando o servidor](running.md) — flags de boot e presets.
