# `structured` — Constrained JSON output

The example uses a JSON Schema → GBNF grammar to force the model to
emit only valid JSON of a specific shape.

```rust,no_run
use llama_crab::high_level::completion::json_schema_grammar;
use llama_crab::high_level::completion::CompletionOptions;
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
    let grammar_text = json_schema_grammar(&schema).unwrap();
    let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(1024))?;
    let grammar = unsafe { LlamaSampler::grammar(llama.model(), &grammar_text, "root")? };
    let greedy = LlamaSampler::greedy()
        .ok_or_else(|| std::io::Error::other("failed to create greedy sampler"))?;
    let mut sampler = LlamaSampler::chain(vec![grammar, greedy], false)
        .ok_or_else(|| std::io::Error::other("failed to create sampler chain"))?;
    let resp = llama.create_completion_with_sampler(
        "Generate a fictional person as JSON: ",
        CompletionOptions::new(32),
        &mut sampler,
    )?;
    println!("{}", resp.text);
    Ok(())
}
```

Run with:

```bash
cargo run -p structured --release -- model.gguf
```

## Expected output

```
{"name": "Alice", "age": 30}
```
