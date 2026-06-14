//! Build-time permission generation for the Tauri plugin.

const COMMANDS: &[&str] = &[
    "load_model",
    "unload_model",
    "list_models",
    "retrieve_model",
    "create_chat_completion",
    "stream_chat_completion",
    "create_completion",
    "stream_completion",
    "create_embedding",
    "create_rerank",
    "tokenize",
    "tokenize_count",
    "detokenize",
    "cancel",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
