use anyhow::Result;
use llama_crab::speculative::{DraftModel, PromptLookupDecoding};

fn main() -> Result<()> {
    let prompt: Vec<llama_crab::LlamaToken> = (0..32).map(llama_crab::LlamaToken).collect();
    let draft = PromptLookupDecoding::default();
    let out = draft.draft(&prompt, 8);
    println!("drafted tokens: {out:?}");
    Ok(())
}
