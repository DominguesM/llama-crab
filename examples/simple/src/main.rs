use anyhow::Result;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: simple <model.gguf> [prompt]"))?;
    let prompt = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "Once upon a time".to_string());

    let mut llama = Llama::load(
        LlamaParams::new(&model)
            .with_n_ctx(512)
            .with_n_gpu_layers(99),
    )?;
    let resp = llama.create_completion(&prompt, 64)?;
    println!("{}", resp.text);
    Ok(())
}
