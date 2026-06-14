---
title: Structured Output
---

# Structured output

`llama-crab` supports grammar-constrained generation. For JSON, the common path
is to convert a JSON Schema into GBNF and attach it to the sampler chain.

## JSON Schema to grammar

```rust
use llama_crab::high_level::completion::json_schema_grammar;
use serde_json::json;

let schema = json!({
    "type": "object",
    "properties": {
        "name": { "type": "string" },
        "age": { "type": "integer" }
    },
    "required": ["name", "age"]
});

let grammar_text = json_schema_grammar(&schema)?;
```

## Constrained completion

```rust
use llama_crab::chat::{render_builtin, BuiltinTemplate};
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::high_level::completion::CompletionOptions;
use llama_crab::sampling::LlamaSampler;
use llama_crab::{Llama, LlamaParams, Role};

let mut llama = Llama::load(LlamaParams::new("models/model.gguf").with_n_ctx(1024))?;
let grammar = unsafe { LlamaSampler::grammar(llama.model(), &grammar_text, "root")? };
let greedy = LlamaSampler::greedy().expect("greedy sampler");
let mut sampler = LlamaSampler::chain(vec![grammar, greedy], false).expect("sampler chain");

let messages = vec![
    ChatMessage::new(Role::System, "Return only JSON."),
    ChatMessage::new(Role::User, "Create one fictional person."),
];
let prompt = render_builtin(BuiltinTemplate::ChatMl, &messages, &[], true);

let response = llama.create_completion_with_sampler(
    &prompt,
    CompletionOptions::new(96),
    &mut sampler,
)?;
```

After generation, parse and validate the response with `serde_json` or the
schema validator used by your application.

## Tools versus structured output

Use structured output when the response itself must match a schema. Use a
tool-call pattern when the model should choose or populate an action that your
Rust code will execute. The `examples/tools` program demonstrates a lightweight
tool pattern using JSON extraction and validation in Rust.
