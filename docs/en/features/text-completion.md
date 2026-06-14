# Text completion

The simplest `llama-crab` workflow: feed the model a prompt string,
get a text continuation back. This page documents the
`create_completion` family of methods, the stop-sequence and
log-probability knobs, streaming, best-of-N, and FIM (fill-in-the-
middle) for code.

## The basic call

```rust
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(2048))?;
let resp = llama.create_completion("Once upon a time", 64)?;
println!("{}", resp.text);
```

The [`Completion`] struct returned by the high-level call carries:

| Field | Type | Description |
| --- | --- | --- |
| `text` | `String` | The generated text, with the prompt stripped. |
| `tokens` | `Vec<LlamaToken>` | The generated token ids. |
| `logprobs` | `Option<CompletionLogprobs>` | Per-token log probabilities, when requested. |
| `timings` | `CompletionTimings` | Prompt ingestion, generation, and total wall time. |
| `stop_reason` | `StopReason` | Why generation stopped (`Stop`, `Length`, `TokensLimit`, `Canceled`). |

## Customising the call

`create_completion` is a thin wrapper over
`create_completion_with_options`, which takes a [`CompletionOptions`]
builder. Use it to expose the rest of the sampler chain, the stop
sequences, the log-prob settings and the best-of-N knob.

```rust
use llama_crab::{CompletionOptions, Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(2048))?;

let resp = llama.create_completion_with_options(
    "The capital of France is",
    CompletionOptions::new(32)
        .with_temperature(0.7)
        .with_top_p(0.95, 1)
        .with_top_k(40)
        .with_stop_sequence("\n\n")
        .with_logprobs(true, 5)
        .with_echo(false),
)?;
```

### `CompletionOptions` reference

| Method | Default | Description |
| --- | --- | --- |
| `new(max_tokens)` | – | Sets the maximum number of tokens to generate. |
| `with_temperature(t)` | `0.8` | Temperature; `0.0` selects greedy decoding. |
| `with_top_k(k)` | `40` | Restrict to the top K tokens. |
| `with_top_p(p, min_keep)` | `0.95, 1` | Nucleus sampling. |
| `with_min_p(p, min_keep)` | `0.05, 1` | Min-P sampling. |
| `with_typical_p(p, min_keep)` | `1.0, 1` | Locally-typical sampling. |
| `with_tfs_z(z)` | `1.0` | Tail-free sampling. |
| `with_repeat_penalty(p)` | `1.0` | Repetition penalty. |
| `with_frequency_penalty(p)` | `0.0` | Frequency penalty. |
| `with_presence_penalty(p)` | `0.0` | Presence penalty. |
| `with_penalty_last_n(n)` | `64` | Tokens to consider when applying penalties. |
| `with_mirostat(...)` | `0` | Mirostat mode (`0` = off, `1`, `2`). |
| `with_seed(seed)` | random | RNG seed. |
| `with_stop_sequence(s)` | – | Adds a single stop sequence. |
| `with_stop_sequences([s])` | – | Adds multiple stop sequences. |
| `with_logit_bias(biases)` | `{}` | Manual logit bias. |
| `with_logprobs(enable, k)` | `false, 0` | Per-token log probabilities. |
| `with_echo(echo)` | `false` | Echo the prompt back as part of the response. |
| `with_suffix(suffix)` | – | Suffix appended after the prompt (for FIM). |
| `with_best_of(n)` | `n` | Number of internal candidates for `n`. |
| `with_grammar(text)` | – | GBNF grammar (requires the `common` feature). |

## Streaming

For real-time UIs, use `create_completion_stream`:

```rust
use std::io::{self, Write};
use llama_crab::{CompletionOptions, Llama, LlamaParams, StreamControl};

let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(512))?;
let prompt = "Write one short sentence about Rust.";
let mut stdout = io::stdout().lock();
let mut write_error: Option<io::Error> = None;

let completion = llama.create_completion_stream(
    prompt,
    CompletionOptions::new(64).with_stop_sequence("\n\n"),
    |chunk| {
        if let Err(err) = write!(stdout, "{}", chunk.text).and_then(|_| stdout.flush()) {
            write_error = Some(err);
            return StreamControl::Stop;
        }
        StreamControl::Continue
    },
)?;

if let Some(err) = write_error {
    return Err(err.into());
}
```

The callback cannot return a `Result`, so capture I/O errors and
return `StreamControl::Stop`; after the stream returns, propagate
the captured error.

See the [streaming example](../examples/streaming.md) for a
self-contained program.

## FIM (fill-in-the-middle) for code

Code-completion models expect a `prefix<SUFFIX_FILL>middle<SUFFIX>`-style
prompt. `llama-crab` exposes `complete_infill` for that:

```rust
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(1024))?;

let prefix = "fn main() {\n    println!(\"";
let suffix = "\");\n}";
let resp = llama.complete_infill(prefix, suffix)?;
println!("{}", resp.text);
```

The `complete_infill` helper renders a `BuiltinTemplate::CodeFim`
template around the prefix and suffix, then runs the normal
generation loop. Make sure the model you're using has been trained
on a FIM task — the GGUF metadata usually declares it.

## Best-of-N

`with_best_of(n)` generates `n` internal completions, scores them
by average log probability, and returns the top `n` (the public
choice count). Use it to trade compute for quality:

```rust
use llama_crab::{CompletionOptions, Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(1024))?;
let resp = llama.create_completion_with_options(
    "def fibonacci(n):",
    CompletionOptions::new(64).with_best_of(4),
)?;
```

The returned `Completion` has the same shape as a single completion;
the extra candidates are internal.

## Log probabilities

`with_logprobs(true, k)` populates `Completion.logprobs` with the
top `k` token log probabilities at every position, plus the
selected token:

```rust
pub struct CompletionLogprobs {
    pub tokens:           Vec<LlamaToken>,
    pub text_offset:      Vec<usize>,
    pub token_logprobs:   Vec<Option<f32>>,
    pub top_logprobs:     Vec<Vec<TopLogprob>>,
    pub top_logprobs_idx: Vec<usize>,
}
```

Use it for uncertainty estimates, perplexity scoring, or to
implement a custom UI that shows alternatives.

## Stop sequences

Stop sequences are matched **after** a token is decoded, on the
detokenised chunk. Add as many as you want; the first one to match
ends the generation. They are case-sensitive and whitespace-significant.

```rust
CompletionOptions::new(64)
    .with_stop_sequence("\n\n")
    .with_stop_sequence("</answer>")
    .with_stop_sequence("User:")
```

The default behaviour matches against any of them.

## Echoing the prompt

`with_echo(true)` returns the prompt as part of the generated text.
Useful for debugging, less useful in production.

## Performance tips

- **Reuse the model.** Each `Llama::load` is O(seconds). Load once,
  call many times.
- **Tune `n_threads` to physical cores.** A 16-core Mac with
  hyperthreading should use 8–12 threads, not 16.
- **Use `temp = 0.0` for benchmarks.** It picks the greedy sampler
  and gives the most reproducible timings.
- **Offload to the GPU when the model fits in VRAM.** A
  `with_n_gpu_layers(99)` call is usually 5–10× faster than CPU.
- **Set `with_seed` to a fixed value for unit tests.** The default
  RNG seed is random per call, which makes test assertions flaky.

## Where to next?

- [Sampling strategies](../guides/sampling.md) — for full control
  over the sampler chain.
- [Streaming example](../examples/streaming.md) — a self-contained
  program that streams tokens to stdout.
- [Speculative decoding](speculative.md) — when you need more
  tokens per second than the model can natively produce.

[`Completion`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Completion.html
[`CompletionOptions`]: https://docs.rs/llama-crab/latest/llama_crab/struct.CompletionOptions.html
