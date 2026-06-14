# Tauri Permissions

The plugin exposes Tauri v2 permissions for each IPC command.

The default permission set is:

```toml
[default]
description = "Allows local LLM inference through llama-crab."
permissions = [
  "allow-load-model",
  "allow-unload-model",
  "allow-generate",
  "allow-chat-stream",
  "allow-cancel",
  "allow-get-loaded-models",
  "allow-get-model-info"
]
```

## Permission table

| Identifier | Command |
| --- | --- |
| `llama-crab:allow-load-model` | Enables `load_model`. |
| `llama-crab:deny-load-model` | Denies `load_model`. |
| `llama-crab:allow-unload-model` | Enables `unload_model`. |
| `llama-crab:deny-unload-model` | Denies `unload_model`. |
| `llama-crab:allow-generate` | Enables `generate`. |
| `llama-crab:deny-generate` | Denies `generate`. |
| `llama-crab:allow-chat-stream` | Enables `chat_stream`. |
| `llama-crab:deny-chat-stream` | Denies `chat_stream`. |
| `llama-crab:allow-cancel` | Enables `cancel`. |
| `llama-crab:deny-cancel` | Denies `cancel`. |
| `llama-crab:allow-get-loaded-models` | Enables `get_loaded_models`. |
| `llama-crab:deny-get-loaded-models` | Denies `get_loaded_models`. |
| `llama-crab:allow-get-model-info` | Enables `get_model_info`. |
| `llama-crab:deny-get-model-info` | Denies `get_model_info`. |

## Practical guidance

Use the default permission set for a local-first app that exposes all model operations. For a narrower UI, grant only the commands the window needs. For example, a read-only diagnostics view may need `get_loaded_models` and `get_model_info` but not `load_model`, `unload_model`, or generation commands.

Remember that permissions gate IPC access. They do not validate model paths, model provenance, prompt size, or resource usage. Apply those rules in your application layer.
