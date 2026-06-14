# JSON-Schema & GBNF grammars

Constrained decoding is the most reliable way to get a model to emit
structured output. `llama-crab` ships with a pure-Rust JSON-Schema →
GBNF converter that supports a useful subset of [JSON Schema
2020-12](https://json-schema.org/draft/2020-12/json-schema-core.html).

## Quickstart

```rust,no_run
use llama_crab::high_level::completion::json_schema_grammar;
use serde_json::json;

let schema = json!({
    "type": "object",
    "properties": {
        "name": { "type": "string" },
        "age":  { "type": "integer" }
    },
    "required": ["name", "age"]
});
let grammar = json_schema_grammar(&schema).unwrap();
# let _ = grammar;
```

Then plug the grammar into a sampler. Grammar enforcement is a sampler
stage like any other and is available with the `common` cargo feature:

```rust,no_run
# #[cfg(feature = "common")] {
# use llama_crab::high_level::completion::{json_schema_grammar, CompletionOptions};
# use llama_crab::sampling::LlamaSampler;
# use llama_crab::{Llama, LlamaParams};
# use serde_json::json;
# let schema = json!({"type":"object"});
# let grammar = json_schema_grammar(&schema)?;
# let mut llama = Llama::load(LlamaParams::new("model.gguf"))?;

let grammar = unsafe { LlamaSampler::grammar(llama.model(), &grammar, "root")? };
let greedy = LlamaSampler::greedy()
    .ok_or_else(|| std::io::Error::other("failed to create greedy sampler"))?;
let mut sampler = LlamaSampler::chain(vec![grammar, greedy], false)
    .ok_or_else(|| std::io::Error::other("failed to create sampler chain"))?;
let completion = llama.create_completion_with_sampler(
    "Return one object: ",
    CompletionOptions::new(64),
    &mut sampler,
)?;
# let _ = completion;
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

For a runnable program that parses JSON after generation, see the
[`structured`](./examples/structured.md) example.

## Supported JSON-Schema features

- `type`: `object`, `array`, `string`, `integer`, `number`, `boolean`, `null`
- `properties`, `required`, `additionalProperties` (with sub-schema)
- `items` (single-schema) and `prefixItems`/`minItems`/`maxItems`
- `enum` (string / integer / boolean / null)
- `const`
- `minimum`, `maximum`, `exclusiveMinimum`, `exclusiveMaximum`
- `minLength`, `maxLength`, `pattern`
- `format`: `date-time`, `email`, `uri`, `uuid`
- `oneOf`, `anyOf`, `allOf`
- `$ref` (local, `#/definitions/...` style)
- `definitions` and `$defs`

## Custom grammars

For full control, build a GBNF string by hand and pass it directly to
the `grammar` sampler (gated by the `common` feature).
