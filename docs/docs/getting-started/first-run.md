---
title: First run
---

# First run

Create a small application and point it at a GGUF model:

```bash
cargo new hello-crab
cd hello-crab
cargo add llama-crab
```

Replace `src/main.rs`:

```rust
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(
        LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
            .with_n_ctx(2048)
            .with_n_threads(4),
    )?;

    let completion = llama.create_completion("The capital of France is", 16)?;
    println!("{}", completion.text);
    Ok(())
}
```

Run in release mode:

```bash
cargo run --release
```

If you are working from a source checkout, use the repository example wrappers
documented in [Development](../contributing/development.md).

## What success looks like

A healthy run loads the model and emits generated text. The exact text depends
on the model and sampling defaults.
