# `stateful_chat` — Interactive REPL

A multi-turn chat REPL that grows the conversation history on every
turn. Supports `/clear`, `/save`, and `EOF` to quit. The model is
loaded once and reused; only the message list grows.

## Run

```bash
./examples/run.sh stateful_chat
# or, manually:
./scripts/download_models.sh smol
cargo run --release --bin run_chat
```

Downloads `Qwen2.5-0.5B-Instruct-GGUF` (~400 MB).

## Commands

| Command  | Action                                |
| -------- | ------------------------------------- |
| `/exit`  | Quit (also `/quit`, `/q`, or Ctrl+D)  |
| `/clear` | Reset history (keeps the system msg)   |
| `/save`  | Print the conversation as JSON         |
| anything else | Sent as a user message           |

## What it does

```rust,no_run
use llama_crab::chat::BuiltinTemplate;
use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
use llama_crab::{Llama, LlamaParams, Role};
# let mut llama = Llama::load(LlamaParams::new("m.gguf").with_n_ctx(4096))?;

let mut history: Vec<ChatMessage> = vec![
    ChatMessage::new(Role::System,
        "You are a helpful, concise assistant. Always reply in English, in under 2 sentences."),
];

// On every user turn:
history.push(ChatMessage::new(Role::User, "What is Rust?".into()));
let resp = create_chat_completion_with(
    &mut llama, &history, BuiltinTemplate::ChatMl, &[], 128,
)?;
history.push(ChatMessage::new(Role::Assistant, resp.content));
# Ok::<(), Box<dyn std::error::Error>>(())
```

The history is the entire context — each call re-sends everything
and the prompt-cache (see [Caching](../caching.md)) skips the parts
that haven't changed.

## Expected output

```
🦀 llama-crab interactive chat
   model : models/qwen2.5-0.5b-instruct-q4_k_m.gguf
   commands: /exit  /clear  /save

> What is Rust?
  (0.81s)
assistant> Rust is a memory-safe systems programming language.

> /save
[
  { "role": "system", ... },
  { "role": "user", "content": "What is Rust?" },
  { "role": "assistant", "content": "Rust is a ..." }
]
```

## Full source

[`examples/stateful_chat/src/main.rs`][src].

[src]: https://github.com/DominguesM/llama-crab/tree/main/examples/stateful_chat/src/main.rs
