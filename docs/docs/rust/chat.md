---
title: Chat
---

# Chat

Chat generation renders role-tagged messages into a model prompt, then runs text
completion.

## Minimal chat

```rust
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::{Llama, LlamaParams, Role};

let mut llama = Llama::load(LlamaParams::new("models/model.gguf").with_n_ctx(2048))?;
let messages = vec![
    ChatMessage::new(Role::System, "You are a concise assistant."),
    ChatMessage::new(Role::User, "What is Rust in one sentence?"),
];

let assistant = llama.create_chat_completion(&messages, 64)?;
println!("{}", assistant.content);
```

## Pick a template

The plain helper uses the default plain template. Most instruct GGUFs expect a
specific chat template, so use `create_chat_completion_with` when needed:

```rust
use llama_crab::chat::BuiltinTemplate;
use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
use llama_crab::{Llama, Role};

let messages = vec![
    ChatMessage::new(Role::System, "Always answer in English. Be concise."),
    ChatMessage::new(Role::User, "Introduce yourself."),
];

let response = create_chat_completion_with(
    &mut llama,
    &messages,
    BuiltinTemplate::ChatMl,
    &[],
    128,
)?;
```

The `chat` and `stateful_chat` examples both use `BuiltinTemplate::ChatMl`.

## Conversation history

`llama-crab` does not hide your application state. Keep the conversation as a
`Vec<ChatMessage>`, append each user message, generate, then append the assistant
reply:

```rust
history.push(ChatMessage::new(Role::User, user_input));
let response = create_chat_completion_with(&mut llama, &history, BuiltinTemplate::ChatMl, &[], 96)?;
history.push(ChatMessage::new(Role::Assistant, response.content));
```

`examples/stateful_chat/src/main.rs` adds `/clear` and `/save` commands around
that pattern.

## Streaming chat

For token-by-token output, call `create_chat_completion_stream` or
`create_chat_completion_stream_with`. The callback receives completion chunks;
the returned `ChatMessage` contains the final assistant content.

## Tools

The lower-level chat module has tool definition and parser support. The current
`examples/tools` binary demonstrates the practical pattern: describe the tool in
the system prompt, ask the model for a JSON call, parse the JSON, then execute
the function in Rust.
