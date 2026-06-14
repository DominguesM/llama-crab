# `quickstart` — Primeiro programa ponta a ponta

O menor programa que exercita cada passo comum: carregar um modelo,
tokenizar, completion de texto, completion de chat e infill de
código FIM. Use como ponto de partida quando quiser verificar se
sua toolchain está corretamente conectada.

## Execute

=== "Um comando"

    ```bash
    ./examples/run.sh quickstart
    ```

=== "Manual"

    ```bash
    ./scripts/download_models.sh smol
    cargo run --release --bin run_quickstart
    ```

Baixa o `Qwen2.5-0.5B-Instruct-GGUF` (~400 MB) na primeira vez.

## O que ele faz

```rust
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::{Llama, LlamaParams, Role};

let mut llama = Llama::load(
    LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
        .with_n_ctx(2048)
        .with_n_threads(4),
)?;

// 1. Completion de texto simples.
let resp = llama.create_completion("The capital of France is", 16)?;

// 2. Completion de chat com um turno system + user.
let history = vec![
    ChatMessage::new(Role::System, "You are a concise assistant."),
    ChatMessage::new(Role::User, "What is Rust in one sentence?"),
];
let resp = llama.create_chat_completion(&history, 64)?;

// 3. FIM code infill.
let fill = llama.complete_infill("fn main() {", "}")?;
```

## Saída esperada

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

O texto real varia; as temporizações dependem do hardware. O
importante é que todas as três chamadas retornem sem erro.

## Passo a passo

### Passo 1: carregar o modelo

```rust
let mut llama = Llama::load(
    LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
        .with_n_ctx(2048)
        .with_n_threads(4),
)?;
```

- `with_n_ctx(2048)` reserva um cache KV de 2K tokens — suficiente
  para sessões de chat curtas. Veja o [guia de backends](../guides/backends.md)
  para os trade-offs de contextos maiores.
- `with_n_threads(4)` usa 4 threads de CPU para os loops de
  ingestão de prompt e decodificação. O padrão é o número de
  cores físicos.

### Passo 2: completion simples

```rust
let resp = llama.create_completion("The capital of France is", 16)?;
println!("{}", resp.text);
```

Uma única chamada tokeniza o prompt, roda a passada forward e
amostra 16 tokens. O [`Completion`] retornado carrega o texto, os
tokens, as log-probabilidades por token e as temporizações.

### Passo 3: completion de chat

```rust
let history = vec![
    ChatMessage::new(Role::System, "You are a concise assistant."),
    ChatMessage::new(Role::User, "What is Rust in one sentence?"),
];
let resp = llama.create_chat_completion(&history, 64)?;
```

O `create_chat_completion` de alto nível escolhe um template
padrão e roda o mesmo loop de decodificação que `create_completion`.
Para controle explícito do template, use `create_chat_completion_with`.

### Passo 4: infill FIM

```rust
let fill = llama.complete_infill("fn main() {", "}")?;
```

FIM (fill-in-the-middle) é uma tarefa específica de código onde o
modelo emite o corpo ausente de uma função. O helper
`complete_infill` renderiza um template `BuiltinTemplate::CodeFim`
ao redor do prefixo e sufixo, depois roda o loop de geração
normal.

## Variações comuns

=== "Offload de GPU"

    ```rust
    let mut llama = Llama::load(
        LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
            .with_n_ctx(2048)
            .with_n_gpu_layers(99),
    )?;
    ```

=== "Contexto maior"

    ```rust
    let mut llama = Llama::load(
        LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
            .with_n_ctx(8192)
            .with_n_threads(8),
    )?;
    ```

=== "Preset mobile"

    ```rust
    use llama_crab::MobilePreset;
    let mut llama = Llama::load(
        LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
            .with_mobile_preset(MobilePreset::Balanced)
            .with_n_ctx(2048),
    )?;
    ```

## Código-fonte completo

[`examples/quickstart/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/quickstart/src/main.rs) —
~80 linhas, anotado.

## Por onde ir a partir daqui

- [Streaming](streaming.md) — o próximo passo mais comum para
  desenvolvedores de apps.
- [Chat com estado](stateful-chat.md) — conversa multi-turno com
  histórico crescente.
- [Arquitetura](../core-concepts/architecture.md) — o que acontece
  dentro dos helpers de alto nível.

[`Completion`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Completion.html
