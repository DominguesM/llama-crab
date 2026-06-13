# `embedding_search` — Semantic search

Embeds a query and a tiny fixed corpus with a BGE model, then ranks
the corpus by cosine similarity. The classic "given a query, find the
closest documents" workflow in ~50 lines.

## Run

```bash
./examples/run.sh embedding_search
# or, manually:
./scripts/download_models.sh bge
cargo run --release --bin run_embeddings
```

Downloads `bge-small-en-v1.5-q4_k_m.gguf` (~30 MB).

## What it does

```rust,no_run
use llama_crab::context::params::PoolingType;
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(
    LlamaParams::new("models/bge-small-en-v1.5-q4_k_m.gguf")
        .with_n_ctx(512)
        .with_embeddings(true)
        .with_pooling_type(PoolingType::Cls),
)?;

let corpus = &[
    "Rust is a memory-safe systems language without a garbage collector.",
    "Python is a high-level dynamic language with duck typing.",
    "The Eiffel Tower is one of the most visited monuments in the world.",
    "Borrow checking enforces lifetimes at compile time in Rust.",
];

let q = llama.embed("What programming language is safest?", true)?;
let mut scored: Vec<(usize, f32)> = corpus.iter().enumerate()
    .map(|(i, doc)| {
        let v = llama.embed(doc, true).unwrap();
        let sim: f32 = q.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
        (i, sim)
    })
    .collect();
scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Because the embeddings are L2-normalized, the dot product equals the
cosine similarity.

## Expected output

```
📊 results (cosine similarity, higher = more similar):
   0.823  doc-1  Rust is a memory-safe systems language without a garbage collector.
   0.741  doc-4  Borrow checking enforces lifetimes at compile time in Rust.
   0.312  doc-2  Python is a high-level dynamic language with duck typing.
   0.088  doc-3  The Eiffel Tower is one of the most visited monuments in the world.

Query: What programming language is safest?
Top match: doc-1 (cosine = 0.823)
```

## Full source

[`examples/embedding_search/src/main.rs`][src].

[src]: https://github.com/DominguesM/llama-crab/tree/main/examples/embedding_search/src/main.rs
