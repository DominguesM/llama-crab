# `simple` — Plain text completion

The smallest possible program: load a model, generate a completion,
print the result. Use it as a starting point for a one-shot CLI
tool or as a template when you want full control over the sampler
chain.

## Run

```bash
cargo run --bin simple --release -- model.gguf
```

The first positional argument is the path to a GGUF model.

## What it does

```rust
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(
        LlamaParams::new("model.gguf")
            .with_n_ctx(2048)
            .with_n_gpu_layers(99),
    )?;
    let resp = llama.create_completion("Once upon a time", 64)?;
    println!("{}", resp.text);
    Ok(())
}
```

## Expected output

```
, there was a little girl who loved to read.
```

The actual text depends on the model. The point of the example is
the *shape* of the program: a few lines, no ceremony, all the
defaults applied.

## Customising the call

Pass [`CompletionOptions`] to the high-level helper to expose the
rest of the sampler chain:

```rust
use llama_crab::{CompletionOptions, Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(LlamaParams::new("model.gguf"))?;
    let resp = llama.create_completion_with_options(
        "Once upon a time",
        CompletionOptions::new(64)
            .with_temperature(0.7)
            .with_top_p(0.9, 1)
            .with_stop_sequence("\n\n"),
    )?;
    print!("{}", resp.text);
    Ok(())
}
```

See the [text completion guide](../features/text-completion.md) for
the full menu of options.

## Using a custom sampler chain

For full control, build a [`SamplerChain`] and call
`create_completion_with_sampler`:

```rust
use llama_crab::sampling::SamplerChain;
use llama_crab::{CompletionOptions, Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(LlamaParams::new("model.gguf"))?;
    let mut sampler = SamplerChain::new()
        .temp(0.7)
        .top_p(0.9, 1)
        .min_p(0.05, 1)
        .penalties(64, 1.1, 0.0, 0.0)
        .build();
    let resp = llama.create_completion_with_sampler(
        "Once upon a time",
        CompletionOptions::new(64),
        &mut sampler,
    )?;
    print!("{}", resp.text);
    Ok(())
}
```

See the [sampling strategies guide](../guides/sampling.md) for the
full menu of samplers and recommended chains.

## Full source

[`examples/simple/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/simple/src/main.rs).

## Where to next?

- [Streaming](streaming.md) — for token-by-token output.
- [Chat](chat.md) — when you want role-based messages.
- [FIM](#) — fill-in-the-middle for code completion.

[`CompletionOptions`]: https://docs.rs/llama-crab/latest/llama_crab/struct.CompletionOptions.html
[`SamplerChain`]: https://docs.rs/llama-crab/latest/llama_crab/sampling/struct.SamplerChain.html
