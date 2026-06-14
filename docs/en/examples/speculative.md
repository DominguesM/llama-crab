# `speculative` — Prompt-lookup draft decoding

Demonstrates [`PromptLookupDecoding`]: scan the prompt for the last
`n` tokens and emit what followed them as a draft. No extra model
required.

## Run

=== "One-command"

    ```bash
    ./examples/run.sh speculative
    ```

=== "Manual"

    ```bash
    ./scripts/download_models.sh smol
    cargo run --release --bin speculative
    ```

Downloads `Qwen2.5-0.5B-Instruct-GGUF` (~400 MB).

## What it does

```rust
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
```

The prompt contains a repetition (`"Rust is fast"` appears twice),
so the draft picks up `"and memory safe"` and emits it as the
candidate next tokens.

## Expected output

```
prompt> Rust is fast and memory safe. Rust is fast
drafted token ids> [Token(...), Token(...), ...]
drafted text> and memory safe
```

## Tuning knobs

| Knob | Description | Typical range |
| --- | --- | --- |
| `max_ngram_size` | How many trailing tokens form the lookup key. | 2–4 |
| `num_pred_tokens` | How many tokens to emit when a match is found. | 4–16 |

Larger `max_ngram_size` finds more matches but is more sensitive
to small edits. Larger `num_pred_tokens` reduces the verification
overhead per accepted token, but a wrong draft is more expensive
to recover from.

## When this helps

- **Repetitive prompts** — code, lists, RAG that quotes the
  context.
- **FIM infill** — the body of a function appears earlier in the
  file.
- **Long templated prompts** — the system prompt repeats.

For open-ended creative writing, acceptance drops and the overhead
can exceed the savings. Measure before adopting.

## Custom draft models

The `DraftModel` trait lets you plug in any token-proposing
strategy — a smaller model, a regex automaton, a finite-state
machine, a trie of common phrases:

```rust
use llama_crab::speculative::DraftModel;
use llama_crab::token::LlamaToken;

struct AlwaysHello;
impl DraftModel for AlwaysHello {
    fn draft(&self, _input: &[LlamaToken], n: usize) -> Vec<LlamaToken> {
        // Replace with: sample n tokens from your smaller model.
        Vec::new()
    }
}
```

Then drive the speculative step with the [`speculative_decode`]
free function.

## When acceptance is too low

A few rules of thumb:

| Symptom | Likely cause | Fix |
| --- | --- | --- |
| Acceptance < 30 % | The prompt is not repetitive enough. | Try a different draft model (a small instruct GGUF). |
| Acceptance > 80 % but speedup is small | The draft step is too slow. | Use a smaller draft, or `PromptLookupDecoding`. |
| Speedup is negative | The main model is already small. | Speculative decoding rarely helps sub-1B models. |

## Full source

[`examples/speculative/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/speculative/src/main.rs).

## Where to next?

- [Speculative decoding guide](../features/speculative.md) — the
  full reference, including custom draft models.
- [Performance tuning recipe](../recipes/performance.md) —
  measure throughput with and without speculative decoding.

[`PromptLookupDecoding`]: https://docs.rs/llama-crab/latest/llama_crab/speculative/struct.PromptLookupDecoding.html
[`speculative_decode`]: https://docs.rs/llama-crab/latest/llama_crab/speculative/fn.speculative_decode.html
