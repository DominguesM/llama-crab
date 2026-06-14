# `tauri-plugin-llama-crab`

Tauri 2 plugin for in-app local inference with [`llama-crab`](https://crates.io/crates/llama-crab).

A worker thread owns the model and context. Commands cover model loading,
chat and text completions (with streaming), embeddings, rerank and tokenizer
helpers. JavaScript/TypeScript clients live in the workspace `packages/`
directory.

## Rust usage

```rust,no_run
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_llama_crab::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Commands

| Command | Description |
| --- | --- |
| `load_model` / `unload_model` | Load or unload a GGUF model by id. |
| `list_models` / `retrieve_model` | Inspect loaded models. |
| `create_chat_completion` / `stream_chat_completion` | Chat with tool calls and streaming. |
| `create_completion` / `stream_completion` | Text completions. |
| `create_embedding` | Embeddings. |
| `create_rerank` | Rerank. |
| `tokenize` / `tokenize_count` / `detokenize` | Tokenizer helpers. |
| `cancel` | Cancel an in-flight request. |

## TypeScript client

```ts
import { LlamaCrabTauri } from "@llama-crab/tauri"

const client = new LlamaCrabTauri()
```

See [`@llama-crab/tauri`](../../packages/tauri/README.md) for the full client
API and [`@llama-crab/core`](../../packages/core/README.md) for the shared
contracts.

## Resources

- [API reference (docs.rs)](https://docs.rs/tauri-plugin-llama-crab)
- [Tauri integration guide](https://dominguesm.github.io/llama-crab/tauri/)
- [Workspace README](../../README.md)

## License

Licensed under the [MIT License](../../LICENSE-MIT).
