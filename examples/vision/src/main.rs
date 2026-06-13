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
use llama_crab::multimodal::{default_media_marker, MtmdBitmap, MtmdContext, MtmdInputText};
use llama_crab::sampling::LlamaSampler;
use llama_crab::token::LlamaToken;
use llama_crab::{Llama, LlamaParams};
use std::time::Instant;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let mut args = std::env::args().skip(1);
    let model = args.next().ok_or_else(|| {
        anyhow::anyhow!("usage: vision <model.gguf> <mmproj.gguf> <image.png> [prompt]")
    })?;
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
    // Prepend the default media marker so `mtmd_tokenize` knows where
    // to insert the image tokens in the prompt.
    let marker = default_media_marker();
    let media_prompt = if model.to_ascii_lowercase().contains("lfm") {
        format!("<|im_start|>user\n{marker}\n{prompt}<|im_end|>\n<|im_start|>assistant\n")
    } else {
        format!("{marker}\n{prompt}")
    };
    let chunks = mtmd.tokenize(MtmdInputText::new(&media_prompt), &[&bitmap])?;
    eprintln!("Produced {} chunks", chunks.len());

    // Clear the KV cache for sequence 0 so the eval below starts at
    // position 0 (mirrors `create_completion`).
    let _ = llama.context().seq_rm(0, -1, -1);

    let ctx_ptr = llama.context().raw_handle();
    let n_batch = llama.context().n_batch() as i32;
    let new_n_past = unsafe { chunks.eval(&mtmd, ctx_ptr, 0, 0, n_batch, true)? };
    eprintln!("Consumed {new_n_past} positions");

    let start = Instant::now();
    let mut sampler = LlamaSampler::greedy().expect("greedy");
    let mut out = String::new();
    let eos = llama.model().token_eos();
    // Greedy sample, feed each token back so the next iteration samples
    // from a fresh forward pass. After multimodal eval, sample from the
    // last emitted logits with `-1`; thereafter the 1-token batch has logits
    // at index 0.
    let mut n_generated = 0_usize;
    for _ in 0..128 {
        let idx = if n_generated == 0 { -1 } else { 0 };
        let tok: LlamaToken = unsafe { sampler.sample(ctx_ptr, idx) };
        sampler.accept(tok);
        if tok == eos {
            break;
        }
        let piece = llama.model().detokenize(&[tok], false)?;
        out.push_str(&piece);
        // Feed the token back to keep the KV cache growing.
        let mut single = llama_crab::batch::LlamaBatch::new(1, 1);
        single.add(tok, new_n_past + n_generated as i32, &[0], true)?;
        llama.context().decode(&single)?;
        n_generated += 1;
    }
    eprintln!();
    println!("assistant> {}", out);
    eprintln!(
        "\n(generated in {} tokens, {:?})",
        n_generated,
        start.elapsed()
    );
    Ok(())
}
