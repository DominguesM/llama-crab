---
title: Text and Chat Examples
---

# Text and chat examples

These examples use a small instruct GGUF by default:
`models/qwen2.5-0.5b-instruct-q4_k_m.gguf`.

## Quickstart

```bash
./run.sh quickstart
```

Shows the broadest first-run path: load a model, tokenize a prompt, generate a
text completion, run one chat completion, and try fill-in-the-middle.

## Simple completion

```bash
./run.sh simple
```

Runs a minimal one-shot completion with `Llama::load` and
`create_completion`. Direct form:

```bash
cargo run --release --bin simple -- models/qwen2.5-0.5b-instruct-q4_k_m.gguf "Once upon a time"
```

## Streaming

```bash
./run.sh streaming
```

Uses `create_completion_stream` and writes each chunk to stdout as it arrives.
Use this as the starting point for terminal UIs or HTTP streaming adapters.

## One-shot chat

```bash
./run.sh chat
```

Builds a short `Vec<ChatMessage>`, renders it with `BuiltinTemplate::ChatMl`,
and prints the assistant response.

## Stateful chat

```bash
./run.sh stateful_chat
```

Starts an interactive REPL. The example keeps conversation history in memory and
supports:

- `/clear` to reset the history.
- `/save` to print the conversation as JSON.
- `/exit` to quit.

## Speculative decoding demo

```bash
./run.sh speculative
```

Demonstrates prompt lookup drafting with `PromptLookupDecoding`. It is a compact
API demonstration, not a benchmark harness.
