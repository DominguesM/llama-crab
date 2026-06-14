---
title: Embeddings
---

# Embeddings

Embedding extraction uses a model loaded with embeddings enabled. Use an
embedding GGUF, not a regular chat model.

## Single embedding

```rust
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(
    LlamaParams::new("models/bge-small-en-v1.5-q4_k_m.gguf")
        .with_n_ctx(512)
        .with_embeddings(true),
)?;

let embedding = llama.embed("Rust is a systems programming language.", true)?;
println!("dim = {}", embedding.len());
```

The second argument controls L2 normalization. The high-level helper tokenizes
the text, encodes it, reads sequence embeddings, and optionally normalizes the
result.

## Pooling

Some embedding models require a specific pooling strategy. The
`embedding_search` example uses BGE-small with CLS pooling:

```rust
use llama_crab::context::params::PoolingType;
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(
    LlamaParams::new("models/bge-small-en-v1.5-q4_k_m.gguf")
        .with_n_ctx(512)
        .with_embeddings(true)
        .with_pooling_type(PoolingType::Cls),
)?;
```

## Similarity search

Compute one query embedding, embed your corpus, and sort by cosine similarity.
For normalized vectors, cosine similarity is the dot product. If you request raw
vectors, divide by both vector norms.

## Reranking

The server has a `/v1/rerank` route when started with `--reranking` and a
reranking model:

```bash
llama-crab-server \
  --model /models/reranker.gguf \
  --reranking \
  --pooling rank
```

## Operational caveat

Embedding and reranking behavior can depend on backend, model metadata, pooling,
and build mode. If vectors have an unexpected dimension, scores collapse near
zero, or a backend crashes, isolate the embedding path first and compare pooling
and backend settings before changing unrelated chat-generation code.

Repository embedding and reranking examples are covered in
[Development](../contributing/development.md).
