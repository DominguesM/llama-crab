use anyhow::Result;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: embeddings <model.gguf> [text]"))?;
    let text = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "Hello, world!".into());

    let mut llama = Llama::load(
        LlamaParams::new(&model)
            .with_n_ctx(512)
            .with_embeddings(true),
    )?;
    let tokens = llama.model().tokenize(&text, true, false)?;
    let embedding = llama.embed(&text, true)?;
    let norm = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
    let preview: Vec<f32> = embedding.iter().copied().take(8).collect();

    println!("text: {text}");
    println!("tokens: {tokens:?}");
    println!("embedding_dim: {}", embedding.len());
    println!("embedding_l2_norm: {norm:.6}");
    println!("embedding_preview: {preview:.6?}");
    Ok(())
}
