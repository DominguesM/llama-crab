# `reranker` — Pontuação bi-encoder

Incorpora uma query e um conjunto pequeno de documentos com um
modelo de embedding e os ranqueia por similaridade cosseno. Um
reranker bi-encoder — rápido, e bom o suficiente para demonstrar o
padrão antes de partir para um cross-encoder real.

## Execute

=== "Um comando"

    ```bash
    ./examples/run.sh reranker
    ```

=== "Manual"

    ```bash
    ./scripts/download_models.sh bge
    cargo run --release --bin reranker
    ```

Baixa o `bge-small-en-v1.5-q4_k_m.gguf` (~30 MB).

## O que ele faz

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

O produto escalar em embeddings L2-normalizados é igual à
similaridade cosseno.

## Saída esperada

```
query: safe systems programming language
 1. score=0.8523 document=Rust is a memory-safe systems programming language.
 2. score=0.2147 document=Bananas are yellow fruit rich in potassium.
 3. score=0.1572 document=Paris is the capital city of France.
```

## Bi-encoder vs cross-encoder

| Método | Latência | Qualidade | Quando usar |
| --- | --- | --- | --- |
| **Bi-encoder** (este exemplo) | Barato — codifique cada texto uma vez, depois produto escalar. | Bom para recuperação "está na faixa". | Recuperação de primeiro estágio sobre milhares de documentos. |
| **Cross-encoder** (`Llama::rerank`) | Caro — uma passada de modelo por par. | Muito melhor em relevância fina. | Reranking de segundo estágio sobre o top K de candidatos. |

Um pipeline típico usa ambos: um bi-encoder rápido recupera 100
candidatos, depois um cross-encoder os reranqueia.

## Usando o cross-encoder `Llama::rerank`

Para ranqueamentos de maior qualidade, use um modelo cross-encoder.
Carregue com `PoolingType::Rank`:

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

O cross-encoder codifica o par `(query, document)` junto, então o
resultado é um único logit por par. Cross-encoders são mais
lentos mas mais precisos.

## Código-fonte completo

[`examples/reranker/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/reranker/src/main.rs).

## Por onde ir a partir daqui

- [Guia de embeddings & reranking](../features/embeddings.md) — a
  referência completa, incluindo `Llama::rerank`.
- [Receita de RAG](../recipes/rag.md) — bi-encoder + cross-encoder
  em um pipeline de recuperação real.
