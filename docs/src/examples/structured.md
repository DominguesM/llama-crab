# `structured` — Constrained JSON output

The example uses a JSON Schema → GBNF grammar to force the model to
emit only valid JSON of a specific shape.

```rust,no_run
use llama_crab::high_level::completion::json_schema_grammar;
use llama_crab::sampling::{LlamaSampler, SamplerChain};
use llama_crab::{Llama, LlamaParams};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age":  { "type": "integer" }
        },
        "required": ["name", "age"]
    });
    let _grammar = json_schema_grammar(&schema).unwrap();
    // In v0.2 the grammar is fed to a grammar sampler (gated by
    // `common` feature). For v0.1 the JSON is parsed post-hoc.
    let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(1024))?;
    let resp = llama.create_completion(
        "Generate a fictional person as JSON: ",
        32,
    )?;
    println!("{}", resp.text);
    Ok(())
}
```

Run with:

```bash
cargo run --bin structured --release -- model.gguf
```

## Expected output

```
{"name": "Alice", "age": 30}
```
