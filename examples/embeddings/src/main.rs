use anyhow::Result;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: embeddings <model.gguf> [text]"))?;
    let text = std::env::args().nth(2).unwrap_or_else(|| "Hello, world!".into());

    let llama = Llama::load(
        LlamaParams::new(&model)
            .with_n_ctx(512)
            .with_embeddings(true),
    )?;
    let tokens = llama.model().tokenize(&text, true, false)?;
    println!("tokens: {tokens:?}");
    Ok(())
}
