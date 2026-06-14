# Tauri Plugin

Install the plugin in your Tauri application by registering `tauri_plugin_llama_crab::init()` in the builder.

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_llama_crab::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

The plugin setup stores `PluginState::default()` in Tauri state and registers the IPC command handler for the `llama-crab` plugin namespace.

## Runtime flow

1. The frontend asks to load a model.
2. `load_model` creates a model id if one was not provided.
3. The plugin loads `LlamaParams` on a blocking worker thread.
4. The loaded model id and worker handle are stored in plugin state.
5. Generation and chat commands look up the worker by model id.
6. `unload_model` removes the model and asks the worker to shut down.

## Commands

| Command | Input | Output |
| --- | --- | --- |
| `load_model` | Load model payload with path, optional id, and runtime parameters. | `{ id }` |
| `unload_model` | Model id. | No value. |
| `generate` | Text generation payload. | `{ text }` |
| `chat_stream` | Chat payload plus a Tauri `Channel<TokenEvent>`. | Token events on the channel. |
| `cancel` | Request id. | No value. |
| `get_loaded_models` | None. | Array of model ids. |
| `get_model_info` | Model id. | Loaded model metadata. |

The generated IPC command prefix used by the TypeScript client is:

```text
plugin:llama-crab|load_model
plugin:llama-crab|unload_model
plugin:llama-crab|get_loaded_models
plugin:llama-crab|get_model_info
plugin:llama-crab|chat_stream
```

## Worker behavior

`WorkerHandle::load` starts a thread named `llama-crab-model-worker` and waits for model loading to finish. Completion work is sent to that worker over an internal channel.

Streaming chat sends `TokenEvent` values to the frontend channel:

```ts
type TokenEvent = {
  requestId: string
  token: string
  index: number
  done?: boolean
  stopReason?: string
}
```

When cancellation is requested, the worker callback checks the cancellation flag and stops the stream.

## Errors

Plugin errors serialize as camelCase objects:

```json
{
  "kind": "modelNotFound",
  "message": "model `local` is not loaded"
}
```

Known kinds in the current code include `invalidRequest`, `modelNotFound`, `worker`, and `inference`.
