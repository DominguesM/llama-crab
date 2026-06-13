# Stateful chat

A stateful chat keeps the conversation history alive across turns so
the model sees the full context every time. At the API level this is
just a growing `Vec<ChatMessage>` that you re-send on each turn —
`llama-crab` reuses the prompt prefix cache to avoid recomputing
previous turns from scratch.

## Minimal loop

```rust,no_run
use llama_crab::chat::{ChatMessage, BuiltinTemplate};
use llama_crab::high_level::chat_completion::create_chat_completion_with;
use llama_crab::{Llama, LlamaParams, Role};
# let mut llama = Llama::load(LlamaParams::new("m.gguf").with_n_ctx(4096))?;

let mut history: Vec<ChatMessage> = vec![
    ChatMessage::new(Role::System, "You are a concise assistant."),
];

// First user turn.
history.push(ChatMessage::new(Role::User, "What is Rust?".into()));
let resp = create_chat_completion_with(
    &mut llama, &history, BuiltinTemplate::ChatMl, &[], 128,
)?;
history.push(ChatMessage::new(Role::Assistant, resp.content));

// Second user turn — the model sees the previous exchange.
history.push(ChatMessage::new(Role::User, "Show me hello-world.".into()));
let resp = create_chat_completion_with(
    &mut llama, &history, BuiltinTemplate::ChatMl, &[], 128,
)?;
# let _ = resp;
# Ok::<(), Box<dyn std::error::Error>>(())
```

For a full interactive REPL (with `/clear`, `/save`, EOF handling),
see [`stateful_chat`](./examples/stateful_chat.md).

## Picking a template

The model's GGUF metadata usually declares its chat template. Use
[`detect_chat_format`] on the metadata to read it, or force a known
one with `BuiltinTemplate::ChatMl` / `Llama3` / `Qwen2` / …

```rust,no_run
# use llama_crab::chat::detect_chat_format;
# use std::collections::BTreeMap;
let mut md = BTreeMap::new();
md.insert("general.architecture".into(), "llama3".into());
let template = detect_chat_format(&md);
# let _ = template;
```

## Trimming history

`LlamaParams::with_n_ctx(N)` caps the number of tokens the context
can hold. When the history grows past it, you have three options:

1. **Truncate head** — drop the oldest user/assistant turns, keep the
   system message. Simplest and what most chat UIs do.
2. **Summarize** — periodically replace the oldest turns with a single
   `Role::System` summary.
3. **Bigger `n_ctx`** — pay more memory and per-step latency.

## Session persistence

To resume a conversation after the process exits, serialize the
history with `serde_json` and reload it on the next start. KV cache
entries ([Caching & session state](./caching.md)) make the resume
nearly free if the prompt is byte-identical.

```rust,no_run
# use llama_crab::chat::ChatMessage;
# let history: Vec<ChatMessage> = Vec::new();
let json = serde_json::to_string_pretty(&history)?;
std::fs::write("conversation.json", json)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Where to next?

- [`stateful_chat` example](./examples/stateful_chat.md)
- [Chat & tool calling](./chat.md)
- [Caching & session state](./caching.md)

[`detect_chat_format`]: https://docs.rs/llama-crab/latest/llama_crab/chat/fn.detect_chat_format.html
