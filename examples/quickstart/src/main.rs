//! `quickstart` — the simplest possible end-to-end example.
//!
//! Loads a small GGUF, tokenizes a prompt, runs a single completion
//! and prints the result. Designed to be the first program a new
//! user runs. Run with the convenience script:
//!
//! ```bash
//! ./examples/run.sh quickstart
//! ```
//!
//! or directly with `cargo` after downloading the model:
//!
//! ```bash
//! ./scripts/download_models.sh smol
//! cargo run --release --bin run_quickstart
//! ```

use anyhow::{Context, Result};
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::{Llama, LlamaParams, Role};
use std::io::{self, Write};
use std::time::Instant;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "models/qwen2.5-0.5b-instruct-q4_k_m.gguf".to_string());

    eprintln!("🦀 llama-crab quickstart");
    eprintln!("   model : {model}");
    eprintln!();

    let start = Instant::now();
    let mut llama = Llama::load(LlamaParams::new(&model).with_n_ctx(2048).with_n_threads(4))
        .with_context(|| format!("failed to load {model}"))?;
    eprintln!(
        "✓ model loaded in {:.2}s  ({} layers, vocab={})",
        start.elapsed().as_secs_f64(),
        llama.model().n_layer(),
        llama.model().n_vocab()
    );

    // Step 1 — tokenize a small prompt and print the ids.
    let prompt = "The capital of France is";
    let tokens = llama.model().tokenize(prompt, true, false)?;
    eprintln!("\n📝 tokenize({prompt:?}) → {} tokens", tokens.len());
    for (i, t) in tokens.iter().enumerate() {
        let piece = llama.model().detokenize(&[*t], false).unwrap_or_default();
        eprintln!("   [{i:>2}] {t} = {piece:?}");
    }
    io::stderr().flush().ok();

    // Step 2 — single-shot text completion. Uses the default greedy
    // sampler under the hood.
    eprintln!("\n▶ create_completion({prompt:?}, 16)");
    let t0 = Instant::now();
    let resp = llama.create_completion(prompt, 16)?;
    eprintln!(
        "   → {} tokens in {:.2}s",
        resp.n_tokens,
        t0.elapsed().as_secs_f64()
    );
    println!("{}", resp.text);

    // Step 3 — one round of chat completion.
    eprintln!("▶ create_chat_completion(What is Rust?)");
    let history = vec![
        ChatMessage::new(Role::System, "You are a concise assistant."),
        ChatMessage::new(Role::User, "What is Rust in one sentence?"),
    ];
    let t0 = Instant::now();
    let resp = llama.create_chat_completion(&history, 64)?;
    eprintln!("   → assistant in {:.2}s", t0.elapsed().as_secs_f64());
    println!("assistant> {}", resp.content);

    // Step 4 — one round of FIM code infill.
    eprintln!("▶ complete_infill(\"fn main() {{\", \"}}\")");
    match llama.complete_infill("fn main() {", "}") {
        Ok(fill) => println!("{fill}"),
        Err(e) => eprintln!("   (skipped: {e})"),
    }

    eprintln!("\n✓ done in {:.2}s", start.elapsed().as_secs_f64());
    Ok(())
}
