# `embeddings` — Embedding extraction

A minimal example that loads an embedding GGUF, tokenizes text, runs
a single forward pass with `with_embeddings(true)` and prints the
L2-norm and a small preview of the resulting vector.

## Run

=== "One-command"

    ```bash
    ./examples/run.sh embeddings
    ```

=== "Manual"

    ```bash
    ./scripts/download_models.sh bge
    cargo run --release --bin run_embeddings
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

let text = "Hello, world!";
let embedding = llama.embed(text, true)?;   // true = L2-normalize

let norm = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
println!("dim={}", embedding.len());
println!("l2_norm={norm:.6}");
```

The L2-normalised vector has `norm = 1.0` (within float precision),
so the dot product of two vectors equals their cosine similarity.

## Expected output

```
text: Hello, world!
embedding_dim: 384
embedding_l2_norm: 1.000000
embedding_preview: [0.012345, -0.006789, ...]
```

## Pooling type

BGE / GTE / E5 expect CLS pooling — the first token (BOS) is the
summary. Use:

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
| `PoolingType::None` | Token-level embeddings (no pooling). |
| `PoolingType::Mean` | Default. Sentence-transformers-style models. |
| `PoolingType::Cls` | BGE / GTE / E5. |
| `PoolingType::Last` | Last non-pad token. |
| `PoolingType::Rank` | Cross-encoder rerankers. |

## Batch embeddings

For multi-document workloads, use `embed_texts`:

```rust
let texts = vec!["Rust is memory-safe.", "Python is dynamic."];
let embeddings = llama.embed_texts(&texts, true)?;
println!("dim={}", embeddings[0].len());
```

## Common variations

=== "Different pooling"

    ```rust
    use llama_crab::context::params::PoolingType;
    LlamaParams::new("bge-small-en-v1.5-q4_k_m.gguf")
        .with_embeddings(true)
        .with_pooling_type(PoolingType::Mean)
    ```

=== "Skip the BOS token"

    ```rust
    use llama_crab::embed::EmbedOptions;
    let v = llama.embed_with_options(
        "Hello, world!",
        EmbedOptions::new().with_start_token(false),
    )?;
    ```

## Pitfalls

- **Forgot `with_embeddings(true)`** — `embed` panics with
  "embedding mode is not enabled".
- **Wrong pooling type** — similarity is `NaN` or close to zero
  across all pairs. BGE / GTE / E5 want `Cls`; sentence-transformers
  want `Mean`.
- **Comparing unnormalised vectors** — similarity scores look
  wrong. Pass `normalize = true` to `embed`.

## Full source

[`examples/embeddings/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/embeddings/src/main.rs).

## Where to next?

- [Semantic search](embedding-search.md) — cosine ranking over a
  small corpus.
- [Reranker](reranker.md) — bi-encoder ranking demo.
- [Embeddings & reranking guide](../features/embeddings.md) — the
  full reference.
- [RAG recipe](../recipes/rag.md) — embeddings in a retrieval
  pipeline.
