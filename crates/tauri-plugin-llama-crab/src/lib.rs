//! Tauri plugin for llama-crab.

#![allow(missing_docs)]

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod commands;
mod error;
mod models;
mod state;
mod worker;

pub use error::{PluginError, Result};
pub use models::{
    ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ChatMessage,
    CompletionChunkFrame, CompletionRequest, CompletionResponse, DetokenizeRequest,
    DetokenizeResponse, EmbeddingInput, EmbeddingRequest, EmbeddingResponse, LoadModelRequest,
    LoadModelResponse, LoadedModelInfo, MobilePresetName, ModelKind, ModelListResponse,
    PoolingName, RerankRequest, RerankResponse, TokenizeCountResponse, TokenizeRequest,
    TokenizeResponse,
};
pub use state::PluginState;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("llama-crab")
        .invoke_handler(tauri::generate_handler![
            commands::load_model,
            commands::unload_model,
            commands::list_models,
            commands::retrieve_model,
            commands::create_chat_completion,
            commands::stream_chat_completion,
            commands::create_completion,
            commands::stream_completion,
            commands::create_embedding,
            commands::create_rerank,
            commands::tokenize,
            commands::tokenize_count,
            commands::detokenize,
            commands::cancel,
        ])
        .setup(|app, _api| {
            app.manage(PluginState::default());
            Ok(())
        })
        .build()
}
