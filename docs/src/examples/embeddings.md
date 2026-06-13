# `embeddings` — Embedding extraction

A minimal example that loads an embedding GGUF, tokenizes text, runs
a single forward pass with `with_embeddings(true)` and prints the
L2-norm and a small preview of the resulting vector.

## Run

```bash
./examples/run.sh embeddings
# or, manually:
./scripts/download_models.sh bge
cargo run --release --bin run_embeddings
```

Downloads `bge-small-en-v1.5-q4_k_m.gguf` (~30 MB).

## What it does

```rust,no_run
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(
    LlamaParams::new("models/bge-small-en-v1.5-q4_k_m.gguf")
        .with_n_ctx(512)
        .with_embeddings(true),
)?;

let text = "Hello, world!";
let embedding = llama.embed(text, true)?; // true = L2-normalize

let norm = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
println!("dim={}", embedding.len());
println!("l2_norm={norm:.6}");
# Ok::<(), Box<dyn std::error::Error>>(())
```

For semantic search (query a corpus by cosine similarity), see
[`embedding_search`](./embedding_search.md).

## Expected output

```
text: Hello, world!
embedding_dim: 384
embedding_l2_norm: 1.000000
embedding_preview: [0.012345, -0.006789, ...]
```

## Full source

[`examples/embeddings/src/main.rs`][src].

[src]: https://github.com/DominguesM/llama-crab/tree/main/examples/embeddings/src/main.rs
