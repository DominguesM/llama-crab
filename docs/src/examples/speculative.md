# `speculative` — Prompt-lookup draft decoding

Demonstrates [`PromptLookupDecoding`]: scan the prompt for the
last `n` tokens and emit what followed them as a draft. No extra
model required.

## Run

```bash
./examples/run.sh speculative
# or, manually:
./scripts/download_models.sh smol
cargo run --release --bin speculative
```

Downloads `Qwen2.5-0.5B-Instruct-GGUF` (~400 MB).

## What it does

```rust,no_run
use llama_crab::speculative::{DraftModel, PromptLookupDecoding};
use llama_crab::{Llama, LlamaParams};

let llama = Llama::load(LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
    .with_n_ctx(1024))?;

let prompt = "Rust is fast and memory safe. Rust is fast";
let prompt_tokens = llama.model().tokenize(prompt, true, true)?;

let draft = PromptLookupDecoding::new(3, 8);
let drafted = draft.draft(&prompt_tokens, 8);

let drafted_text = llama.model().detokenize(&drafted, false)?;
println!("drafted text> {}", drafted_text.trim());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The prompt contains a repetition (`"Rust is fast"` appears twice), so
the draft picks up `"and memory safe"` and emits it as the candidate
next tokens.

## Expected output

```
prompt> Rust is fast and memory safe. Rust is fast
drafted token ids> [Token(...), Token(...), ...]
drafted text> and memory safe
```

## When this helps

- Repetitive prompts (code, lists, RAG that quotes the context)
- FIM infill (the body of a function appears earlier in the file)
- Long templated prompts where the system prompt repeats

For open-ended creative writing, acceptance drops and the overhead
can exceed the savings — measure before adopting.

## Full source

[`examples/speculative/src/main.rs`][src].

[`PromptLookupDecoding`]: https://docs.rs/llama-crab/latest/llama_crab/speculative/struct.PromptLookupDecoding.html
[src]: https://github.com/DominguesM/llama-crab/tree/main/examples/speculative/src/main.rs
