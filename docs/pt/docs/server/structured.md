# Saída estruturada

Para saída estruturada, o servidor aceita três campos de requisição,
aplicados em ordem de estrita crescente:

1. `grammar` — uma string GBNF bruta.
2. `json_schema` — um objeto JSON Schema (convertido para GBNF).
3. `response_format` — um wrapper em torno de `json_schema` que
   também lida com os modos `text` e `json_object`.

O sampler de gramática é encadeado **antes** da estratégia de
amostragem da requisição, então a saída é sempre válida contra a
gramática.

## `response_format`

A maneira mais simples de pedir saída JSON. Três modos:

| Modo | Significado |
| --- | --- |
| `{"type":"text"}` | Texto simples, sem restrições. O padrão. |
| `{"type":"json_object"}` | JSON válido, mas sem schema imposto. |
| `{"type":"json_schema","json_schema":{"schema": ...}}` | JSON válido que corresponde ao schema dado. |

### Exemplo: uma pessoa fictícia

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [{"role":"user","content":"Crie uma pessoa fictícia."}],
    "template": "chatml",
    "max_tokens": 96,
    "response_format": {
      "type": "json_schema",
      "json_schema": {
        "schema": {
          "type": "object",
          "properties": {
            "name": { "type": "string" },
            "age":  { "type": "integer" }
          },
          "required": ["name", "age"]
        }
      }
    }
  }'
```

O modelo é forçado a emitir exatamente:

```json
{"name": "Alice", "age": 30}
```

…ou qualquer outro objeto válido que corresponda ao schema. O
sampler de gramática rejeita tokens que quebrariam a saída parcial.

## `json_schema`

Um atalho para `response_format: { type: "json_schema", … }` quando
você não precisa do wrapper no estilo OpenAI:

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [{"role":"user","content":"Crie uma pessoa fictícia."}],
    "template": "chatml",
    "max_tokens": 96,
    "json_schema": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "age":  { "type": "integer" }
      },
      "required": ["name", "age"]
    }
  }'
```

`json_schema` e `response_format` são mutuamente exclusivos na mesma
requisição. Se você passar ambos, o servidor retorna `400 Bad
Request`.

## `grammar`

Uma string de gramática GBNF bruta. Use quando a saída não é JSON,
ou quando você precisa de uma restrição que o conversor
JSON-Schema não suporta.

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [{"role":"user","content":"O céu é azul?"}],
    "template": "chatml",
    "max_tokens": 16,
    "grammar": "root ::= \"sim\" | \"não\""
  }'
```

O modelo é forçado a emitir exatamente `sim` ou `não` (seguido de
EOS).

## Combinando com streaming

Todas as três formas funcionam com `"stream": true`. Os chunks
carregam o texto conforme é gerado, mas cada token é garantido a
manter a saída em um caminho para uma regra de gramática válida:

```bash
curl -N http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [{"role":"user","content":"Crie uma pessoa fictícia."}],
    "template": "chatml",
    "max_tokens": 96,
    "stream": true,
    "response_format": {
      "type": "json_schema",
      "json_schema": {
        "schema": {
          "type": "object",
          "properties": {
            "name": { "type": "string" },
            "age":  { "type": "integer" }
          },
          "required": ["name", "age"]
        }
      }
    }
  }'
```

## Combinando com tools

Você pode pedir tanto uma resposta JSON quanto uma tool call, mas
apenas uma será emitida por turno. Se o modelo emite uma tool
call, o `finish_reason` é `"tool_calls"` e o `response_format` é
ignorado para esse turno. Depois que você executa a tool e
re-prompta, o modelo pode usar `response_format` para responder o
próximo turno.

## Features JSON-Schema suportadas

O mesmo subconjunto que a API segura
([JSON-Schema & gramáticas GBNF](../features/grammars.md)).
Destaques:

- `type: object` com `properties`, `required`, `additionalProperties`.
- `type: array` com `items`, `prefixItems`, `minItems`, `maxItems`.
- `type: string` com `minLength`, `maxLength`, `pattern`.
- `type: integer` / `number` com `minimum`, `maximum`, ….
- `enum`, `const`.
- `format: date-time`, `email`, `uri`, `uuid`.
- `oneOf`, `anyOf`, `allOf`.
- `$ref` (local `#/definitions/...`).

Palavras-chave condicionais (`if`, `then`, `else`) e recursão
profunda são parciais.

## Armadilhas

| Armadilha | O que dá errado | Correção |
| --- | --- | --- |
| Schema é vazio `{}` | O modelo emite qualquer valor JSON. | Forneça um `type` concreto na raiz. |
| `response_format` e `json_schema` ambos definidos | `400 Bad Request`. | Use um ou outro. |
| Sampler de gramática roda mas o modelo é pequeno demais | Saída é válida mas semanticamente off. | Aumente o tamanho do modelo ou melhore o prompt. |
| Schema sem campos `required` | Saída pode omitir chaves importantes. | Adicione `required` ao schema. |
| `format: "email"` e o modelo emite `a@b` | Alguns modelos consideram `a@b` válido; outros não. | Use `pattern: "^[^@]+@[^@]+$"` para validação mais estrita. |

## Por onde ir a partir daqui

- [JSON-Schema & gramáticas GBNF](../features/grammars.md) — o
  guia da API segura subjacente.
- [Referência da API](api.md) — o schema completo da requisição.
- [Streaming](streaming.md) — o contrato SSE.
