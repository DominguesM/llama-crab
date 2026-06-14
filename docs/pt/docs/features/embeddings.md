# Embeddings & reranking

`llama-crab` expõe o pipeline de embeddings do `llama.cpp` através
de um único helper de alto nível, [`Llama::embed`], mais knobs de
pooling e normalização em `LlamaContextParams`. Esta página percorre
a habilitação de embeddings, os quatro modos de pooling, busca
semântica e o helper cross-encoder `Llama::rerank`.

## Habilitando embeddings

Carregue o modelo com `with_embeddings(true)`. Por padrão o
contexto usa **mean pooling**; escolha uma estratégia diferente
com `with_pooling_type`:

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
| `None`  | Embeddings em nível de token. Sem pooling — a saída é uma matriz. |
| `Mean`  | **Padrão.** Robusto para similaridade geral de sentenças. |
| `Cls`   | BGE / GTE / E5 — usa o primeiro token (BOS) como o resumo. |
| `Last`  | Usa o último token não-pad. |
| `Rank`  | Rerankers cross-encoder. Produz um único logit por par. |

## Computando um embedding

```rust
let v: Vec<f32> = llama.embed("Rust é memory-safe.", true)?;
//                                       normalize = true ^^^^
```

`embed(..., true)` retorna um vetor **L2-normalizado**, então o
produto escalar de dois vetores é igual à similaridade cosseno. A
função retorna `Result<Vec<f32>, LlamaError>`.

### Opções de embedding

O helper `embed` aceita configuração opcional através de
`EmbedOptions`:

```rust
use llama_crab::embed::EmbedOptions;

let v = llama.embed_with_options(
    "Rust é memory-safe.",
    EmbedOptions::new()
        .with_normalize(true)
        .with_start_token(false)   // pula o token BOS
)?;
```

## Embeddings em batch

Para cargas de trabalho multi-documento, prefira `embed_texts` (ou
`embed_texts_with_options`):

```rust
use llama_crab::Llama;

let texts = vec![
    "Rust é memory-safe.",
    "Python é uma linguagem dinâmica.",
    "A Torre Eiffel fica em Paris.",
];
let embeddings = llama.embed_texts(&texts, true)?;   // Vec<Vec<f32>>
```

A chamada em batch amortiza o custo de carga do modelo, mas cada
texto ainda é avaliado independentemente. Use as APIs de batch e
sequência de baixo nível quando precisar de maior throughput.

## Busca semântica

Incorpore uma query e um corpus, depois ranqueie por similaridade
cosseno:

```rust
let corpus = [
    "Rust é memory-safe.",
    "Paris é a capital da França.",
    "Bananas são frutas amarelas.",
];
let query = "linguagem de programação segura";

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

O exemplo completo vive em
[`examples/embedding_search/`](../examples/embedding-search.md).

## Reranking

Rerankers (também conhecidos como cross-encoders) pontuam pares
`(query, document)` **conjuntamente** em vez de a partir de
embeddings independentes. Eles dão ranqueamentos melhores ao
custo de uma passada de modelo por par.

`llama-crab` inclui `Llama::rerank(query, documents)` para
modelos cross-encoder de rank. Carregue o modelo com embeddings
habilitados e `PoolingType::Rank`, depois passe a query e os
documentos:

```rust
use llama_crab::context::params::PoolingType;
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(
    LlamaParams::new("bge-reranker-base-q4_k_m.gguf")
        .with_n_ctx(512)
        .with_embeddings(true)
        .with_pooling_type(PoolingType::Rank),
)?;

let scores = llama.rerank("sistemas de programação seguros", &[
    "Rust previne muitos bugs de memória.",
    "Paris é a capital da França.",
])?;
```

O helper atualmente codifica cada par `(query, document)`
independentemente. Use as APIs de batch e sequência de baixo nível
quando precisar de maior throughput.

### Bi-encoder vs cross-encoder

| Método | Latência | Qualidade | Quando usar |
| --- | --- | --- | --- |
| **Bi-encoder** (cosseno em embeddings independentes) | Barato — codifique cada texto uma vez, depois produto escalar. | Bom para recuperação "está na faixa". | Recuperação de primeiro estágio sobre milhares de documentos. |
| **Cross-encoder** (rerank em pares `(query, doc)`) | Caro — uma passada de modelo por par. | Muito melhor em relevância fina. | Reranking de segundo estágio sobre o top K de candidatos. |

Um pipeline típico usa ambos: um bi-encoder rápido recupera 100
candidatos, depois um cross-encoder os reranqueia.

## Construindo um índice de vetores

`llama-crab` é agnóstico em relação à camada de armazenamento. O
"índice" mais simples é um `Vec<(String, Vec<f32>)>` de pares
`(texto, embedding)` mantido em memória. Para corpora maiores,
combine os embeddings com um de:

- [`hnsw`](https://crates.io/crates/hnsw) — HNSW nativo em Rust.
- [`qdrant-client`](https://crates.io/crates/qdrant-client) — DB
  de vetores Qdrant.
- [`pgvector`](https://github.com/pgvector/pgvector) — Postgres
  com suporte a vetores.

A invariante importante é que o índice **armazena vetores
L2-normalizados** e a query também é normalizada — então o
produto escalar é igual à similaridade cosseno e você pode usar
um único tipo de índice para ambos.

## Armadilhas comuns

| Armadilha | Sintoma | Correção |
| --- | --- | --- |
| Tipo de pooling errado | Similaridade é `NaN` ou próxima de zero em todos os pares. | BGE / GTE / E5 esperam `Cls`; modelos estilo sentence-transformers preferem `Mean`. |
| Esqueceu `with_embeddings(true)` | `embed` entra em pânico com "embedding mode is not enabled". | Adicione `.with_embeddings(true)` aos params. |
| Comparando vetores não normalizados | Pontuações de similaridade parecem erradas. | Passe `normalize = true` para `embed`. |
| Cross-encoder carregado com pooling `Mean` | `rerank` retorna pontuações garbage. | Use `PoolingType::Rank` para cross-encoders. |
| Modelo de embedding é pequeno demais para o idioma | Pontuações de similaridade parecem ruído. | Escolha um modelo treinado no idioma que você quer incorporar. |

## Por onde ir a partir daqui

- [Exemplo de embeddings](../examples/embeddings.md) — um programa
  de 30 linhas que imprime um embedding.
- [Exemplo de busca semântica](../examples/embedding-search.md) —
  ranqueamento cosseno sobre um corpus pequeno.
- [Exemplo de reranker](../examples/reranker.md) — uma demo de
  reranker bi-encoder.
- [Receita de RAG](../recipes/rag.md) — combinando embeddings, um
  vector store e um modelo de chat em um único pipeline.

[`Llama::embed`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html#method.embed
