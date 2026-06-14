# Tauri

`tauri-plugin-llama-crab` embeds local llama-crab inference inside a Tauri application. It registers a plugin named `llama-crab`, manages loaded models in Tauri state, and exposes IPC commands for loading models, generating text, streaming chat tokens, cancellation, and model inspection.

The TypeScript package `@llama-crab/tauri` provides an OpenAI-like client on top of those IPC commands. For most app code, prefer the TypeScript client instead of invoking plugin commands directly.

## Main pieces

| Piece | Purpose |
| --- | --- |
| Rust plugin | Registers `plugin:llama-crab` IPC commands and owns model workers. |
| Plugin state | Tracks loaded model ids, model metadata, workers, and cancellation flags. |
| Worker thread | Owns one loaded `Llama` instance per model and runs inference work. |
| Permissions | Tauri v2 command permissions that gate model IPC access. |
| `@llama-crab/tauri` | TypeScript client for model management and chat completions. |

## Command surface

The plugin currently registers:

| Command | Purpose |
| --- | --- |
| `load_model` | Load a GGUF model and store it by id. |
| `unload_model` | Remove a loaded model and shut down its worker. |
| `generate` | Run a text completion request against a loaded model. |
| `chat_stream` | Stream chat tokens through a Tauri `Channel`. |
| `cancel` | Mark an active request as cancelled. |
| `get_loaded_models` | Return loaded model ids. |
| `get_model_info` | Return metadata for one loaded model. |
