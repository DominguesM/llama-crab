use tauri::{ipc::Channel, State};
use uuid::Uuid;

use crate::{
    error::{PluginError, Result},
    models::{
        ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, CompletionChunkFrame,
        CompletionRequest, CompletionResponse, DetokenizeRequest, DetokenizeResponse,
        EmbeddingRequest, EmbeddingResponse, LoadModelRequest, LoadModelResponse, LoadedModelInfo,
        ModelListResponse, RerankRequest, RerankResponse, TokenizeCountResponse, TokenizeRequest,
        TokenizeResponse,
    },
    state::PluginState,
    worker::WorkerHandle,
};

#[tauri::command]
pub async fn load_model(
    state: State<'_, PluginState>,
    payload: LoadModelRequest,
) -> Result<LoadModelResponse> {
    let id = payload
        .id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    if state.model_info(&id).is_some() {
        return Err(PluginError::invalid_request(format!(
            "model `{id}` is already loaded"
        )));
    }
    let info = LoadedModelInfo::new(
        id,
        payload.path.clone(),
        payload.kind,
        payload.mobile_preset.map(|preset| preset.as_str().into()),
        payload.pooling.map(|pooling| pooling.as_str().into()),
        payload.mmproj_path.clone(),
    );
    let params = payload.llama_params();
    let worker = tauri::async_runtime::spawn_blocking(move || WorkerHandle::load(params))
        .await
        .map_err(|error| PluginError::worker(error.to_string()))??;

    let response = info.load_response();
    state.insert_loaded_model(info, worker);
    Ok(response)
}

#[tauri::command]
pub async fn unload_model(state: State<'_, PluginState>, id: String) -> Result<()> {
    let entry = state.remove_model(&id)?;
    if let Some(worker) = entry.worker {
        worker.shutdown();
    }
    Ok(())
}

#[tauri::command]
pub async fn list_models(state: State<'_, PluginState>) -> Result<ModelListResponse> {
    Ok(ModelListResponse {
        object: "list",
        data: state.loaded_model_infos(),
    })
}

#[tauri::command]
pub async fn retrieve_model(state: State<'_, PluginState>, id: String) -> Result<LoadedModelInfo> {
    state
        .model_info(&id)
        .ok_or_else(|| PluginError::model_not_found(&id))
}

#[tauri::command]
pub async fn create_chat_completion(
    state: State<'_, PluginState>,
    payload: ChatCompletionRequest,
) -> Result<ChatCompletionResponse> {
    let worker = state.worker(&payload.model)?;
    tauri::async_runtime::spawn_blocking(move || worker.create_chat_completion(payload))
        .await
        .map_err(|error| PluginError::worker(error.to_string()))?
}

#[tauri::command]
pub async fn stream_chat_completion(
    state: State<'_, PluginState>,
    payload: ChatCompletionRequest,
    on_chunk: Channel<ChatCompletionChunk>,
) -> Result<()> {
    let worker = state.worker(&payload.model)?;
    let request_id = Uuid::new_v4().to_string();
    let cancel = state.insert_request(request_id.clone());
    let result = tauri::async_runtime::spawn_blocking({
        let request_id = request_id.clone();
        move || worker.stream_chat_completion(request_id, payload, cancel, on_chunk)
    })
    .await;

    state.remove_request(&request_id);
    result.map_err(|error| PluginError::worker(error.to_string()))?
}

#[tauri::command]
pub async fn create_completion(
    state: State<'_, PluginState>,
    payload: CompletionRequest,
) -> Result<CompletionResponse> {
    let worker = state.worker(&payload.model)?;
    tauri::async_runtime::spawn_blocking(move || worker.create_completion(payload))
        .await
        .map_err(|error| PluginError::worker(error.to_string()))?
}

#[tauri::command]
pub async fn stream_completion(
    state: State<'_, PluginState>,
    payload: CompletionRequest,
    on_chunk: Channel<CompletionChunkFrame>,
) -> Result<()> {
    let worker = state.worker(&payload.model)?;
    let request_id = Uuid::new_v4().to_string();
    let cancel = state.insert_request(request_id.clone());
    let result = tauri::async_runtime::spawn_blocking({
        let request_id = request_id.clone();
        move || worker.stream_completion(request_id, payload, cancel, on_chunk)
    })
    .await;

    state.remove_request(&request_id);
    result.map_err(|error| PluginError::worker(error.to_string()))?
}

#[tauri::command]
pub async fn create_embedding(
    state: State<'_, PluginState>,
    payload: EmbeddingRequest,
) -> Result<EmbeddingResponse> {
    let worker = state.worker(&payload.model)?;
    tauri::async_runtime::spawn_blocking(move || worker.create_embedding(payload))
        .await
        .map_err(|error| PluginError::worker(error.to_string()))?
}

#[tauri::command]
pub async fn create_rerank(
    state: State<'_, PluginState>,
    payload: RerankRequest,
) -> Result<RerankResponse> {
    let worker = state.worker(&payload.model)?;
    tauri::async_runtime::spawn_blocking(move || worker.create_rerank(payload))
        .await
        .map_err(|error| PluginError::worker(error.to_string()))?
}

#[tauri::command]
pub async fn tokenize(
    state: State<'_, PluginState>,
    payload: TokenizeRequest,
) -> Result<TokenizeResponse> {
    let worker = state.worker(&payload.model)?;
    tauri::async_runtime::spawn_blocking(move || worker.tokenize(payload))
        .await
        .map_err(|error| PluginError::worker(error.to_string()))?
}

#[tauri::command]
pub async fn tokenize_count(
    state: State<'_, PluginState>,
    payload: TokenizeRequest,
) -> Result<TokenizeCountResponse> {
    let worker = state.worker(&payload.model)?;
    tauri::async_runtime::spawn_blocking(move || worker.tokenize_count(payload))
        .await
        .map_err(|error| PluginError::worker(error.to_string()))?
}

#[tauri::command]
pub async fn detokenize(
    state: State<'_, PluginState>,
    payload: DetokenizeRequest,
) -> Result<DetokenizeResponse> {
    let worker = state.worker(&payload.model)?;
    tauri::async_runtime::spawn_blocking(move || worker.detokenize(payload))
        .await
        .map_err(|error| PluginError::worker(error.to_string()))?
}

#[tauri::command]
pub async fn cancel(state: State<'_, PluginState>, request_id: String) -> Result<()> {
    state.cancel_request(&request_id);
    Ok(())
}
