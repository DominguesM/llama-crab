# Embeddings & reranking

`llama-crab` exposes the embedding pipeline of `llama.cpp` through a
single high-level helper — [`Llama::embed`] — plus pooling and
normalization knobs on `LlamaContextParams`.

## Enabling embeddings

Load the model with `with_embeddings(true)`. By default the context
uses **mean pooling**; pick a different strategy with
`with_pooling_type`:

```rust,no_run
use llama_crab::context::params::PoolingType;
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(
    LlamaParams::new("bge-small-en-v1.5-q4_k_m.gguf")
        .with_n_ctx(512)
        .with_embeddings(true)
        .with_pooling_type(PoolingType::Cls),
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

| Pooling | When to use                                                  |
| ------- | ------------------------------------------------------------ |
| `None`  | Token-level embeddings (no pooling).                          |
| `Mean`  | Default. Robust for general sentence similarity.              |
| `Cls`   | BGE / GTE / E5 — uses the first token (BOS) as the summary.   |
| `Last`  | Uses the last non-pad token.                                 |

## Computing one embedding

```rust,no_run
# use llama_crab::{Llama, LlamaParams};
# let mut llama = Llama::load(LlamaParams::new("m.gguf").with_embeddings(true))?;
let v: Vec<f32> = llama.embed("Rust is memory-safe.", true)?;
//                                       normalize = true ^^^^
# Ok::<(), Box<dyn std::error::Error>>(())
```

`embed(..., true)` returns an **L2-normalized** vector, so the dot
product of two vectors equals their cosine similarity.

## Semantic search

Embed a query and a corpus, then rank by cosine similarity:

```rust,no_run
# use llama_crab::{Llama, LlamaParams};
# let mut llama = Llama::load(LlamaParams::new("m.gguf").with_embeddings(true))?;

let corpus = ["Rust is memory-safe.", "Paris is the capital of France."];
let query  = "safe systems programming language";

let q = llama.embed(query, true)?;
let mut scored: Vec<(usize, f32)> = corpus.iter().enumerate()
    .map(|(i, doc)| {
        let v = llama.embed(doc, true).unwrap();
        let sim: f32 = q.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
        (i, sim)
    })
    .collect();
scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

for (i, sim) in &scored {
    println!("{sim:.3}  {}", corpus[*i]);
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

For a runnable program, see
[`embedding_search`](./examples/embedding_search.md).

## Reranking

Rerankers (a.k.a. cross-encoders) score `(query, document)` pairs
**jointly** rather than from independent embeddings. They give better
rankings at the cost of one model pass per pair.

`llama-crab` includes `Llama::rerank(query, documents)` for cross-encoder
rank models. Load the model with embeddings enabled and `PoolingType::Rank`,
then pass the query and documents:

```rust,no_run
# use llama_crab::context::params::PoolingType;
# use llama_crab::{Llama, LlamaParams};
# let mut llama = Llama::load(
#     LlamaParams::new("reranker.gguf")
#         .with_embeddings(true)
#         .with_pooling_type(PoolingType::Rank)
# )?;
let scores = llama.rerank("safe systems programming", &[
    "Rust prevents many memory bugs.",
    "Paris is the capital of France.",
])?;
# let _ = scores;
# Ok::<(), Box<dyn std::error::Error>>(())
```

The helper currently encodes each `(query, document)` pair independently.
Use the lower-level batch and sequence APIs when you need higher throughput.

## Where to next?

- [`embedding_search` example](./examples/embedding_search.md)
- [`reranker` example](./examples/reranker.md)
- [Reference](./reference.md)

[`Llama::embed`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html#method.embed
