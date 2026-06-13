# `reranker` — Cross-encoder scoring

Embeds a query and a small set of documents with an embedding model
and ranks them by cosine similarity. A bi-encoder reranker — fast, and
good enough to demonstrate the pattern before you reach for a true
cross-encoder.

## Run

```bash
./examples/run.sh reranker
# or, manually:
./scripts/download_models.sh bge
cargo run --release --bin reranker
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

let query = "safe systems programming language";
let documents = [
    "Rust is a memory-safe systems programming language.",
    "Paris is the capital city of France.",
    "Bananas are yellow fruit rich in potassium.",
];

let q = llama.embed(query, true)?;
let mut scored: Vec<(f32, &str)> = documents.iter().map(|doc| {
    let v = llama.embed(doc, true).unwrap();
    let sim: f32 = q.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
    (sim, *doc)
}).collect();
scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The dot product on L2-normalized embeddings equals the cosine
similarity.

## Expected output

```
query: safe systems programming language
 1. score=0.8523 document=Rust is a memory-safe systems programming language.
 2. score=0.2147 document=Bananas are yellow fruit rich in potassium.
 3. score=0.1572 document=Paris is the capital city of France.
```

## Full source

[`examples/reranker/src/main.rs`][src].

[src]: https://github.com/DominguesM/llama-crab/tree/main/examples/reranker/src/main.rs
