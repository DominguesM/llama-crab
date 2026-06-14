---
title: Embeddings and Reranking Examples
---

# Embeddings and reranking examples

Use these examples with embedding or reranking GGUFs. A normal chat model is not
enough for embedding extraction.

## Raw embeddings

```bash
./examples/run.sh embeddings
```

Loads `models/bge-small-en-v1.5-q4_k_m.gguf`, enables embeddings on the context,
embeds one text, and prints:

- token IDs,
- embedding dimension,
- L2 norm,
- the first few vector values.

Direct form:

```bash
cargo run --release --bin embeddings -- \
  models/bge-small-en-v1.5-q4_k_m.gguf \
  "Hello, world!"
```

## Semantic search

```bash
./examples/run.sh embedding_search
```

Embeds a query and a tiny fixed corpus, then sorts documents by cosine
similarity. The example uses `PoolingType::Cls` for BGE-small.

You can pass a custom query:

```bash
./examples/run.sh embedding_search -- "Which language checks memory safety?"
```

## Embedding-based reranker

```bash
./examples/run.sh reranker
```

Ranks a small document list by embedding similarity to the fixed query
`safe systems programming language`.

## HTTP rerank server

```bash
./examples/run.sh rerank
```

Starts `llama-crab-server` with the BGE reranker model and:

```text
--reranking --pooling rank
```

The route is `POST /v1/rerank`.

## Troubleshooting note

If these examples behave differently across machines or branches, compare release
versus debug mode, backend feature flags, pooling type, and model file. Recent
local validation showed that embedding dimensions and stability can vary with
branch/backend/build mode, so use the wrapper's release run as the baseline
before debugging application code.
