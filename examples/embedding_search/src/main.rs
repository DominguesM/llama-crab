//! `embeddings` — compute sentence embeddings with a BGE model.
//!
//! Run with:
//!
//! ```bash
//! ./examples/run.sh embeddings
//! ```
//!
//! or after downloading the BGE-small model directly:
//!
//! ```bash
//! ./scripts/download_models.sh bge
//! cargo run --release --bin run_embeddings
//! ```
//!
//! The first positional argument selects the GGUF path (default
//! `models/bge-small-en-v1.5-q4_k_m.gguf`). The second positional
//! argument (or `LLAMA_CRAB_QUERY` env var) selects the query.

use anyhow::{Context, Result};
use llama_crab::context::params::PoolingType;
use llama_crab::sampling::LlamaSampler;
use llama_crab::{Llama, LlamaParams};
use std::time::Instant;

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "models/bge-small-en-v1.5-q4_k_m.gguf".to_string());
    let query = std::env::args()
        .nth(2)
        .or_else(|| std::env::var("LLAMA_CRAB_QUERY").ok())
        .unwrap_or_else(|| "What programming language is safest?".to_string());

    eprintln!("🦀 llama-crab embeddings example");
    eprintln!("   model : {model}");
    eprintln!("   query : {query}");
    eprintln!();

    let start = Instant::now();
    let mut llama = Llama::load(
        LlamaParams::new(&model)
            .with_n_ctx(512)
            .with_embeddings(true)
            .with_pooling_type(PoolingType::Cls),
    )
    .with_context(|| format!("failed to load {model}"))?;
    eprintln!("✓ model loaded in {:.2}s", start.elapsed().as_secs_f64());

    // Tiny fixed corpus.
    let corpus: &[(&str, &str)] = &[
        (
            "doc-1",
            "Rust is a memory-safe systems language without a garbage collector.",
        ),
        (
            "doc-2",
            "Python is a high-level dynamic language with duck typing.",
        ),
        (
            "doc-3",
            "The Eiffel Tower is one of the most visited monuments in the world.",
        ),
        (
            "doc-4",
            "Borrow checking enforces lifetimes at compile time in Rust.",
        ),
    ];

    eprintln!("\n▶ embedding {} corpus items + 1 query", corpus.len() + 1);
    let q_vec = llama.embed(&query, true)?;
    let mut scored: Vec<(&str, f32)> = Vec::with_capacity(corpus.len());
    for (id, text) in corpus {
        let v = llama.embed(text, true)?;
        scored.push((id, cosine(&q_vec, &v)));
    }
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    eprintln!("\n📊 results (cosine similarity, higher = more similar):");
    for (id, score) in &scored {
        let text = corpus.iter().find(|(i, _)| *i == *id).unwrap().1;
        eprintln!("   {score:>5.3}  {id:<6}  {text}");
    }

    println!("\nQuery: {query}");
    println!("Top match: {} (cosine = {:.3})", scored[0].0, scored[0].1);
    let top = corpus.iter().find(|(i, _)| *i == scored[0].0).unwrap().1;
    println!("  > {top}");
    // Keep the import live for `cargo check`.
    let _ = std::mem::size_of::<LlamaSampler>();
    Ok(())
}
