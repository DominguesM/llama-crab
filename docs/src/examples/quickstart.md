# `quickstart` — First end-to-end program

The smallest program that exercises every common step: load a model,
tokenize, plain completion, chat completion, and a FIM code-infill.

## Run

```bash
./examples/run.sh quickstart
# or, manually:
./scripts/download_models.sh smol
cargo run --release --bin run_quickstart
```

Downloads `Qwen2.5-0.5B-Instruct-GGUF` (~400 MB) the first time.

## What it does

```rust,no_run
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::{Llama, LlamaParams, Role};

let mut llama = Llama::load(
    LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
        .with_n_ctx(2048)
        .with_n_threads(4),
)?;

// 1. Plain text completion.
let resp = llama.create_completion("The capital of France is", 16)?;

// 2. Chat completion with a system + user turn.
let history = vec![
    ChatMessage::new(Role::System, "You are a concise assistant."),
    ChatMessage::new(Role::User, "What is Rust in one sentence?"),
];
let resp = llama.create_chat_completion(&history, 64)?;

// 3. FIM code infill.
let fill = llama.complete_infill("fn main() {", "}")?;
# let _ = resp; let _ = fill;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Expected output

```
🦀 llama-crab quickstart
   model : models/qwen2.5-0.5b-instruct-q4_k_m.gguf

✓ model loaded in 1.20s  (28 layers, vocab=151665)

▶ create_completion("The capital of France is", 16)
   → 16 tokens in 0.40s
 Paris is the capital of France.

▶ create_chat_completion(What is Rust?)
   → assistant in 0.80s
assistant> Rust is a memory-safe systems programming language.

▶ complete_infill("fn main() {", "}")
    println!("hello");
```

(actual text varies; timings depend on hardware)

## Full source

[`examples/quickstart/src/main.rs`][src] — 80 lines, annotated.

[src]: https://github.com/DominguesM/llama-crab/tree/main/examples/quickstart/src/main.rs
