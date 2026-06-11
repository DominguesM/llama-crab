//! End-to-end vision example: load Gemma 4 or LFM2.5-VL, attach an image,
//! and ask a question about it.
//!
//! Usage:
//!   cargo run --features mtmd --bin vision --release -- \
//!     <model.gguf> <mmproj.gguf> <image.png> [prompt]
//!
//! Example:
//!   cargo run --features mtmd --bin vision --release -- \
//!     models/gemma-4-E4B-it-Q4_K_M.gguf \
//!     models/gemma-4-E4B-it-mmproj.gguf \
//!     tests/fixtures/test_image.png

use anyhow::Result;
use llama_crab::multimodal::{MtmdBitmap, MtmdContext, MtmdInputText};
use llama_crab::sampling::LlamaSampler;
use llama_crab::token::LlamaToken;
use llama_crab::{Llama, LlamaParams};
use std::time::Instant;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut args = std::env::args().skip(1);
    let model = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("usage: vision <model.gguf> <mmproj.gguf> <image.png> [prompt]"))?;
    let mmproj = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing mmproj path"))?;
    let image = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing image path"))?;
    let prompt = args
        .next()
        .unwrap_or_else(|| "Describe this image in one sentence.".to_string());

    eprintln!("Loading {model}...");
    let mut llama = Llama::load(LlamaParams::new(&model).with_n_ctx(4096))?;
    eprintln!(
        "Loaded: {} layers, {} ctx, {} embd",
        llama.model().n_layer(),
        llama.model().n_ctx_train(),
        llama.model().n_embd()
    );

    eprintln!("Initializing mmproj from {mmproj}...");
    let mtmd = MtmdContext::init_from_file(&mmproj, llama.model())?;
    if !mtmd.support_vision() {
        anyhow::bail!("this projector does not support vision");
    }

    eprintln!("Decoding {image}...");
    let bitmap = MtmdBitmap::from_file(&image)?;
    eprintln!("Image: {}x{} px", bitmap.nx(), bitmap.ny());

    eprintln!("Tokenizing prompt + image...");
    let chunks = mtmd.tokenize(MtmdInputText::new(&prompt), &[&bitmap])?;
    eprintln!("Produced {} chunks", chunks.len());

    let ctx_ptr = llama.context().raw_handle();
    let n_batch = llama.context().n_batch() as i32;
    let new_n_past = unsafe { chunks.eval(&mtmd, ctx_ptr, 0, 0, n_batch, true)? };
    eprintln!("Consumed {new_n_past} positions");

    let start = Instant::now();
    let mut sampler = LlamaSampler::greedy().expect("greedy");
    let mut out = String::new();
    let eos = llama.model().token_eos();
    for _ in 0..128 {
        let tok: LlamaToken = unsafe { sampler.sample(ctx_ptr, new_n_past - 1) };
        sampler.accept(tok);
        if tok == eos {
            break;
        }
        if let Ok(piece) = llama.model().detokenize(&[tok], false) {
            out.push_str(&piece);
        }
    }
    eprintln!();
    println!("assistant> {}", out);
    eprintln!("\n(generated in {:?})", start.elapsed());
    Ok(())
}
