# `chat` — Multi-turn chat

The simplest chat program: a system message, a single user turn,
print the assistant response. Use it as a starting point for any
chat-based tool or as a template for an automated test.

## Run

```bash
cargo run --bin chat --release -- model.gguf
```

The first positional argument is the path to an instruct-tuned
GGUF.

## What it does

```rust
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

## Expected output

```
assistant> Rust is a systems programming language focused on safety, speed,
and concurrency. It achieves memory safety without a garbage collector
through its ownership and borrowing system.
```

The actual text depends on the model.

## Picking a template

The high-level `create_chat_completion` picks a default template.
For explicit control, use `create_chat_completion_with`:

```rust
use llama_crab::chat::BuiltinTemplate;
use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
use llama_crab::{Llama, LlamaParams, Role};

let mut llama = Llama::load(LlamaParams::new("model.gguf"))?;
let history = vec![
    ChatMessage::new(Role::System, "You are a helpful assistant."),
    ChatMessage::new(Role::User, "Hi!"),
];

let resp = create_chat_completion_with(
    &mut llama,
    &history,
    BuiltinTemplate::Llama3,   // explicit template
    &[],                        // no tools
    128,                        // max tokens
)?;
```

Or auto-detect from the GGUF metadata:

```rust
let template = detect_chat_format(&llama.model().metadata());
let resp = create_chat_completion_with(
    &mut llama, &history, template, &[], 128,
)?;
```

See the [chat & tool calling guide](../features/chat.md) for the
full template list and the auto-detection rules.

## Two-turn conversation

To send a follow-up user turn, push the assistant response into the
history and call again:

```rust
let mut history = vec![
    ChatMessage::new(Role::System, "You are a concise assistant."),
    ChatMessage::new(Role::User, "What is Rust?"),
];

let resp = llama.create_chat_completion(&history, 128)?;
println!("assistant> {}", resp.content);

history.push(ChatMessage::new(Role::Assistant, resp.content));
history.push(ChatMessage::new(Role::User, "Show me hello-world."));

let resp = llama.create_chat_completion(&history, 128)?;
println!("assistant> {}", resp.content);
```

See the [stateful chat example](stateful-chat.md) for the full
REPL pattern.

## Adding tools

Pass a `Vec<ToolDefinition>` to `create_chat_completion_with`:

```rust
use llama_crab::chat::ToolDefinition;
use llama_crab::chat::BuiltinTemplate;
use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
use llama_crab::{Llama, LlamaParams, Role};
use serde_json::json;

let tool = ToolDefinition::new("get_weather", "Get the weather for a city")
    .with_parameters(json!({
        "type": "object",
        "properties": { "city": { "type": "string" } },
        "required": ["city"]
    }));

let history = vec![ChatMessage::new(Role::User, "Weather in Tokyo?")];
let resp = create_chat_completion_with(
    &mut llama,
    &history,
    BuiltinTemplate::ChatMl,
    &[tool],
    96,
)?;
```

Then parse the response with a `ToolParser`. See the
[tool calling example](tools.md) for the full loop.

## Full source

[`examples/chat/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/chat/src/main.rs).

## Where to next?

- [Stateful chat](stateful-chat.md) — interactive REPL.
- [Tool calling](tools.md) — function-calling loop.
- [Chat & tool calling guide](../features/chat.md) — templates,
  parsers, multi-turn tool protocol.
