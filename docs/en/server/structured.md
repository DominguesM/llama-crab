# Structured output

For structured output, the server accepts three request fields,
applied in order of increasing strictness:

1. `grammar` — a raw GBNF string.
2. `json_schema` — a JSON Schema object (converted to GBNF).
3. `response_format` — a wrapper around `json_schema` that also
   handles `text` and `json_object` modes.

The grammar sampler is chained **before** the request sampling
strategy, so the output is always valid against the grammar.

## `response_format`

The simplest way to ask for JSON output. Three modes:

| Mode | Meaning |
| --- | --- |
| `{"type":"text"}` | Plain text, no constraints. The default. |
| `{"type":"json_object"}` | Valid JSON, but no schema enforced. |
| `{"type":"json_schema","json_schema":{"schema": ...}}` | Valid JSON that matches the given schema. |

### Example: a fictional person

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [{"role":"user","content":"Create one fictional person."}],
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

The model is forced to emit exactly:

```json
{"name": "Alice", "age": 30}
```

…or any other valid object that matches the schema. The grammar
sampler rejects tokens that would break the partial output.

## `json_schema`

A shorthand for `response_format: { type: "json_schema", … }` when
you don't need the OpenAI-style wrapper:

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [{"role":"user","content":"Create one fictional person."}],
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

`json_schema` and `response_format` are mutually exclusive in the
same request. If you pass both, the server returns `400 Bad
Request`.

## `grammar`

A raw GBNF grammar string. Use this when the output is not JSON, or
when you need a constraint that the JSON-Schema converter does not
support.

```bash
curl http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [{"role":"user","content":"Is the sky blue?"}],
    "template": "chatml",
    "max_tokens": 16,
    "grammar": "root ::= \"yes\" | \"no\""
  }'
```

The model is forced to emit exactly `yes` or `no` (followed by EOS).

## Combining with streaming

All three forms work with `"stream": true`. The chunks carry the
text as it is generated, but each token is guaranteed to keep the
output on a path to a valid grammar rule:

```bash
curl -N http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "messages": [{"role":"user","content":"Create one fictional person."}],
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

## Combining with tools

You can ask for both a JSON response and a tool call, but only one
will be emitted per turn. If the model emits a tool call, the
`finish_reason` is `"tool_calls"` and the `response_format` is
ignored for that turn. After you run the tool and re-prompt, the
model can use `response_format` to answer the next turn.

## Supported JSON-Schema features

The same subset as the safe API
([JSON-Schema & GBNF grammars](../features/grammars.md)). Highlights:

- `type: object` with `properties`, `required`,
  `additionalProperties`.
- `type: array` with `items`, `prefixItems`, `minItems`, `maxItems`.
- `type: string` with `minLength`, `maxLength`, `pattern`.
- `type: integer` / `number` with `minimum`, `maximum`, ….
- `enum`, `const`.
- `format: date-time`, `email`, `uri`, `uuid`.
- `oneOf`, `anyOf`, `allOf`.
- `$ref` (local `#/definitions/...`).

Conditional keywords (`if`, `then`, `else`) and deep recursion are
partial.

## Pitfalls

| Pitfall | What goes wrong | Fix |
| --- | --- | --- |
| Schema is empty `{}` | The model emits any JSON value. | Provide a concrete `type` at the root. |
| `response_format` and `json_schema` both set | `400 Bad Request`. | Use one or the other. |
| Grammar sampler runs but model is too small | Output is valid but semantically off. | Increase model size or improve the prompt. |
| Schema has no `required` fields | Output may omit important keys. | Add `required` to the schema. |
| `format: "email"` and the model emits `a@b` | Some models consider `a@b` valid; some don't. | Use `pattern: "^[^@]+@[^@]+$"` for stricter validation. |

## Where to next?

- [JSON-Schema & GBNF grammars](../features/grammars.md) — the
  underlying safe-API guide.
- [API reference](api.md) — the full request schema.
- [Streaming](streaming.md) — the SSE contract.
