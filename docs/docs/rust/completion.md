---
title: Text Completion
---

# Text completion

Use text completion when you already have the exact prompt string you want to
send to the model.

## Minimal completion

```rust
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("models/model.gguf").with_n_ctx(512))?;
let response = llama.create_completion("Once upon a time", 64)?;
println!("{}", response.text);
```

This mirrors the `simple` example in the `llama-crab-examples` repository.

## Completion options

For stop sequences, logprobs, suffixes, prompt echoing, and sampler settings,
use `CompletionOptions`:

```rust
use llama_crab::{CompletionOptions, Llama, LlamaParams, SamplingOptions};

let mut llama = Llama::load(LlamaParams::new("models/model.gguf"))?;
let options = CompletionOptions::new(96)
    .with_stop_sequence("\n\n")
    .with_logprobs(3)
    .with_sampling(SamplingOptions::chat().with_temperature(0.2));

let response = llama.create_completion_with_options("Explain Rust ownership:", options)?;
println!("{}", response.text);
```

`CompletionOptions::new` defaults to greedy decoding (`temperature = 0.0`).
`CompletionOptions::sampled` starts from the probabilistic sampler defaults.

## Streaming

Streaming calls your callback once per chunk. Return `StreamControl::Stop` to
stop early.

```rust
use std::io::{self, Write};
use llama_crab::{CompletionOptions, Llama, LlamaParams, StreamControl};

let mut llama = Llama::load(LlamaParams::new("models/model.gguf"))?;
let mut out = io::stdout().lock();

let final_completion = llama.create_completion_stream(
    "Write one short sentence about Rust.",
    CompletionOptions::new(64),
    |chunk| {
        let _ = write!(out, "{}", chunk.text);
        let _ = out.flush();
        StreamControl::Continue
    },
)?;
```

See the `streaming` example in `llama-crab-examples` for the full
error-handling version.

## Custom sampler

If the built-in sampling options are not enough, build a sampler chain and pass
it to `create_completion_with_sampler`.

```rust
use llama_crab::sampling::LlamaSampler;
use llama_crab::{CompletionOptions, Llama};

let greedy = LlamaSampler::greedy().expect("greedy sampler");
let mut sampler = LlamaSampler::chain(vec![greedy], false).expect("sampler chain");

let response = llama.create_completion_with_sampler(
    "Return one JSON object:",
    CompletionOptions::new(64),
    &mut sampler,
)?;
```

The generated API reference for these types starts at
[`/llama-crab/api/rust/llama_crab/`](https://dominguesm.github.io/llama-crab/api/rust/llama_crab/).
