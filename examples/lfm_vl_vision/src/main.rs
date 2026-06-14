//! `lfm_vl_vision` — interactive multi-question REPL using the **Liquid AI
//! LFM2.5-VL 1.6B** vision-language model. Smaller than Gemma 4 (~1 GB vs
//! ~5 GB), so it runs comfortably on CPU and on older Apple Silicon.
//!
//! Ask a question, get an answer, ask another — the chat history is
//! kept on the model side by re-rendering the full conversation into
//! the prompt each turn (the same approach used by `stateful_chat`).
//!
//! Usage:
//!
//! ```bash
//! ./examples/run.sh lfm_vl
//! ```
//!
//! or, after `./scripts/download_models.sh lfm-vl`:
//!
//! ```bash
//! cargo run --release --bin run_lfm_vl
//! ```
//!
//! Commands while running:
//!   * `/image <path>` — change the active image
//!   * `/clear`        — reset the chat history
//!   * `/exit`         — quit
//!
//! The default model + image paths match what `./scripts/download_models.sh
//! lfm-vl` produces.

use anyhow::{Context, Result};
use llama_crab::batch::LlamaBatch;
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::multimodal::{default_media_marker, MtmdBitmap, MtmdContext, MtmdInputText};
use llama_crab::sampling::LlamaSampler;
use llama_crab::token::LlamaToken;
use llama_crab::{Llama, LlamaParams, Role};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

const DEFAULT_MODEL: &str = "models/LFM2.5-VL-1.6B-Q4_K_M.gguf";
const DEFAULT_MMPROJ: &str = "models/LFM2.5-VL-1.6B-mmproj-BF16.gguf";
const DEFAULT_IMAGE: &str = "tests/fixtures/test_image.png";

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // ---- CLI args (all optional) ---------------------------------------
    let mut args = std::env::args().skip(1);
    let model = args.next().unwrap_or_else(|| DEFAULT_MODEL.to_string());
    let mmproj = args.next().unwrap_or_else(|| DEFAULT_MMPROJ.to_string());
    let mut image = PathBuf::from(args.next().unwrap_or_else(|| DEFAULT_IMAGE.to_string()));
    if let Some(p) = args.next() {
        // 4th positional = first prompt, no REPL
        return run_single_turn(&model, &mmproj, &image, &p);
    }

    eprintln!("🦀 llama-crab lfm_vl REPL");
    eprintln!("   model  : {model}");
    eprintln!("   mmproj : {mmproj}");
    eprintln!("   image  : {}", image.display());
    eprintln!("   commands: /image <path>  /clear  /exit");
    eprintln!();

    // ---- model + projector ---------------------------------------------
    let start = Instant::now();
    let mut llama = Llama::load(LlamaParams::new(&model).with_n_ctx(4096).with_n_threads(4))
        .with_context(|| format!("failed to load {model}"))?;
    eprintln!(
        "✓ model loaded in {:.2}s  ({} layers)",
        start.elapsed().as_secs_f64(),
        llama.model().n_layer()
    );
    let mtmd = MtmdContext::init_from_file(&mmproj, llama.model())
        .with_context(|| format!("failed to init {mmproj}"))?;
    if !mtmd.support_vision() {
        anyhow::bail!("this projector does not support vision");
    }

    // ---- conversation history ------------------------------------------
    let mut history: Vec<ChatMessage> = vec![ChatMessage::new(
        Role::System,
        "You are a concise vision assistant. Look carefully at the image \
         and answer in 1-2 sentences, in English.",
    )];

    // ---- REPL loop -----------------------------------------------------
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("\n[image: {}] > ", image.display());
        stdout.flush().ok();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            eprintln!("(EOF)");
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if let Some(rest) = input.strip_prefix("/image ") {
            image = PathBuf::from(rest.trim());
            eprintln!("(image set to {})", image.display());
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
            _ => {}
        }

        history.push(ChatMessage::new(Role::User, input.to_string()));
        match ask(&mut llama, &mtmd, &image, &history) {
            Ok(reply) => {
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

/// One-shot helper used when the user passes 4 positional arguments.
fn run_single_turn(model: &str, mmproj: &str, image: &Path, prompt: &str) -> Result<()> {
    let mut llama = Llama::load(LlamaParams::new(model).with_n_ctx(4096))?;
    let mtmd = MtmdContext::init_from_file(mmproj, llama.model())?;
    let history = vec![
        ChatMessage::new(
            Role::System,
            "You are a concise vision assistant. Reply in 1-2 sentences.",
        ),
        ChatMessage::new(Role::User, prompt.to_string()),
    ];
    let reply = ask(&mut llama, &mtmd, image, &history)?;
    println!("assistant> {reply}");
    Ok(())
}

/// Build the prompt from the chat history, prepend the image and run
/// the decode + sample loop. Returns the assistant's reply as a
/// trimmed `String`.
fn ask(
    llama: &mut Llama,
    mtmd: &MtmdContext,
    image: &Path,
    history: &[ChatMessage],
) -> Result<String> {
    use llama_crab::chat::render_builtin;
    use llama_crab::chat::BuiltinTemplate;

    // Render the conversation using LFM's ChatML-style template, with
    // the media marker inserted into the latest user turn.
    let marker = default_media_marker();
    let mut prompt_history = history.to_vec();
    if let Some(last_user) = prompt_history.iter_mut().rfind(|m| m.role == Role::User) {
        last_user.content = format!("{marker}\n{}", last_user.content);
    }
    let media_prompt = render_builtin(BuiltinTemplate::ChatMl, &prompt_history, &[], true);

    // Decode the image.
    let bitmap = MtmdBitmap::from_file(image)
        .with_context(|| format!("failed to load image {}", image.display()))?;
    let chunks = mtmd.tokenize(MtmdInputText::new(&media_prompt), &[&bitmap])?;

    // Clear seq 0 so eval starts at position 0.
    let _ = llama.context().seq_rm(0, -1, -1);

    let ctx_ptr = llama.context().raw_handle();
    let n_batch = llama.context().n_batch() as i32;
    let new_n_past = unsafe { chunks.eval(mtmd, ctx_ptr, 0, 0, n_batch, true) }
        .map_err(|e| anyhow::anyhow!("chunks.eval: {e}"))?;

    // Greedy decode. After multimodal eval, sample from the last emitted
    // logits with `-1`; after that every 1-token batch has logits at index 0.
    let mut sampler = LlamaSampler::greedy().expect("greedy");
    let eos = llama.model().token_eos();
    let mut out = String::new();
    for n_generated in 0_usize..256 {
        let idx = if n_generated == 0 { -1 } else { 0 };
        let tok: LlamaToken = unsafe { sampler.sample(ctx_ptr, idx) };
        sampler.accept(tok);
        if tok == eos {
            break;
        }
        let piece = llama.model().detokenize(&[tok], false)?;
        out.push_str(&piece);
        let mut single = LlamaBatch::new(1, 1);
        single.add(tok, new_n_past + n_generated as i32, &[0], true)?;
        llama.context().decode(&single)?;
    }
    Ok(out.trim().to_string())
}
