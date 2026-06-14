# `quickstart` — First end-to-end program

The smallest program that exercises every common step: load a model,
tokenize, plain completion, chat completion, and FIM code-infill.
Use it as a starting point when you want to verify your toolchain
is wired correctly.

## Run

=== "One-command"

    ```bash
    ./examples/run.sh quickstart
    ```

=== "Manual"

    ```bash
    ./scripts/download_models.sh smol
    cargo run --release --bin run_quickstart
    ```

Downloads `Qwen2.5-0.5B-Instruct-GGUF` (~400 MB) the first time.

## What it does

```rust
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

The actual text varies; timings depend on hardware. The important
part is that all three calls return without an error.

## Walk-through

### Step 1: load the model

```rust
let mut llama = Llama::load(
    LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
        .with_n_ctx(2048)
        .with_n_threads(4),
)?;
```

- `with_n_ctx(2048)` reserves a 2 K token KV cache — enough for
  short chat sessions. See the [backends guide](../guides/backends.md)
  for the trade-offs of larger contexts.
- `with_n_threads(4)` uses 4 CPU threads for the prompt ingestion
  and decode loops. Defaults to the number of physical cores.

### Step 2: plain completion

```rust
let resp = llama.create_completion("The capital of France is", 16)?;
println!("{}", resp.text);
```

A single call tokenises the prompt, runs the forward pass and
samples 16 tokens. The returned [`Completion`] carries the text,
the tokens, the per-token log probabilities, and the timings.

### Step 3: chat completion

```rust
let history = vec![
    ChatMessage::new(Role::System, "You are a concise assistant."),
    ChatMessage::new(Role::User, "What is Rust in one sentence?"),
];
let resp = llama.create_chat_completion(&history, 64)?;
```

The high-level `create_chat_completion` picks a default template
and runs the same decode loop as `create_completion`. For explicit
template control, use `create_chat_completion_with`.

### Step 4: FIM infill

```rust
let fill = llama.complete_infill("fn main() {", "}")?;
```

FIM (fill-in-the-middle) is a code-specific task where the model
emits the missing body of a function. The `complete_infill` helper
renders a `BuiltinTemplate::CodeFim` template around the prefix
and suffix, then runs the normal generation loop.

## Common variations

=== "GPU offload"

    ```rust
    let mut llama = Llama::load(
        LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
            .with_n_ctx(2048)
            .with_n_gpu_layers(99),
    )?;
    ```

=== "Bigger context"

    ```rust
    let mut llama = Llama::load(
        LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
            .with_n_ctx(8192)
            .with_n_threads(8),
    )?;
    ```

=== "Mobile preset"

    ```rust
    use llama_crab::MobilePreset;
    let mut llama = Llama::load(
        LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
            .with_mobile_preset(MobilePreset::Balanced)
            .with_n_ctx(2048),
    )?;
    ```

## Full source

[`examples/quickstart/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/quickstart/src/main.rs) —
~80 lines, annotated.

## Where to next?

- [Streaming](streaming.md) — the most common next step for app
  developers.
- [Stateful chat](stateful-chat.md) — multi-turn conversation
  with growing history.
- [Architecture](../core-concepts/architecture.md) — what happens
  inside the high-level helpers.

[`Completion`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Completion.html
