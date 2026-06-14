# `chat` — Chat multi-turno

O programa de chat mais simples: uma mensagem de sistema, um único
turno do usuário, imprime a resposta do assistant. Use como ponto
de partida para qualquer ferramenta baseada em chat ou como
template para um teste automatizado.

## Execute

```bash
cargo run --bin chat --release -- modelo.gguf
```

O primeiro argumento posicional é o caminho para um GGUF
tuned-instruct.

## O que ele faz

```rust
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::{Llama, LlamaParams, Role};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(LlamaParams::new("modelo.gguf").with_n_ctx(4096))?;
    let history = vec![
        ChatMessage::new(Role::System, "You are a concise assistant."),
        ChatMessage::new(Role::User, "What is Rust?"),
    ];
    let resp = llama.create_chat_completion(&history, 128)?;
    println!("assistant> {}", resp.content);
    Ok(())
}
```

## Saída esperada

```
assistant> Rust is a systems programming language focused on safety, speed,
and concurrency. It achieves memory safety without a garbage collector
through its ownership and borrowing system.
```

O texto real depende do modelo.

## Escolhendo um template

O `create_chat_completion` de alto nível escolhe um template padrão.
Para controle explícito, use `create_chat_completion_with`:

```rust
use llama_crab::chat::BuiltinTemplate;
use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
use llama_crab::{Llama, LlamaParams, Role};

let mut llama = Llama::load(LlamaParams::new("modelo.gguf"))?;
let history = vec![
    ChatMessage::new(Role::System, "You are a helpful assistant."),
    ChatMessage::new(Role::User, "Hi!"),
];

let resp = create_chat_completion_with(
    &mut llama,
    &history,
    BuiltinTemplate::Llama3,
    &[],
    128,
)?;
```

Ou auto-detecte a partir dos metadados do GGUF:

```rust
let template = detect_chat_format(&llama.model().metadata());
let resp = create_chat_completion_with(
    &mut llama, &history, template, &[], 128,
)?;
```

Veja o [guia de chat & tool calling](../features/chat.md) para a
lista completa de templates e as regras de auto-detecção.

## Conversa de dois turnos

Para enviar um turno de usuário subsequente, empurre a resposta do
assistant para o histórico e chame de novo:

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

Veja o [exemplo de chat com estado](stateful-chat.md) para o padrão
de REPL completo.

## Adicionando tools

Passe um `Vec<ToolDefinition>` para `create_chat_completion_with`:

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

Depois faça parse da resposta com um `ToolParser`. Veja o
[exemplo de tool calling](tools.md) para o loop completo.

## Código-fonte completo

[`examples/chat/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/chat/src/main.rs).

## Por onde ir a partir daqui

- [Chat com estado](stateful-chat.md) — REPL interativo.
- [Tool calling](tools.md) — loop de function-calling.
- [Guia de chat & tool calling](../features/chat.md) — templates,
  parsers, protocolo de tool multi-turno.
