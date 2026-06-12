# `chat` — Multi-turn chat

```rust,no_run
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::{Llama, LlamaParams, Role};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(4096))?;
    let history = vec![
        ChatMessage::new(Role::System, "You are a concise assistant."),
        ChatMessage::new(Role::User, "What is Rust?"),
    ];
    let resp = llama.create_chat_completion(&history, 128)?;
    println!("assistant> {}", resp.content);
    Ok(())
}
```

Run with:

```bash
cargo run --bin chat --release -- model.gguf
```

## Expected output

```
assistant> Rust is a systems programming language focused on safety, speed,
and concurrency. It achieves memory safety without a garbage collector
through its ownership and borrowing system.
```
