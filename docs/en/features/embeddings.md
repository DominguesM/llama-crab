# Embeddings & reranking

`llama-crab` exposes the embedding pipeline of `llama.cpp` through a
single high-level helper, [`Llama::embed`], plus pooling and
normalisation knobs on `LlamaContextParams`. This page walks through
enabling embeddings, the four pooling modes, semantic search, and
the cross-encoder `Llama::rerank` helper.

## Enabling embeddings

Load the model with `with_embeddings(true)`. By default the context
uses **mean pooling**; pick a different strategy with
`with_pooling_type`:

```rust
use llama_crab::context::params::PoolingType;
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(
    LlamaParams::new("bge-small-en-v1.5-q4_k_m.gguf")
        .with_n_ctx(512)
        .with_embeddings(true)
        .with_pooling_type(PoolingType::Cls),
)?;
```

| Pooling | When to use it |
| --- | --- |
| `None`  | Token-level embeddings. No pooling — the output is a matrix. |
| `Mean`  | **Default.** Robust for general sentence similarity. |
| `Cls`   | BGE / GTE / E5 — uses the first token (BOS) as the summary. |
| `Last`  | Uses the last non-pad token. |
| `Rank`  | Cross-encoder rerankers. Produces a single logit per pair. |

## Computing one embedding

```rust
let v: Vec<f32> = llama.embed("Rust is memory-safe.", true)?;
//                                       normalize = true ^^^^
```

`embed(..., true)` returns an **L2-normalised** vector, so the dot
product of two vectors equals their cosine similarity. The function
returns `Result<Vec<f32>, LlamaError>`.

### Embedding options

The `embed` helper takes optional configuration through
`EmbedOptions`:

```rust
use llama_crab::embed::EmbedOptions;

let v = llama.embed_with_options(
    "Rust is memory-safe.",
    EmbedOptions::new()
        .with_normalize(true)
        .with_start_token(false)   // skip the BOS token
)?;
```

## Batch embeddings

For multi-document workloads, prefer `embed_texts` (or
`embed_texts_with_options`):

```rust
use llama_crab::Llama;

let texts = vec![
    "Rust is memory-safe.",
    "Python is a dynamic language.",
    "The Eiffel Tower is in Paris.",
];
let embeddings = llama.embed_texts(&texts, true)?;   // Vec<Vec<f32>>
```

The batch call amortises model load cost, but each text is still
evaluated independently. Use the lower-level batch and sequence APIs
when you need higher throughput.

## Semantic search

Embed a query and a corpus, then rank by cosine similarity:

```rust
let corpus = [
    "Rust is memory-safe.",
    "Paris is the capital of France.",
    "Bananas are yellow fruit.",
];
let query = "safe systems programming language";

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
```

The full example lives in
[`examples/embedding_search/`](../examples/embedding-search.md).

## Reranking

Rerankers (a.k.a. cross-encoders) score `(query, document)` pairs
**jointly** rather than from independent embeddings. They give
better rankings at the cost of one model pass per pair.

`llama-crab` includes `Llama::rerank(query, documents)` for
cross-encoder rank models. Load the model with embeddings enabled
and `PoolingType::Rank`, then pass the query and documents:

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
    "Rust prevents many memory bugs.",
    "Paris is the capital of France.",
])?;
```

The helper currently encodes each `(query, document)` pair
independently. Use the lower-level batch and sequence APIs when you
need higher throughput.

### Bi-encoder vs cross-encoder

| Method | Latency | Quality | When to use it |
| --- | --- | --- | --- |
| **Bi-encoder** (cosine on independent embeddings) | Cheap — encode each text once, then dot product. | Good for "is this in the ballpark" retrieval. | First-stage retrieval over thousands of documents. |
| **Cross-encoder** (rerank on `(query, doc)` pairs) | Expensive — one model pass per pair. | Much better at fine-grained relevance. | Second-stage reranking over the top K candidates. |

A typical pipeline uses both: a fast bi-encoder retrieves 100
candidates, then a cross-encoder reranks them.

## Building a vector index

`llama-crab` is unopinionated about the storage layer. The simplest
"index" is a `Vec<(String, Vec<f32>)>` of `(text, embedding)` pairs
held in memory. For larger corpora, pair the embeddings with one of:

- [`hnsw`](https://crates.io/crates/hnsw) — Rust-native HNSW.
- [`qdrant-client`](https://crates.io/crates/qdrant-client) — Qdrant
  vector DB.
- [`pgvector`](https://github.com/pgvector/pgvector) — Postgres with
  vector support.

The important invariant is that the index **stores L2-normalised
vectors** and the query is also normalised — then the dot product
equals cosine similarity and you can use a single index type for
both.

## Common pitfalls

| Pitfall | Symptom | Fix |
| --- | --- | --- |
| Wrong pooling type | Similarity is `NaN` or close to zero across all pairs. | BGE / GTE / E5 expect `Cls`; sentence-transformers-style models prefer `Mean`. |
| Forgot `with_embeddings(true)` | `embed` panics with "embedding mode is not enabled". | Add `.with_embeddings(true)` to the params. |
| Comparing unnormalised vectors | Similarity scores look wrong. | Pass `normalize = true` to `embed`. |
| Cross-encoder loaded with `Mean` pooling | `rerank` returns garbage scores. | Use `PoolingType::Rank` for cross-encoders. |
| Embedding model is too small for the language | Similarity scores look like noise. | Pick a model trained on the language you want to embed. |

## Where to next?

- [Embeddings example](../examples/embeddings.md) — a 30-line
  program that prints one embedding.
- [Semantic search example](../examples/embedding-search.md) —
  cosine ranking over a small corpus.
- [Reranker example](../examples/reranker.md) — a bi-encoder
  reranker demo.
- [RAG recipe](../recipes/rag.md) — combining embeddings, a vector
  store and a chat model in a single pipeline.

[`Llama::embed`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html#method.embed
