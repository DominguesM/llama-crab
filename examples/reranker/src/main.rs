use anyhow::Result;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: reranker <reranker.gguf>"))?;
    // Cross-encoder rerankers are loaded with `pooling_type = Rank`.
    let _ = Llama::load(LlamaParams::new(&model).with_n_ctx(512))?;
    println!("Reranker loaded (full ranking pipeline lands in v0.2).");
    Ok(())
}
