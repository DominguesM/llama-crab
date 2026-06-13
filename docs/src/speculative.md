# Speculative decoding

Speculative decoding trades a small amount of extra compute for a
large speedup on agreement-heavy workloads. A cheap **draft** step
proposes `n` candidate tokens; the main model verifies them in a
single forward pass. Accepted tokens cost one decode step each, just
like greedy sampling — but you also got them "for free" if the draft
was right.

`llama-crab` exposes two flavours in the [`speculative`] module:

| Flavour                  | What you provide                            |
| ------------------------ | ------------------------------------------- |
| [`PromptLookupDecoding`] | Nothing — drafts from n-grams in the prompt. |
| Your own `DraftModel`    | A smaller model, a trie, a regex, …          |

## Prompt-lookup n-gram

`PromptLookupDecoding` is a zero-config draft model. It looks for the
last `k` tokens of the current sequence earlier in the prompt and
emits whatever followed them as the draft. Works extremely well on
code, lists, and any output that repeats a pattern from the input.

```rust,no_run
use llama_crab::speculative::{DraftModel, PromptLookupDecoding};
use llama_crab::{Llama, LlamaParams};

let llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(2048))?;
let prompt = "Rust is fast and memory safe. Rust is fast";
let tokens = llama.model().tokenize(prompt, true, true)?;

let draft = PromptLookupDecoding::new(3, 8);
let drafted = draft.draft(&tokens, 8);
# let _ = drafted;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Tuning knobs:

- `max_ngram_size` — how many trailing tokens form the lookup key. `2`
  –`4` is a good starting point.
- `num_pred_tokens` — how many tokens to emit when a match is found.

## Custom draft models

Implement the [`DraftModel`] trait for anything that can propose
tokens — a smaller quantized GGUF loaded into a second `Llama`, a
regex automaton, a finite-state machine, …

```rust,no_run
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

## Driving a speculative step

The free function [`speculative_decode`] feeds the draft through the
main context, samples at every position, accepts the longest matching
prefix and returns the accepted tokens.

```rust,no_run
# use llama_crab::speculative::{DraftModel, speculative_decode};
# use llama_crab::sampling::LlamaSampler;
# use llama_crab::token::LlamaToken;
# let main_ctx: *mut llama_crab_sys::llama_context = std::ptr::null_mut();
# let mut sampler = LlamaSampler::greedy()?;
# let draft = llama_crab::speculative::PromptLookupDecoding::new(2, 4);
# let history: Vec<LlamaToken> = Vec::new();

let accepted: Vec<LlamaToken> = unsafe {
    speculative_decode(main_ctx, &mut sampler, &draft, &history, 4)
};
# let _ = accepted;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The function is `unsafe` because `main_ctx` must point at a live,
unaliased context owned by the caller. The high-level [`Llama`]
orchestrator exposes the raw handle through
`llama.context().raw_handle()` when you need it.

## When it helps

- **High draft acceptance** — repetitive inputs, FIM, structured
  output, RAG answers that quote the prompt.
- **Cheap draft step** — n-gram lookups are nanoseconds; a small
  draft model should be 5–10× smaller than the main model.
- **Single-user latency** — throughput gains disappear under batching
  because the main model is already busy.

Where it **doesn't** help: open-ended creative generation (acceptance
drops below ~50%), tiny models (the overhead eats the savings), or
heavily batched servers.

## Where to next?

- [`speculative` example](./examples/speculative.md)
- [Sampling guide](./sampling.md)

[`speculative`]: https://docs.rs/llama-crab/latest/llama_crab/speculative/index.html
[`DraftModel`]: https://docs.rs/llama-crab/latest/llama_crab/speculative/trait.DraftModel.html
[`PromptLookupDecoding`]: https://docs.rs/llama-crab/latest/llama_crab/speculative/struct.PromptLookupDecoding.html
[`speculative_decode`]: https://docs.rs/llama-crab/latest/llama_crab/speculative/fn.speculative_decode.html
[`Llama`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html
