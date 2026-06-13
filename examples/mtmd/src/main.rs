//! Minimal mtmd example that loads a text model, an mmproj projector and an
//! image, then generates a short answer.
//!
//! Usage:
//!   cargo run --release --bin mtmd -- <model.gguf> <mmproj.gguf> <image.png> [prompt]

use anyhow::Result;
use llama_crab::batch::LlamaBatch;
use llama_crab::multimodal::{default_media_marker, MtmdBitmap, MtmdContext, MtmdInputText};
use llama_crab::sampling::LlamaSampler;
use llama_crab::token::LlamaToken;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut args = std::env::args().skip(1);
    let model = args.next().ok_or_else(|| {
        anyhow::anyhow!("usage: mtmd <model.gguf> <mmproj.gguf> <image> [prompt]")
    })?;
    let mmproj = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing mmproj path"))?;
    let image = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing image path"))?;
    let prompt = args
        .next()
        .unwrap_or_else(|| "Describe this image in one short sentence.".to_string());

    let mut llama = Llama::load(LlamaParams::new(&model).with_n_ctx(4096))?;
    let mtmd = MtmdContext::init_from_file(&mmproj, llama.model())?;
    if !mtmd.support_vision() {
        anyhow::bail!("this projector does not support vision");
    }

    let bitmap = MtmdBitmap::from_file(&image)?;
    let marker = default_media_marker();
    let media_prompt = if model.to_ascii_lowercase().contains("lfm") {
        format!("<|im_start|>user\n{marker}\n{prompt}<|im_end|>\n<|im_start|>assistant\n")
    } else {
        format!("{marker}\n{prompt}")
    };
    let chunks = mtmd.tokenize(MtmdInputText::new(&media_prompt), &[&bitmap])?;

    let _ = llama.context().seq_rm(0, -1, -1);
    let ctx_ptr = llama.context().raw_handle();
    let new_n_past =
        unsafe { chunks.eval(&mtmd, ctx_ptr, 0, 0, llama.context().n_batch() as i32, true)? };

    let mut sampler = LlamaSampler::greedy().expect("greedy");
    let eos = llama.model().token_eos();
    let mut out = String::new();
    let mut n_generated = 0_usize;
    for _ in 0..96 {
        let idx = if n_generated == 0 { -1 } else { 0 };
        let tok: LlamaToken = unsafe { sampler.sample(ctx_ptr, idx) };
        sampler.accept(tok);
        if tok == eos {
            break;
        }
        out.push_str(&llama.model().detokenize(&[tok], false)?);

        let single = LlamaBatch::one(tok, new_n_past + n_generated as i32, 0, true);
        llama.context().decode(&single)?;
        n_generated += 1;
    }

    println!("assistant> {}", out.trim());
    Ok(())
}
