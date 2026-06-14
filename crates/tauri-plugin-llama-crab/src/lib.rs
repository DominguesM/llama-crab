//! Tauri plugin for llama-crab.

#![allow(missing_docs)]

use serde::Deserialize;
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

/// Default and override values for the plugin.
///
/// Pass via [`init_with_config`] to apply at startup. Anything left as
/// `None` lets the corresponding per-request field win, falling back to
/// the `llama-crab` defaults.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub default_n_ctx: Option<u32>,
    pub default_n_batch: Option<u32>,
    pub default_n_ubatch: Option<u32>,
    pub default_n_threads: Option<i32>,
    pub default_n_threads_batch: Option<i32>,
    pub default_n_gpu_layers: Option<i32>,
    pub default_model_name: Option<String>,
}

/// Register the plugin with default [`Config`].
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    init_with_config::<R>(Config::default())
}

/// Register the plugin with a custom [`Config`].
pub fn init_with_config<R: Runtime>(config: Config) -> TauriPlugin<R> {
    Builder::<R>::new("llama-crab")
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
        .setup(move |app, _api| {
            app.manage(PluginState::with_config(config));
            Ok(())
        })
        .build()
}
