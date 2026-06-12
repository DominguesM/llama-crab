# `simple` — Plain text completion

```rust,no_run
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

Run with:

```bash
cargo run --bin simple --release -- model.gguf
```

## Expected output

```
, there was a little girl who loved to read.
```

(approximate — actual output depends on the model)
