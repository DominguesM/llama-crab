# `embeddings` — Extração de embedding

Um exemplo mínimo que carrega um GGUF de embedding, tokeniza texto,
executa uma única passada forward com `with_embeddings(true)` e
imprime a L2-norm e uma pequena prévia do vetor resultante.

## Execute

=== "Um comando"

    ```bash
    ./examples/run.sh embeddings
    ```

=== "Manual"

    ```bash
    ./scripts/download_models.sh bge
    cargo run --release --bin run_embeddings
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

let text = "Hello, world!";
let embedding = llama.embed(text, true)?;   // true = L2-normaliza

let norm = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
println!("dim={}", embedding.len());
println!("l2_norm={norm:.6}");
```

O vetor L2-normalizado tem `norm = 1.0` (dentro da precisão de
float), então o produto escalar de dois vetores é igual à
similaridade cosseno.

## Saída esperada

```
text: Hello, world!
embedding_dim: 384
embedding_l2_norm: 1.000000
embedding_preview: [0.012345, -0.006789, ...]
```

## Tipo de pooling

BGE / GTE / E5 esperam pooling CLS — o primeiro token (BOS) é o
resumo. Use:

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

| Pooling | Quando usar |
| --- | --- |
| `PoolingType::None` | Embeddings em nível de token (sem pooling). |
| `PoolingType::Mean` | Padrão. Modelos estilo sentence-transformers. |
| `PoolingType::Cls` | BGE / GTE / E5. |
| `PoolingType::Last` | Último token não-pad. |
| `PoolingType::Rank` | Rerankers cross-encoder. |

## Embeddings em batch

Para cargas de trabalho multi-documento, use `embed_texts`:

```rust
let texts = vec!["Rust é memory-safe.", "Python é dinâmica."];
let embeddings = llama.embed_texts(&texts, true)?;
println!("dim={}", embeddings[0].len());
```

## Variações comuns

=== "Pooling diferente"

    ```rust
    use llama_crab::context::params::PoolingType;
    LlamaParams::new("bge-small-en-v1.5-q4_k_m.gguf")
        .with_embeddings(true)
        .with_pooling_type(PoolingType::Mean)
    ```

=== "Pular o token BOS"

    ```rust
    use llama_crab::embed::EmbedOptions;
    let v = llama.embed_with_options(
        "Hello, world!",
        EmbedOptions::new().with_start_token(false),
    )?;
    ```

## Armadilhas

- **Esqueceu `with_embeddings(true)`** — `embed` entra em pânico
  com "embedding mode is not enabled".
- **Tipo de pooling errado** — similaridade é `NaN` ou próxima de
  zero em todos os pares. BGE / GTE / E5 querem `Cls`;
  sentence-transformers quer `Mean`.
- **Comparando vetores não normalizados** — pontuações de
  similaridade parecem erradas. Passe `normalize = true` para
  `embed`.

## Código-fonte completo

[`examples/embeddings/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/embeddings/src/main.rs).

## Por onde ir a partir daqui

- [Busca semântica](embedding-search.md) — ranqueamento cosseno
  sobre um corpus pequeno.
- [Reranker](reranker.md) — demo de reranker bi-encoder.
- [Guia de embeddings & reranking](../features/embeddings.md) — a
  referência completa.
- [Receita de RAG](../recipes/rag.md) — embeddings em um pipeline
  de recuperação.
