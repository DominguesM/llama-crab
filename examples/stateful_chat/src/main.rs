//! `chat` — interactive multi-turn chat that grows a conversation
//! history token by token.
//!
//! The model is loaded once; each user turn appends the previous
//! assistant reply to the history and asks for a new one. The history
//! is **not** truncated between turns (the context size is the only
//! limit) — this is the simplest possible persistent state.
//!
//! Run with:
//!
//! ```bash
//! ./examples/run.sh chat
//! ```
//!
//! or directly with `cargo` after downloading the model:
//!
//! ```bash
//! ./scripts/download_models.sh smol
//! cargo run --release --bin run_chat
//! ```
//!
//! Commands while running:
//!   * `/exit` — quit
//!   * `/clear` — reset the history
//!   * `/save` — print the conversation so far
//!   * anything else — user message

use anyhow::{Context, Result};
use llama_crab::chat::BuiltinTemplate;
use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
use llama_crab::{Llama, LlamaParams, Role};
use std::io::{self, BufRead, Write};
use std::time::Instant;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "models/qwen2.5-0.5b-instruct-q4_k_m.gguf".to_string());

    eprintln!("🦀 llama-crab interactive chat");
    eprintln!("   model : {model}");
    eprintln!("   commands: /exit  /clear  /save");
    eprintln!();

    let start = Instant::now();
    let mut llama = Llama::load(LlamaParams::new(&model).with_n_ctx(4096).with_n_threads(4))
        .with_context(|| format!("failed to load {model}"))?;
    eprintln!(
        "✓ model loaded in {:.2}s  ({} layers)",
        start.elapsed().as_secs_f64(),
        llama.model().n_layer()
    );

    let mut history: Vec<ChatMessage> = vec![ChatMessage::new(
        Role::System,
        "You are a helpful, concise assistant. Always reply in English, in under 2 sentences.",
    )];

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("\n> ");
        stdout.flush().ok();
        let mut line = String::new();
        let n = stdin.lock().read_line(&mut line)?;
        if n == 0 {
            eprintln!("(EOF)");
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        match input {
            "/exit" | "/quit" | "/q" => {
                eprintln!("bye.");
                break;
            }
            "/clear" => {
                history.truncate(1);
                eprintln!("(history cleared)");
                continue;
            }
            "/save" => {
                let json = serde_json::to_string_pretty(&history)?;
                eprintln!("{json}");
                continue;
            }
            _ => {}
        }

        history.push(ChatMessage::new(Role::User, input.to_string()));
        let t0 = Instant::now();
        match create_chat_completion_with(&mut llama, &history, BuiltinTemplate::ChatMl, &[], 96) {
            Ok(resp) => {
                let reply = resp.content.trim().to_string();
                let dt = t0.elapsed();
                eprintln!("  ({:.2}s)", dt.as_secs_f64());
                println!("assistant> {reply}");
                history.push(ChatMessage::new(Role::Assistant, reply));
            }
            Err(e) => {
                eprintln!("error: {e}");
                history.pop();
            }
        }
    }
    Ok(())
}
