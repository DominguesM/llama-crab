use anyhow::{anyhow, Result};
use llama_crab::{Llama, LlamaParams};
use std::cmp::Ordering;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("usage: reranker <embedding-model.gguf>"))?;

    let query = "safe systems programming language";
    let documents = [
        "Rust is a memory-safe systems programming language.",
        "Paris is the capital city of France.",
        "Bananas are yellow fruit rich in potassium.",
    ];

    let mut llama = Llama::load(
        LlamaParams::new(&model)
            .with_n_ctx(512)
            .with_embeddings(true),
    )?;
    let query_embedding = llama.embed(query, true)?;
    let mut scored = Vec::with_capacity(documents.len());
    for document in documents {
        let document_embedding = llama.embed(document, true)?;
        scored.push((cosine(&query_embedding, &document_embedding), document));
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
    println!("query: {query}");
    for (rank, (score, document)) in scored.iter().enumerate() {
        println!("{:>2}. score={score:.4} document={document}", rank + 1);
    }
    Ok(())
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
