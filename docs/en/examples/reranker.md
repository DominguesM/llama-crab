# `reranker` — Bi-encoder scoring

Embeds a query and a small set of documents with an embedding model
and ranks them by cosine similarity. A bi-encoder reranker — fast,
and good enough to demonstrate the pattern before you reach for a
true cross-encoder.

## Run

=== "One-command"

    ```bash
    ./examples/run.sh reranker
    ```

=== "Manual"

    ```bash
    ./scripts/download_models.sh bge
    cargo run --release --bin reranker
    ```

Downloads `bge-small-en-v1.5-q4_k_m.gguf` (~30 MB).

## What it does

```rust
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
```

The dot product on L2-normalised embeddings equals the cosine
similarity.

## Expected output

```
query: safe systems programming language
 1. score=0.8523 document=Rust is a memory-safe systems programming language.
 2. score=0.2147 document=Bananas are yellow fruit rich in potassium.
 3. score=0.1572 document=Paris is the capital city of France.
```

## Bi-encoder vs cross-encoder

| Method | Latency | Quality | When to use it |
| --- | --- | --- | --- |
| **Bi-encoder** (this example) | Cheap — encode each text once, then dot product. | Good for "is this in the ballpark" retrieval. | First-stage retrieval over thousands of documents. |
| **Cross-encoder** (`Llama::rerank`) | Expensive — one model pass per pair. | Much better at fine-grained relevance. | Second-stage reranking over the top K candidates. |

A typical pipeline uses both: a fast bi-encoder retrieves 100
candidates, then a cross-encoder reranks them.

## Using the cross-encoder `Llama::rerank`

For higher-quality rankings, use a cross-encoder model. Load it
with `PoolingType::Rank`:

```rust
use llama_crab::context::params::PoolingType;
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(
    LlamaParams::new("bge-reranker-base-q4_k_m.gguf")
        .with_n_ctx(512)
        .with_embeddings(true)
        .with_pooling_type(PoolingType::Rank),
)?;

let scores = llama.rerank("safe systems programming", &[
    "Rust is a memory-safe systems programming language.",
    "Paris is the capital city of France.",
])?;
```

The cross-encoder encodes the `(query, document)` pair together,
so the result is a single logit per pair. Cross-encoders are
slower but more accurate.

## Full source

[`examples/reranker/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/reranker/src/main.rs).

## Where to next?

- [Embeddings & reranking guide](../features/embeddings.md) — the
  full reference, including `Llama::rerank`.
- [RAG recipe](../recipes/rag.md) — bi-encoder + cross-encoder in
  a real retrieval pipeline.
