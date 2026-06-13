use anyhow::{anyhow, Result};
use llama_crab::speculative::{DraftModel, PromptLookupDecoding};
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("usage: speculative <model.gguf>"))?;

    let llama = Llama::load(LlamaParams::new(&model).with_n_ctx(1024))?;
    let prompt = "Rust is fast and memory safe. Rust is fast";
    let prompt_tokens = llama.model().tokenize(prompt, true, true)?;

    let draft = PromptLookupDecoding::new(3, 8);
    let drafted = draft.draft(&prompt_tokens, 8);
    if drafted.is_empty() {
        return Err(anyhow!("prompt lookup produced no draft tokens"));
    }

    let drafted_text = llama.model().detokenize(&drafted, false)?;
    println!("prompt> {prompt}");
    println!("drafted token ids> {drafted:?}");
    println!("drafted text> {}", drafted_text.trim());
    Ok(())
}
