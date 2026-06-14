//! HTTP server binary for local llama-crab inference.

use std::collections::BTreeMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::Parser;
use futures_util::stream;
use futures_util::StreamExt;
use llama_crab::chat::tool_call::{extract_tool_calls, ToolFormat};
use llama_crab::chat::{
    render_builtin, BuiltinTemplate, ChatMessage, Role, ToolCall, ToolDefinition,
};
use llama_crab::high_level::completion::{
    json_schema_grammar, CompletionChunk, CompletionLogprobs, CompletionOptions, SamplingOptions,
    StreamControl,
};
use llama_crab::json_schema::json_object_grammar;
use llama_crab::sampling::LlamaSampler;
use llama_crab::LlamaLogitBias;
use llama_crab::{Completion, Llama, LlamaParams, LlamaToken};
use serde::de;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc as tokio_mpsc, oneshot};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, env = "LLAMA_CRAB_MODEL")]
    model: String,
    #[arg(long, default_value = "127.0.0.1", env = "LLAMA_CRAB_HOST")]
    host: String,
    #[arg(long, default_value_t = 8080, env = "LLAMA_CRAB_PORT")]
    port: u16,
    #[arg(long, default_value_t = 2048, env = "LLAMA_CRAB_N_CTX")]
    n_ctx: u32,
    #[arg(long, default_value_t = 0, env = "LLAMA_CRAB_N_GPU_LAYERS")]
    n_gpu_layers: i32,
    #[arg(long, default_value = "llama-crab", env = "LLAMA_CRAB_MODEL_NAME")]
    model_name: String,
    #[arg(long, default_value_t = false, env = "LLAMA_CRAB_EMBEDDINGS")]
    embeddings: bool,
}

#[derive(Clone)]
struct AppState {
    model_name: String,
    jobs: mpsc::Sender<Job>,
}

enum Job {
    Complete {
        request: CompletionRequest,
        reply: oneshot::Sender<Result<CompletionResponse, String>>,
    },
    CompleteStream {
        request: CompletionRequest,
        chunks: tokio_mpsc::UnboundedSender<Result<StreamFrame, String>>,
    },
    Chat {
        request: ChatRequest,
        reply: oneshot::Sender<Result<ChatResponse, String>>,
    },
    ChatStream {
        request: ChatRequest,
        chunks: tokio_mpsc::UnboundedSender<Result<StreamFrame, String>>,
    },
    Embed {
        request: EmbeddingRequest,
        reply: oneshot::Sender<Result<EmbeddingResponse, String>>,
    },
    Tokenize {
        request: TokenizeRequest,
        reply: oneshot::Sender<Result<TokenizeResponse, String>>,
    },
    TokenCount {
        request: TokenizeRequest,
        reply: oneshot::Sender<Result<TokenizeCountResponse, String>>,
    },
    Detokenize {
        request: DetokenizeRequest,
        reply: oneshot::Sender<Result<DetokenizeResponse, String>>,
    },
}

#[derive(Debug, Deserialize)]
struct CompletionRequest {
    #[serde(default)]
    model: Option<String>,
    prompt: CompletionPrompt,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    max_tokens: Option<usize>,
    #[serde(default)]
    min_tokens: Option<usize>,
    #[serde(default)]
    logprobs: Option<usize>,
    #[serde(default)]
    n: Option<usize>,
    #[serde(default)]
    best_of: Option<usize>,
    #[serde(default, deserialize_with = "deserialize_stop_sequences")]
    stop: Vec<String>,
    #[serde(default)]
    stream: bool,
    #[serde(default)]
    echo: bool,
    #[serde(default)]
    suffix: Option<String>,
    #[serde(default)]
    logit_bias: BTreeMap<String, f32>,
    #[serde(default)]
    logit_bias_type: Option<String>,
    #[serde(flatten)]
    sampling: SamplingRequest,
    #[serde(flatten)]
    structured: StructuredRequest,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum CompletionPrompt {
    Single(String),
    Many(Vec<String>),
}

#[derive(Debug, Serialize)]
struct CompletionResponse {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<CompletionChoice>,
    usage: Usage,
}

#[derive(Debug, Serialize)]
struct CompletionChoice {
    text: String,
    index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<CompletionLogprobsResponse>,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct CompletionLogprobsResponse {
    tokens: Vec<String>,
    text_offset: Vec<usize>,
    token_logprobs: Vec<Option<f32>>,
    top_logprobs: Vec<Option<BTreeMap<String, f32>>>,
}

#[derive(Debug, Deserialize)]
struct ChatRequest {
    #[serde(default)]
    model: Option<String>,
    messages: Vec<ChatRequestMessage>,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    max_tokens: Option<usize>,
    #[serde(default)]
    min_tokens: Option<usize>,
    #[serde(default)]
    logprobs: Option<bool>,
    #[serde(default)]
    top_logprobs: Option<usize>,
    #[serde(default)]
    n: Option<usize>,
    #[serde(default)]
    stream: bool,
    #[serde(default)]
    template: Option<String>,
    #[serde(default, deserialize_with = "deserialize_stop_sequences")]
    stop: Vec<String>,
    #[serde(default)]
    tools: Vec<ChatToolRequest>,
    #[serde(default)]
    tool_choice: Option<Value>,
    #[serde(default)]
    function_call: Option<Value>,
    #[serde(default)]
    logit_bias: BTreeMap<String, f32>,
    #[serde(default)]
    logit_bias_type: Option<String>,
    #[serde(flatten)]
    sampling: SamplingRequest,
    #[serde(flatten)]
    structured: StructuredRequest,
}

#[derive(Debug, Default, Deserialize)]
struct SamplingRequest {
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_k: Option<i32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    tfs_z: Option<f32>,
    #[serde(default)]
    min_p: Option<f32>,
    #[serde(default)]
    typical_p: Option<f32>,
    #[serde(default)]
    min_keep: Option<usize>,
    #[serde(default)]
    penalty_last_n: Option<i32>,
    #[serde(default)]
    repeat_penalty: Option<f32>,
    #[serde(default)]
    frequency_penalty: Option<f32>,
    #[serde(default)]
    presence_penalty: Option<f32>,
    #[serde(default)]
    mirostat_mode: Option<i32>,
    #[serde(default)]
    mirostat_tau: Option<f32>,
    #[serde(default)]
    mirostat_eta: Option<f32>,
    #[serde(default)]
    seed: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
struct StructuredRequest {
    #[serde(default)]
    grammar: Option<String>,
    #[serde(default)]
    json_schema: Option<Value>,
    #[serde(default)]
    response_format: Option<ResponseFormatRequest>,
    #[serde(default)]
    grammar_root: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResponseFormatRequest {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    schema: Option<Value>,
    #[serde(default)]
    json_schema: Option<ResponseFormatJsonSchema>,
}

#[derive(Debug, Deserialize)]
struct ResponseFormatJsonSchema {
    #[serde(default)]
    schema: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatRequestMessage {
    role: String,
    #[serde(default, deserialize_with = "deserialize_chat_content")]
    content: String,
    #[serde(default)]
    tool_call_id: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ChatMessageToolCallRequest>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatToolRequest {
    #[serde(rename = "type")]
    kind: String,
    function: ChatFunctionToolRequest,
}

#[derive(Debug, Deserialize)]
struct ChatFunctionToolRequest {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_tool_parameters")]
    parameters: Value,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatMessageToolCallRequest {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    function: ChatMessageToolCallFunctionRequest,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatMessageToolCallFunctionRequest {
    name: String,
    arguments: Value,
}

#[derive(Debug, Serialize)]
struct ChatResponse {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<ChatChoice>,
    usage: Usage,
}

#[derive(Debug, Serialize)]
struct ChatChoice {
    index: u32,
    message: ChatResponseMessage,
    logprobs: Option<ChatLogprobsResponse>,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatResponseMessage {
    role: &'static str,
    content: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool_calls: Vec<ChatResponseToolCall>,
}

#[derive(Debug, Serialize)]
struct ChatResponseToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: &'static str,
    function: ChatResponseToolCallFunction,
}

#[derive(Debug, Serialize)]
struct ChatResponseToolCallFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize)]
struct ChatLogprobsResponse {
    content: Option<Vec<ChatLogprobToken>>,
    refusal: Option<Vec<ChatLogprobToken>>,
}

#[derive(Debug, Serialize)]
struct ChatLogprobToken {
    token: String,
    logprob: Option<f32>,
    bytes: Option<Vec<u8>>,
    top_logprobs: Vec<ChatTopLogprobToken>,
}

#[derive(Debug, Serialize)]
struct ChatTopLogprobToken {
    token: String,
    logprob: f32,
    bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChoiceCounts {
    public: usize,
    internal: usize,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EmbeddingInput {
    Single(String),
    Many(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct EmbeddingRequest {
    #[serde(default)]
    model: Option<String>,
    input: EmbeddingInput,
    #[serde(default)]
    user: Option<String>,
    #[serde(default = "default_normalize")]
    normalize: bool,
}

#[derive(Debug, Serialize)]
struct EmbeddingResponse {
    object: &'static str,
    data: Vec<EmbeddingItem>,
    model: String,
    usage: EmbeddingUsage,
}

#[derive(Debug, Serialize)]
struct EmbeddingItem {
    object: &'static str,
    embedding: Vec<f32>,
    index: u32,
}

#[derive(Debug, Serialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize)]
struct EmbeddingUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct TokenizeRequest {
    #[serde(default)]
    model: Option<String>,
    input: String,
}

#[derive(Debug, Serialize)]
struct TokenizeResponse {
    tokens: Vec<i32>,
}

#[derive(Debug, Serialize)]
struct TokenizeCountResponse {
    count: usize,
}

#[derive(Debug, Deserialize)]
struct DetokenizeRequest {
    #[serde(default)]
    model: Option<String>,
    tokens: Vec<i32>,
}

#[derive(Debug, Serialize)]
struct DetokenizeResponse {
    text: String,
}

#[derive(Debug, Serialize)]
struct ModelList {
    object: &'static str,
    data: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
struct ModelInfo {
    id: String,
    object: &'static str,
    created: u64,
    owned_by: &'static str,
    permissions: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: ErrorMessage,
}

#[derive(Debug, Serialize)]
struct ErrorMessage {
    message: String,
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Debug, Serialize)]
struct StreamFrame {
    id: String,
    object: &'static str,
    created: u64,
    model: String,
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Serialize)]
struct StreamChoice {
    index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delta: Option<ChatStreamDelta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logprobs: Option<StreamLogprobs>,
    finish_reason: Option<String>,
}

#[derive(Debug)]
enum StreamEvent {
    Frame(StreamFrame),
    Error(String),
    Done,
}

#[derive(Debug, Serialize)]
struct ChatStreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum StreamLogprobs {
    Completion(CompletionLogprobsResponse),
    Chat(ChatLogprobsResponse),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let state = AppState {
        model_name: args.model_name.clone(),
        jobs: spawn_worker(&args)?,
    };
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(models))
        .route("/v1/completions", post(completions))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/embeddings", post(embeddings))
        .route("/extras/tokenize", post(tokenize))
        .route("/extras/tokenize/count", post(tokenize_count))
        .route("/extras/detokenize", post(detokenize))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    eprintln!("llama-crab-server listening on http://{addr}");
    eprintln!("  model : {}", args.model_name);
    eprintln!(
        "  routes: /health, /v1/models, /v1/completions, /v1/chat/completions, \
         /v1/embeddings, /extras/tokenize, /extras/tokenize/count, /extras/detokenize"
    );
    eprintln!("  ctrl+c to stop");
    tracing::info!(%addr, model = %args.model_name, "starting llama-crab-server");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn models(State(state): State<AppState>) -> Json<ModelList> {
    Json(ModelList {
        object: "list",
        data: vec![ModelInfo {
            id: state.model_name,
            object: "model",
            created: unix_timestamp(),
            owned_by: "me",
            permissions: Vec::new(),
        }],
    })
}

async fn completions(
    State(state): State<AppState>,
    Json(request): Json<CompletionRequest>,
) -> Response {
    if request.stream {
        let (tx, rx) = tokio_mpsc::unbounded_channel();
        if let Err(err) = state.jobs.send(Job::CompleteStream {
            request,
            chunks: tx,
        }) {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
        }
        return sse_response(rx);
    }
    let (tx, rx) = oneshot::channel();
    if let Err(err) = state.jobs.send(Job::Complete { request, reply: tx }) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
    }
    match rx.await {
        Ok(Ok(response)) => Json(response).into_response(),
        Ok(Err(err)) => error_response(StatusCode::BAD_REQUEST, err),
        Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

async fn chat_completions(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Response {
    if request.stream {
        let (tx, rx) = tokio_mpsc::unbounded_channel();
        if let Err(err) = state.jobs.send(Job::ChatStream {
            request,
            chunks: tx,
        }) {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
        }
        return sse_response(rx);
    }
    let (tx, rx) = oneshot::channel();
    if let Err(err) = state.jobs.send(Job::Chat { request, reply: tx }) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
    }
    match rx.await {
        Ok(Ok(response)) => Json(response).into_response(),
        Ok(Err(err)) => error_response(StatusCode::BAD_REQUEST, err),
        Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

async fn embeddings(
    State(state): State<AppState>,
    Json(request): Json<EmbeddingRequest>,
) -> Response {
    let (tx, rx) = oneshot::channel();
    if let Err(err) = state.jobs.send(Job::Embed { request, reply: tx }) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
    }
    match rx.await {
        Ok(Ok(response)) => Json(response).into_response(),
        Ok(Err(err)) => error_response(StatusCode::BAD_REQUEST, err),
        Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

async fn tokenize(State(state): State<AppState>, Json(request): Json<TokenizeRequest>) -> Response {
    let (tx, rx) = oneshot::channel();
    if let Err(err) = state.jobs.send(Job::Tokenize { request, reply: tx }) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
    }
    match rx.await {
        Ok(Ok(response)) => Json(response).into_response(),
        Ok(Err(err)) => error_response(StatusCode::BAD_REQUEST, err),
        Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

async fn tokenize_count(
    State(state): State<AppState>,
    Json(request): Json<TokenizeRequest>,
) -> Response {
    let (tx, rx) = oneshot::channel();
    if let Err(err) = state.jobs.send(Job::TokenCount { request, reply: tx }) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
    }
    match rx.await {
        Ok(Ok(response)) => Json(response).into_response(),
        Ok(Err(err)) => error_response(StatusCode::BAD_REQUEST, err),
        Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

async fn detokenize(
    State(state): State<AppState>,
    Json(request): Json<DetokenizeRequest>,
) -> Response {
    let (tx, rx) = oneshot::channel();
    if let Err(err) = state.jobs.send(Job::Detokenize { request, reply: tx }) {
        return error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string());
    }
    match rx.await {
        Ok(Ok(response)) => Json(response).into_response(),
        Ok(Err(err)) => error_response(StatusCode::BAD_REQUEST, err),
        Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

fn spawn_worker(args: &Args) -> Result<mpsc::Sender<Job>, Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<Job>();
    let params = LlamaParams::new(&args.model)
        .with_n_ctx(args.n_ctx)
        .with_n_gpu_layers(args.n_gpu_layers)
        .with_embeddings(args.embeddings);
    let model_name = args.model_name.clone();
    thread::Builder::new()
        .name("llama-crab-worker".to_string())
        .spawn(move || {
            let mut llama = match Llama::load(params) {
                Ok(llama) => llama,
                Err(err) => {
                    tracing::error!(error = %err, "failed to load model");
                    return;
                }
            };
            for job in rx {
                match job {
                    Job::Complete { request, reply } => {
                        let _ = reply.send(run_completion(&mut llama, &model_name, request));
                    }
                    Job::CompleteStream { request, chunks } => {
                        run_completion_stream(&mut llama, &model_name, request, chunks);
                    }
                    Job::Chat { request, reply } => {
                        let _ = reply.send(run_chat(&mut llama, &model_name, request));
                    }
                    Job::ChatStream { request, chunks } => {
                        run_chat_stream(&mut llama, &model_name, request, chunks);
                    }
                    Job::Embed { request, reply } => {
                        let _ = reply.send(run_embeddings(&mut llama, &model_name, request));
                    }
                    Job::Tokenize { request, reply } => {
                        let _ = reply.send(run_tokenize(&llama, request));
                    }
                    Job::TokenCount { request, reply } => {
                        let _ = reply.send(run_tokenize_count(&llama, request));
                    }
                    Job::Detokenize { request, reply } => {
                        let _ = reply.send(run_detokenize(&llama, request));
                    }
                }
            }
        })?;
    Ok(tx)
}

fn run_completion(
    llama: &mut Llama,
    model_name: &str,
    request: CompletionRequest,
) -> Result<CompletionResponse, String> {
    let _requested_model = request.model.as_deref();
    let _request_user = request.user.as_deref();
    let counts = request.choice_counts()?;
    let expose_logprobs = request.logprobs.is_some();
    let score_candidates = counts.internal > counts.public;
    let mut choices = Vec::new();
    let mut prompt_tokens = 0_u32;
    let mut completion_tokens = 0_u32;
    let mut choice_index = 0_u32;
    for prompt in request.prompts() {
        prompt_tokens += llama
            .model()
            .tokenize(prompt, true, true)
            .map_err(|err| err.to_string())?
            .len() as u32;
        let mut candidates = Vec::new();
        for _ in 0..counts.internal {
            let mut options = request.completion_options(SamplingOptions::default(), |text| {
                llama
                    .model()
                    .tokenize(text, false, true)
                    .map_err(|err| err.to_string())
            })?;
            if score_candidates && options.logprobs.is_none() {
                options = options.with_logprobs(0);
            }
            let mut sampler = build_request_sampler(llama, &options, &request.structured)?;
            candidates.push(
                llama
                    .create_completion_with_sampler(prompt, options, &mut sampler)
                    .map_err(|err| err.to_string())?,
            );
        }

        let selected = select_completion_choices(candidates, counts.public);
        for completion in selected {
            completion_tokens += completion.n_tokens as u32;
            let logprobs = if expose_logprobs {
                completion.logprobs.map(completion_logprobs_response)
            } else {
                None
            };
            choices.push(CompletionChoice {
                text: completion.text,
                index: choice_index,
                logprobs,
                finish_reason: Some(stop_reason(completion.stop_reason)),
            });
            choice_index += 1;
        }
    }
    Ok(CompletionResponse {
        id: format!("cmpl-{}", unix_timestamp()),
        object: "text_completion",
        created: unix_timestamp(),
        model: model_name.to_string(),
        choices,
        usage: Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        },
    })
}

fn run_completion_stream(
    llama: &mut Llama,
    model_name: &str,
    request: CompletionRequest,
    chunks: tokio_mpsc::UnboundedSender<Result<StreamFrame, String>>,
) {
    let _requested_model = request.model.as_deref();
    let _request_user = request.user.as_deref();
    let prompts = request.prompts();
    if prompts.len() != 1 {
        let _ = chunks.send(Err(
            "streaming completions require exactly one prompt".to_string()
        ));
        return;
    }
    let prompt = prompts[0].to_string();
    let id = format!("cmpl-{}", unix_timestamp());
    let created = unix_timestamp();
    let options = match request.completion_options(SamplingOptions::default(), |text| {
        llama
            .model()
            .tokenize(text, false, true)
            .map_err(|err| err.to_string())
    }) {
        Ok(options) => options,
        Err(err) => {
            let _ = chunks.send(Err(err));
            return;
        }
    };
    let mut sampler = match build_request_sampler(llama, &options, &request.structured) {
        Ok(sampler) => sampler,
        Err(err) => {
            let _ = chunks.send(Err(err));
            return;
        }
    };
    let result =
        llama.create_completion_stream_with_sampler(&prompt, options, &mut sampler, |chunk| {
            let _ = chunks.send(Ok(stream_frame(
                &id,
                "text_completion",
                created,
                model_name,
                chunk,
            )));
            StreamControl::Continue
        });
    if let Err(err) = result {
        let _ = chunks.send(Err(err.to_string()));
    }
}

fn select_completion_choices(
    mut candidates: Vec<Completion>,
    public_count: usize,
) -> Vec<Completion> {
    candidates.sort_by(|left, right| completion_score(right).total_cmp(&completion_score(left)));
    candidates.truncate(public_count);
    candidates
}

fn completion_score(completion: &Completion) -> f32 {
    let Some(logprobs) = &completion.logprobs else {
        return f32::NEG_INFINITY;
    };
    let mut sum = 0.0_f32;
    let mut count = 0_usize;
    for logprob in logprobs.token_logprobs.iter().flatten().copied() {
        if !logprob.is_finite() {
            return f32::NEG_INFINITY;
        }
        sum += logprob;
        count += 1;
    }
    if count == 0 {
        f32::NEG_INFINITY
    } else {
        sum / count as f32
    }
}

fn run_chat(
    llama: &mut Llama,
    model_name: &str,
    request: ChatRequest,
) -> Result<ChatResponse, String> {
    let _requested_model = request.model.as_deref();
    let _request_user = request.user.as_deref();
    let counts = request.choice_counts()?;
    let prompt = request.chat_prompt()?;
    let prompt_tokens = llama
        .model()
        .tokenize(&prompt, true, true)
        .map_err(|err| err.to_string())?
        .len() as u32;
    let mut choices = Vec::new();
    let mut completion_tokens = 0_u32;
    for index in 0..counts.public {
        let options = request.completion_options(SamplingOptions::chat(), |text| {
            llama
                .model()
                .tokenize(text, false, true)
                .map_err(|err| err.to_string())
        })?;
        let mut sampler = build_request_sampler(llama, &options, &request.structured)?;
        let completion = llama
            .create_completion_with_sampler(&prompt, options, &mut sampler)
            .map_err(|err| err.to_string())?;
        completion_tokens += llama
            .model()
            .tokenize(&completion.text, false, true)
            .map_err(|err| err.to_string())?
            .len() as u32;
        let stop_reason = completion.stop_reason;
        let message = chat_response_message(&request, completion.text)?;
        let finish_reason = chat_finish_reason(stop_reason, &message);
        choices.push(ChatChoice {
            index: index as u32,
            message,
            logprobs: completion.logprobs.map(chat_logprobs_response),
            finish_reason: Some(finish_reason),
        });
    }
    Ok(ChatResponse {
        id: format!("chatcmpl-{}", unix_timestamp()),
        object: "chat.completion",
        created: unix_timestamp(),
        model: model_name.to_string(),
        choices,
        usage: Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        },
    })
}

fn run_chat_stream(
    llama: &mut Llama,
    model_name: &str,
    request: ChatRequest,
    chunks: tokio_mpsc::UnboundedSender<Result<StreamFrame, String>>,
) {
    let _requested_model = request.model.as_deref();
    let _request_user = request.user.as_deref();
    let id = format!("chatcmpl-{}", unix_timestamp());
    let created = unix_timestamp();
    let options = match request.completion_options(SamplingOptions::chat(), |text| {
        llama
            .model()
            .tokenize(text, false, true)
            .map_err(|err| err.to_string())
    }) {
        Ok(options) => options,
        Err(err) => {
            let _ = chunks.send(Err(err));
            return;
        }
    };
    let prompt = match request.chat_prompt() {
        Ok(prompt) => prompt,
        Err(err) => {
            let _ = chunks.send(Err(err));
            return;
        }
    };
    let mut sampler = match build_request_sampler(llama, &options, &request.structured) {
        Ok(sampler) => sampler,
        Err(err) => {
            let _ = chunks.send(Err(err));
            return;
        }
    };
    let _ = chunks.send(Ok(chat_stream_role_frame(&id, created, model_name)));
    let result =
        llama.create_completion_stream_with_sampler(&prompt, options, &mut sampler, |chunk| {
            let _ = chunks.send(Ok(stream_frame(
                &id,
                "chat.completion.chunk",
                created,
                model_name,
                chunk,
            )));
            StreamControl::Continue
        });
    if let Err(err) = result {
        let _ = chunks.send(Err(err.to_string()));
    }
}

fn run_embeddings(
    llama: &mut Llama,
    model_name: &str,
    request: EmbeddingRequest,
) -> Result<EmbeddingResponse, String> {
    let _requested_model = request.model.as_deref();
    let _request_user = request.user.as_deref();
    let inputs = match request.input {
        EmbeddingInput::Single(text) => vec![text],
        EmbeddingInput::Many(texts) => texts,
    };
    let batch = llama
        .embed_texts(&inputs, request.normalize)
        .map_err(|err| err.to_string())?;
    let data = batch
        .vectors
        .into_iter()
        .enumerate()
        .map(|(index, embedding)| EmbeddingItem {
            object: "embedding",
            embedding,
            index: index as u32,
        })
        .collect();
    Ok(EmbeddingResponse {
        object: "list",
        data,
        model: model_name.to_string(),
        usage: EmbeddingUsage {
            prompt_tokens: batch.prompt_tokens,
            total_tokens: batch.total_tokens,
        },
    })
}

fn run_tokenize(llama: &Llama, request: TokenizeRequest) -> Result<TokenizeResponse, String> {
    let _requested_model = request.model.as_deref();
    let tokens = llama
        .model()
        .tokenize(&request.input, true, true)
        .map_err(|err| err.to_string())?
        .into_iter()
        .map(LlamaToken::raw)
        .collect();
    Ok(TokenizeResponse { tokens })
}

fn run_tokenize_count(
    llama: &Llama,
    request: TokenizeRequest,
) -> Result<TokenizeCountResponse, String> {
    let count = run_tokenize(llama, request)?.tokens.len();
    Ok(TokenizeCountResponse { count })
}

fn run_detokenize(llama: &Llama, request: DetokenizeRequest) -> Result<DetokenizeResponse, String> {
    let _requested_model = request.model.as_deref();
    let tokens = request
        .tokens
        .into_iter()
        .map(LlamaToken::from)
        .collect::<Vec<_>>();
    let text = llama
        .model()
        .detokenize(&tokens, true)
        .map_err(|err| err.to_string())?;
    Ok(DetokenizeResponse { text })
}

fn convert_messages(messages: Vec<ChatRequestMessage>) -> Result<Vec<ChatMessage>, String> {
    messages
        .into_iter()
        .map(|message| {
            let role = match message.role.as_str() {
                "system" => Role::System,
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "tool" => Role::Tool,
                "function" => Role::Tool,
                other => return Err(format!("unknown chat role: {other}")),
            };
            let mut converted = ChatMessage::new(role, message.content);
            converted.tool_call_id = message.tool_call_id;
            converted.tool_calls = message
                .tool_calls
                .iter()
                .map(ChatMessageToolCallRequest::to_tool_call)
                .collect::<Result<Vec<_>, _>>()?;
            converted.name = message.name;
            Ok(converted)
        })
        .collect()
}

impl CompletionRequest {
    fn prompts(&self) -> Vec<&str> {
        match &self.prompt {
            CompletionPrompt::Single(prompt) => vec![prompt.as_str()],
            CompletionPrompt::Many(prompts) => prompts.iter().map(String::as_str).collect(),
        }
    }

    fn completion_options(
        &self,
        default_sampling: SamplingOptions,
        tokenize_bias: impl FnMut(&str) -> Result<Vec<LlamaToken>, String>,
    ) -> Result<CompletionOptions, String> {
        let mut options = CompletionOptions::sampled(self.max_tokens.unwrap_or(16))
            .with_sampling(self.sampling.apply(default_sampling))
            .with_min_tokens(self.min_tokens.unwrap_or(0))
            .with_stop_sequences(self.stop.iter().cloned())
            .with_echo_prompt(self.echo)
            .with_logit_biases(parse_logit_bias(
                &self.logit_bias,
                self.logit_bias_type.as_deref(),
                tokenize_bias,
            )?);
        if let Some(suffix) = &self.suffix {
            options = options.with_suffix(suffix.clone());
        }
        if let Some(logprobs) = self.logprobs {
            options = options.with_logprobs(logprobs);
        }
        Ok(options)
    }

    fn choice_counts(&self) -> Result<ChoiceCounts, String> {
        choice_counts(self.n, self.best_of)
    }
}

impl ChatRequest {
    fn chat_tools(&self) -> Result<Vec<ToolDefinition>, String> {
        let tools = self
            .tools
            .iter()
            .map(ChatToolRequest::to_tool_definition)
            .collect::<Result<Vec<_>, _>>()?;
        self.validate_tool_selection(&tools)?;
        Ok(tools)
    }

    fn chat_prompt(&self) -> Result<String, String> {
        let template = request_template(self.template.as_deref());
        let messages = convert_messages(self.messages.clone())?;
        let tools = self.chat_tools()?;
        Ok(render_builtin(template, &messages, &tools, true))
    }

    fn completion_options(
        &self,
        default_sampling: SamplingOptions,
        tokenize_bias: impl FnMut(&str) -> Result<Vec<LlamaToken>, String>,
    ) -> Result<CompletionOptions, String> {
        let mut options = CompletionOptions::sampled(self.max_tokens.unwrap_or(16))
            .with_sampling(self.sampling.apply(default_sampling))
            .with_min_tokens(self.min_tokens.unwrap_or(0))
            .with_stop_sequences(self.stop.iter().cloned())
            .with_logit_biases(parse_logit_bias(
                &self.logit_bias,
                self.logit_bias_type.as_deref(),
                tokenize_bias,
            )?);
        if self.logprobs.unwrap_or(false) {
            options = options.with_logprobs(self.top_logprobs.unwrap_or(0));
        }
        Ok(options)
    }

    fn choice_counts(&self) -> Result<ChoiceCounts, String> {
        choice_counts(self.n, self.n)
    }

    fn validate_tool_selection(&self, tools: &[ToolDefinition]) -> Result<(), String> {
        if let Some(tool_choice) = &self.tool_choice {
            validate_named_tool_choice(tool_choice, tools, "tool_choice")?;
        }
        if let Some(function_call) = &self.function_call {
            validate_named_tool_choice(function_call, tools, "function_call")?;
        }
        Ok(())
    }
}

impl ChatToolRequest {
    fn to_tool_definition(&self) -> Result<ToolDefinition, String> {
        if self.kind != "function" {
            return Err(format!("unsupported chat tool type: {}", self.kind));
        }
        if self.function.name.trim().is_empty() {
            return Err("chat tool function name cannot be empty".to_string());
        }
        Ok(
            ToolDefinition::new(&self.function.name, &self.function.description)
                .with_parameters(self.function.parameters.clone()),
        )
    }
}

impl ChatMessageToolCallRequest {
    fn to_tool_call(&self) -> Result<ToolCall, String> {
        if self.kind != "function" {
            return Err(format!(
                "unsupported chat message tool call type: {}",
                self.kind
            ));
        }
        if self.id.trim().is_empty() {
            return Err("chat message tool call id cannot be empty".to_string());
        }
        if self.function.name.trim().is_empty() {
            return Err("chat message tool call function name cannot be empty".to_string());
        }
        Ok(ToolCall::new(
            &self.id,
            &self.function.name,
            normalize_tool_arguments(&self.function.arguments)?,
        ))
    }
}

fn normalize_tool_arguments(raw: &Value) -> Result<Value, String> {
    match raw {
        Value::String(arguments) => {
            if arguments.trim().is_empty() {
                return Ok(serde_json::json!({}));
            }
            serde_json::from_str(arguments)
                .map_err(|err| format!("invalid tool call arguments JSON: {err}"))
        }
        other => Ok(other.clone()),
    }
}

fn validate_named_tool_choice(
    raw: &Value,
    tools: &[ToolDefinition],
    field: &str,
) -> Result<(), String> {
    let Some(name) = selected_tool_name(raw) else {
        return Ok(());
    };
    if tools.iter().any(|tool| tool.name == name) {
        return Ok(());
    }
    Err(format!("{field} references unknown tool: {name}"))
}

fn selected_tool_name(raw: &Value) -> Option<&str> {
    raw.get("function")
        .and_then(|function| function.get("name"))
        .and_then(Value::as_str)
        .or_else(|| raw.get("name").and_then(Value::as_str))
}

fn choice_counts(n: Option<usize>, best_of: Option<usize>) -> Result<ChoiceCounts, String> {
    let public = n.unwrap_or(1);
    let internal = best_of.unwrap_or(public);
    if public == 0 {
        return Err("n must be greater than zero".to_string());
    }
    if internal == 0 {
        return Err("best_of must be greater than zero".to_string());
    }
    if internal < public {
        return Err("best_of must be greater than or equal to n".to_string());
    }
    Ok(ChoiceCounts { public, internal })
}

fn parse_logit_bias(
    raw: &BTreeMap<String, f32>,
    bias_type: Option<&str>,
    mut tokenize: impl FnMut(&str) -> Result<Vec<LlamaToken>, String>,
) -> Result<Vec<LlamaLogitBias>, String> {
    match bias_type.unwrap_or("input_ids") {
        "input_ids" => raw
            .iter()
            .map(|(token, bias)| {
                let token = token
                    .parse::<i32>()
                    .map_err(|_| format!("invalid logit_bias token id: {token}"))?;
                Ok(LlamaLogitBias::new(token, *bias))
            })
            .collect(),
        "tokens" => {
            let mut biases = Vec::new();
            for (token, bias) in raw {
                for token_id in tokenize(token)? {
                    biases.push(LlamaLogitBias::new(token_id.raw(), *bias));
                }
            }
            Ok(biases)
        }
        other => Err(format!("unsupported logit_bias_type: {other}")),
    }
}

impl SamplingRequest {
    fn apply(&self, mut sampling: SamplingOptions) -> SamplingOptions {
        if let Some(value) = self.temperature {
            sampling.temperature = value;
        }
        if let Some(value) = self.top_k {
            sampling.top_k = value;
        }
        if let Some(value) = self.top_p {
            sampling.top_p = value;
        }
        if let Some(value) = self.tfs_z {
            sampling.tfs_z = value;
        }
        if let Some(value) = self.min_p {
            sampling.min_p = value;
        }
        if let Some(value) = self.typical_p {
            sampling.typical_p = value;
        }
        if let Some(value) = self.min_keep {
            sampling.min_keep = value;
        }
        if let Some(value) = self.penalty_last_n {
            sampling.penalty_last_n = value;
        }
        if let Some(value) = self.repeat_penalty {
            sampling.repeat_penalty = value;
        }
        if let Some(value) = self.frequency_penalty {
            sampling.frequency_penalty = value;
        }
        if let Some(value) = self.presence_penalty {
            sampling.presence_penalty = value;
        }
        if let Some(value) = self.mirostat_mode {
            sampling.mirostat_mode = value;
        }
        if let Some(value) = self.mirostat_tau {
            sampling.mirostat_tau = value;
        }
        if let Some(value) = self.mirostat_eta {
            sampling.mirostat_eta = value;
        }
        if let Some(value) = self.seed {
            sampling.seed = Some(value);
        }
        sampling
    }
}

fn build_request_sampler(
    llama: &Llama,
    options: &CompletionOptions,
    structured: &StructuredRequest,
) -> Result<LlamaSampler, String> {
    let base_sampler = options
        .build_sampler(llama)
        .map_err(|err| err.to_string())?;
    let Some(grammar) = structured.grammar_text()? else {
        return Ok(base_sampler);
    };
    let grammar_root = structured.grammar_root.as_deref().unwrap_or("root");
    let grammar_sampler = unsafe { LlamaSampler::grammar(llama.model(), &grammar, grammar_root) }
        .map_err(|err| err.to_string())?;
    LlamaSampler::chain(vec![grammar_sampler, base_sampler], false)
        .ok_or_else(|| "sampler_chain_init returned null".to_string())
}

impl StructuredRequest {
    fn grammar_text(&self) -> Result<Option<String>, String> {
        if let Some(grammar) = &self.grammar {
            if !grammar.trim().is_empty() {
                return Ok(Some(grammar.clone()));
            }
        }
        if let Some(schema) = &self.json_schema {
            return json_schema_grammar(schema)
                .map(Some)
                .map_err(|err| err.to_string());
        }
        let Some(response_format) = &self.response_format else {
            return Ok(None);
        };
        match response_format.kind.as_str() {
            "text" => Ok(None),
            "json_object" => response_format
                .schema
                .as_ref()
                .map(json_schema_grammar)
                .transpose()
                .map(|grammar| Some(grammar.unwrap_or_else(json_object_grammar)))
                .map_err(|err| err.to_string()),
            "json_schema" => {
                let Some(schema) = response_format
                    .json_schema
                    .as_ref()
                    .and_then(|json_schema| json_schema.schema.as_ref())
                    .or(response_format.schema.as_ref())
                else {
                    return Err("response_format json_schema requires a schema".to_string());
                };
                json_schema_grammar(schema)
                    .map(Some)
                    .map_err(|err| err.to_string())
            }
            other => Err(format!("unsupported response_format type: {other}")),
        }
    }
}

fn request_template(template: Option<&str>) -> BuiltinTemplate {
    template
        .and_then(BuiltinTemplate::from_str_ci)
        .unwrap_or(BuiltinTemplate::ChatMl)
}

fn stream_frame(
    id: &str,
    object: &'static str,
    created: u64,
    model: &str,
    chunk: CompletionChunk,
) -> StreamFrame {
    let logprobs = chunk.logprobs.map(|logprobs| {
        if object == "chat.completion.chunk" {
            StreamLogprobs::Chat(chat_logprobs_response(logprobs))
        } else {
            StreamLogprobs::Completion(completion_logprobs_response(logprobs))
        }
    });
    StreamFrame {
        id: id.to_string(),
        object,
        created,
        model: model.to_string(),
        choices: vec![StreamChoice {
            index: 0,
            text: if object == "chat.completion.chunk" {
                None
            } else {
                Some(chunk.text.clone())
            },
            delta: if object == "chat.completion.chunk" {
                if chunk.stop_reason.is_some() {
                    Some(ChatStreamDelta {
                        role: None,
                        content: None,
                    })
                } else {
                    Some(ChatStreamDelta {
                        role: None,
                        content: Some(chunk.text),
                    })
                }
            } else {
                None
            },
            logprobs,
            finish_reason: chunk.stop_reason.map(stop_reason),
        }],
    }
}

fn chat_stream_role_frame(id: &str, created: u64, model: &str) -> StreamFrame {
    StreamFrame {
        id: id.to_string(),
        object: "chat.completion.chunk",
        created,
        model: model.to_string(),
        choices: vec![StreamChoice {
            index: 0,
            text: None,
            delta: Some(ChatStreamDelta {
                role: Some("assistant"),
                content: None,
            }),
            logprobs: None,
            finish_reason: None,
        }],
    }
}

fn sse_response(rx: tokio_mpsc::UnboundedReceiver<Result<StreamFrame, String>>) -> Response {
    let stream = stream_events(rx).map(|event| match event {
        StreamEvent::Frame(frame) => Ok::<Event, Infallible>(
            Event::default()
                .json_data(frame)
                .unwrap_or_else(|err| Event::default().data(err.to_string())),
        ),
        StreamEvent::Error(err) => Ok(Event::default().event("error").data(err)),
        StreamEvent::Done => Ok(Event::default().data("[DONE]")),
    });
    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

fn stream_events(
    rx: tokio_mpsc::UnboundedReceiver<Result<StreamFrame, String>>,
) -> impl futures_util::Stream<Item = StreamEvent> {
    UnboundedReceiverStream::new(rx)
        .map(|frame| match frame {
            Ok(frame) => StreamEvent::Frame(frame),
            Err(err) => StreamEvent::Error(err),
        })
        .chain(stream::once(async { StreamEvent::Done }))
}

fn error_response(status: StatusCode, message: String) -> Response {
    (
        status,
        Json(ErrorBody {
            error: ErrorMessage {
                message,
                kind: "invalid_request",
            },
        }),
    )
        .into_response()
}

fn stop_reason(reason: llama_crab::StopReason) -> String {
    match reason {
        llama_crab::StopReason::Length => "length",
        llama_crab::StopReason::Eos | llama_crab::StopReason::Stop => "stop",
    }
    .to_string()
}

fn completion_logprobs_response(logprobs: CompletionLogprobs) -> CompletionLogprobsResponse {
    CompletionLogprobsResponse {
        tokens: logprobs.tokens,
        text_offset: logprobs.text_offset,
        token_logprobs: logprobs.token_logprobs,
        top_logprobs: logprobs
            .top_logprobs
            .into_iter()
            .map(|entry| {
                entry.map(|candidates| {
                    candidates
                        .into_iter()
                        .map(|candidate| (candidate.text, candidate.logprob))
                        .collect()
                })
            })
            .collect(),
    }
}

fn chat_logprobs_response(logprobs: CompletionLogprobs) -> ChatLogprobsResponse {
    let content = logprobs
        .tokens
        .into_iter()
        .zip(logprobs.token_logprobs)
        .zip(logprobs.top_logprobs)
        .map(|((token, logprob), top_logprobs)| ChatLogprobToken {
            token,
            logprob,
            bytes: None,
            top_logprobs: top_logprobs
                .unwrap_or_default()
                .into_iter()
                .map(|candidate| ChatTopLogprobToken {
                    token: candidate.text,
                    logprob: candidate.logprob,
                    bytes: None,
                })
                .collect(),
        })
        .collect();
    ChatLogprobsResponse {
        content: Some(content),
        refusal: None,
    }
}

fn chat_response_message(
    request: &ChatRequest,
    content: String,
) -> Result<ChatResponseMessage, String> {
    if request.tools.is_empty() {
        return Ok(ChatResponseMessage {
            role: "assistant",
            content: Some(content),
            tool_calls: Vec::new(),
        });
    }

    let format = ToolFormat::from_chat_format(request.template.as_deref().unwrap_or("chatml"));
    let tool_calls = extract_tool_calls(format, &content)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;

    if tool_calls.is_empty() {
        return Ok(ChatResponseMessage {
            role: "assistant",
            content: Some(content),
            tool_calls: Vec::new(),
        });
    }

    Ok(ChatResponseMessage {
        role: "assistant",
        content: None,
        tool_calls: tool_calls
            .into_iter()
            .map(chat_response_tool_call)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn chat_response_tool_call(call: ToolCall) -> Result<ChatResponseToolCall, String> {
    let arguments = serde_json::to_string(&call.arguments).map_err(|err| err.to_string())?;
    Ok(ChatResponseToolCall {
        id: call.id,
        kind: "function",
        function: ChatResponseToolCallFunction {
            name: call.name,
            arguments,
        },
    })
}

fn chat_finish_reason(reason: llama_crab::StopReason, message: &ChatResponseMessage) -> String {
    if !message.tool_calls.is_empty() {
        "tool_calls".to_string()
    } else {
        stop_reason(reason)
    }
}

fn default_normalize() -> bool {
    false
}

fn default_tool_parameters() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": {},
        "required": []
    })
}

fn deserialize_chat_content<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    chat_content_to_string(value.as_ref()).map_err(de::Error::custom)
}

fn chat_content_to_string(value: Option<&Value>) -> Result<String, String> {
    let Some(value) = value else {
        return Ok(String::new());
    };
    match value {
        Value::Null => Ok(String::new()),
        Value::String(content) => Ok(content.clone()),
        Value::Array(parts) => {
            let mut content = String::new();
            for part in parts {
                let kind = part.get("type").and_then(Value::as_str).unwrap_or("text");
                match kind {
                    "text" => {
                        if let Some(text) = part.get("text").and_then(Value::as_str) {
                            content.push_str(text);
                        }
                    }
                    other => return Err(format!("unsupported chat content part type: {other}")),
                }
            }
            Ok(content)
        }
        other => Err(format!("unsupported chat content value: {other}")),
    }
}

fn deserialize_stop_sequences<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    stop_sequences_from_value(value.as_ref()).map_err(de::Error::custom)
}

fn stop_sequences_from_value(value: Option<&Value>) -> Result<Vec<String>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    match value {
        Value::Null => Ok(Vec::new()),
        Value::String(stop) => Ok(vec![stop.clone()]),
        Value::Array(stops) => stops
            .iter()
            .map(|stop| {
                stop.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| format!("unsupported stop sequence value: {stop}"))
            })
            .collect(),
        other => Err(format!("unsupported stop sequence value: {other}")),
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sampling_request_overrides_defaults() {
        let request = SamplingRequest {
            temperature: Some(0.3),
            top_k: Some(12),
            tfs_z: Some(0.7),
            seed: Some(42),
            ..SamplingRequest::default()
        };

        let sampling = request.apply(SamplingOptions::default());

        assert_eq!(sampling.temperature, 0.3);
        assert_eq!(sampling.top_k, 12);
        assert_eq!(sampling.tfs_z, 0.7);
        assert_eq!(sampling.seed, Some(42));
        assert_eq!(sampling.top_p, SamplingOptions::default().top_p);
    }

    #[test]
    fn structured_request_prefers_grammar_text() {
        let request = StructuredRequest {
            grammar: Some("root ::= \"ok\"".to_string()),
            json_schema: Some(json!({"type": "object"})),
            response_format: None,
            grammar_root: None,
        };

        assert_eq!(
            request.grammar_text().unwrap(),
            Some("root ::= \"ok\"".to_string())
        );
    }

    #[test]
    fn structured_request_converts_json_schema() {
        let request = StructuredRequest {
            grammar: None,
            json_schema: Some(json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                },
                "required": ["name"]
            })),
            response_format: None,
            grammar_root: None,
        };

        let grammar = request.grammar_text().unwrap().unwrap();

        assert!(grammar.contains("root"));
        assert!(grammar.contains("name"));
    }

    #[test]
    fn structured_request_accepts_json_object_response_format() {
        let request = StructuredRequest {
            grammar: None,
            json_schema: None,
            response_format: Some(ResponseFormatRequest {
                kind: "json_object".to_string(),
                schema: None,
                json_schema: None,
            }),
            grammar_root: None,
        };

        let grammar = request.grammar_text().unwrap().unwrap();

        assert!(grammar.contains("root ::= object"));
        assert!(grammar.contains("kv ::="));
    }

    #[test]
    fn structured_request_accepts_schema_response_format() {
        let request = StructuredRequest {
            grammar: None,
            json_schema: None,
            response_format: Some(ResponseFormatRequest {
                kind: "json_object".to_string(),
                schema: Some(json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" }
                    },
                    "required": ["name"]
                })),
                json_schema: None,
            }),
            grammar_root: None,
        };

        let grammar = request.grammar_text().unwrap().unwrap();

        assert!(grammar.contains("root"));
        assert!(grammar.contains("name"));
    }

    #[test]
    fn structured_request_accepts_json_schema_response_format() {
        let request: StructuredRequest = serde_json::from_value(json!({
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "person",
                    "schema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" }
                        },
                        "required": ["name"]
                    }
                }
            }
        }))
        .unwrap();

        let grammar = request.grammar_text().unwrap().unwrap();

        assert!(grammar.contains("root"));
        assert!(grammar.contains("name"));
    }

    #[test]
    fn completion_request_applies_echo_and_suffix_options() {
        let request = CompletionRequest {
            model: None,
            prompt: CompletionPrompt::Single("Question:".to_string()),
            user: None,
            max_tokens: Some(8),
            min_tokens: Some(3),
            logprobs: Some(2),
            n: Some(1),
            best_of: Some(1),
            stop: Vec::new(),
            stream: false,
            echo: true,
            suffix: Some("\nDone".to_string()),
            logit_bias: BTreeMap::new(),
            logit_bias_type: None,
            sampling: SamplingRequest::default(),
            structured: StructuredRequest::default(),
        };

        let options = request
            .completion_options(SamplingOptions::default(), |_| unreachable!())
            .unwrap();

        assert!(options.echo_prompt);
        assert_eq!(options.suffix.as_deref(), Some("\nDone"));
        assert_eq!(options.min_tokens, 3);
        assert_eq!(options.logprobs, Some(2));
    }

    #[test]
    fn completion_request_reports_public_and_internal_choice_counts() {
        let request = CompletionRequest {
            model: None,
            prompt: CompletionPrompt::Single("Question:".to_string()),
            user: None,
            max_tokens: Some(8),
            min_tokens: None,
            logprobs: None,
            n: Some(2),
            best_of: Some(4),
            stop: Vec::new(),
            stream: false,
            echo: false,
            suffix: None,
            logit_bias: BTreeMap::new(),
            logit_bias_type: None,
            sampling: SamplingRequest::default(),
            structured: StructuredRequest::default(),
        };

        let counts = request.choice_counts().unwrap();

        assert_eq!(counts.public, 2);
        assert_eq!(counts.internal, 4);
    }

    #[test]
    fn completion_best_of_selects_highest_average_token_logprob() {
        let low_score = llama_crab::Completion {
            text: "low".to_string(),
            n_tokens: 2,
            stop_reason: llama_crab::StopReason::Length,
            logprobs: Some(CompletionLogprobs {
                tokens: vec!["l".to_string(), "ow".to_string()],
                text_offset: vec![0, 1],
                token_logprobs: vec![Some(-3.0), Some(-3.0)],
                top_logprobs: vec![None, None],
            }),
        };
        let high_score = llama_crab::Completion {
            text: "high".to_string(),
            n_tokens: 2,
            stop_reason: llama_crab::StopReason::Length,
            logprobs: Some(CompletionLogprobs {
                tokens: vec!["hi".to_string(), "gh".to_string()],
                text_offset: vec![0, 2],
                token_logprobs: vec![Some(-0.5), Some(-1.0)],
                top_logprobs: vec![None, None],
            }),
        };

        let choices = select_completion_choices(vec![low_score, high_score], 1);

        assert_eq!(choices[0].text, "high");
    }

    #[test]
    fn completion_logprobs_response_serializes_top_candidates_as_maps() {
        let response = completion_logprobs_response(CompletionLogprobs {
            tokens: vec!["ok".to_string()],
            text_offset: vec![3],
            token_logprobs: vec![Some(-0.25)],
            top_logprobs: vec![Some(vec![llama_crab::TokenLogprob {
                token: 42,
                text: "ok".to_string(),
                logprob: -0.25,
            }])],
        });

        assert_eq!(response.tokens, vec!["ok"]);
        assert_eq!(response.text_offset, vec![3]);
        assert_eq!(response.token_logprobs, vec![Some(-0.25)]);
        assert_eq!(
            response.top_logprobs[0]
                .as_ref()
                .and_then(|entry| entry.get("ok"))
                .copied(),
            Some(-0.25)
        );
    }

    #[test]
    fn chat_logprobs_response_uses_chat_content_shape() {
        let response = chat_logprobs_response(CompletionLogprobs {
            tokens: vec!["ok".to_string()],
            text_offset: vec![3],
            token_logprobs: vec![Some(-0.25)],
            top_logprobs: vec![Some(vec![llama_crab::TokenLogprob {
                token: 42,
                text: "ok".to_string(),
                logprob: -0.25,
            }])],
        });

        let content = response.content.unwrap();
        assert_eq!(content[0].token, "ok");
        assert_eq!(content[0].logprob, Some(-0.25));
        assert_eq!(content[0].bytes, None);
        assert_eq!(content[0].top_logprobs[0].token, "ok");
        assert_eq!(content[0].top_logprobs[0].logprob, -0.25);
        assert!(response.refusal.is_none());
    }

    #[test]
    fn chat_tool_call_response_uses_structured_message_shape() {
        let request: ChatRequest = serde_json::from_value(json!({
            "messages": [{"role": "user", "content": "Weather in Tokyo?"}],
            "template": "chatml",
            "tools": [{
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get weather",
                    "parameters": {
                        "type": "object",
                        "properties": {"city": {"type": "string"}},
                        "required": ["city"]
                    }
                }
            }]
        }))
        .unwrap();

        let message = chat_response_message(
            &request,
            r#"<tool_call>{"name":"get_weather","arguments":{"city":"Tokyo"}}</tool_call>"#
                .to_string(),
        )
        .unwrap();
        let finish_reason = chat_finish_reason(llama_crab::StopReason::Stop, &message);

        let value = serde_json::to_value(ChatChoice {
            index: 0,
            message,
            logprobs: None,
            finish_reason: Some(finish_reason),
        })
        .unwrap();

        assert_eq!(value["message"]["role"], "assistant");
        assert!(value["message"]["content"].is_null());
        assert_eq!(value["message"]["tool_calls"][0]["type"], "function");
        assert_eq!(
            value["message"]["tool_calls"][0]["function"]["name"],
            "get_weather"
        );
        assert_eq!(
            value["message"]["tool_calls"][0]["function"]["arguments"],
            "{\"city\":\"Tokyo\"}"
        );
        assert_eq!(value["finish_reason"], "tool_calls");
    }

    #[test]
    fn text_stream_frame_includes_completion_logprobs() {
        let frame = stream_frame(
            "cmpl-test",
            "text_completion",
            7,
            "model",
            CompletionChunk {
                text: "ok".to_string(),
                token: Some(LlamaToken::from(42)),
                n_tokens: 1,
                stop_reason: None,
                logprobs: Some(CompletionLogprobs {
                    tokens: vec!["ok".to_string()],
                    text_offset: vec![3],
                    token_logprobs: vec![Some(-0.25)],
                    top_logprobs: vec![Some(vec![llama_crab::TokenLogprob {
                        token: 42,
                        text: "ok".to_string(),
                        logprob: -0.25,
                    }])],
                }),
            },
        );

        let Some(StreamLogprobs::Completion(logprobs)) = &frame.choices[0].logprobs else {
            panic!("expected completion logprobs");
        };
        assert_eq!(logprobs.tokens, vec!["ok"]);
        assert_eq!(logprobs.token_logprobs, vec![Some(-0.25)]);
    }

    #[test]
    fn chat_stream_frame_converts_logprobs_to_chat_shape() {
        let frame = stream_frame(
            "chatcmpl-test",
            "chat.completion.chunk",
            7,
            "model",
            CompletionChunk {
                text: "ok".to_string(),
                token: Some(LlamaToken::from(42)),
                n_tokens: 1,
                stop_reason: None,
                logprobs: Some(CompletionLogprobs {
                    tokens: vec!["ok".to_string()],
                    text_offset: vec![3],
                    token_logprobs: vec![Some(-0.25)],
                    top_logprobs: vec![Some(vec![llama_crab::TokenLogprob {
                        token: 42,
                        text: "ok".to_string(),
                        logprob: -0.25,
                    }])],
                }),
            },
        );

        let Some(StreamLogprobs::Chat(logprobs)) = &frame.choices[0].logprobs else {
            panic!("expected chat logprobs");
        };
        let value = serde_json::to_value(&frame).unwrap();
        assert!(value["choices"][0]["delta"].get("role").is_none());
        assert_eq!(value["choices"][0]["delta"]["content"], "ok");
        assert!(value["choices"][0].get("text").is_none());
        let content = logprobs.content.as_ref().unwrap();
        assert_eq!(content[0].token, "ok");
        assert_eq!(content[0].logprob, Some(-0.25));
    }

    #[test]
    fn chat_stream_terminal_frame_has_empty_delta_and_finish_reason() {
        let frame = stream_frame(
            "chatcmpl-test",
            "chat.completion.chunk",
            7,
            "model",
            CompletionChunk {
                text: String::new(),
                token: None,
                n_tokens: 1,
                stop_reason: Some(llama_crab::StopReason::Stop),
                logprobs: None,
            },
        );

        let value = serde_json::to_value(&frame).unwrap();
        assert_eq!(value["choices"][0]["delta"], serde_json::json!({}));
        assert_eq!(value["choices"][0]["finish_reason"], "stop");
    }

    #[test]
    fn chat_stream_initial_frame_announces_assistant_role() {
        let frame = chat_stream_role_frame("chatcmpl-test", 7, "model");

        let value = serde_json::to_value(&frame).unwrap();
        assert_eq!(value["id"], "chatcmpl-test");
        assert_eq!(value["object"], "chat.completion.chunk");
        assert_eq!(
            value["choices"][0]["delta"],
            serde_json::json!({"role": "assistant"})
        );
        assert!(value["choices"][0]["finish_reason"].is_null());
    }

    #[tokio::test]
    async fn stream_events_end_with_done() {
        let (tx, rx) = tokio_mpsc::unbounded_channel();
        tx.send(Ok(stream_frame(
            "cmpl-test",
            "text_completion",
            7,
            "model",
            CompletionChunk {
                text: "ok".to_string(),
                token: Some(LlamaToken::from(42)),
                n_tokens: 1,
                stop_reason: None,
                logprobs: None,
            },
        )))
        .unwrap();
        drop(tx);

        let events = stream_events(rx).collect::<Vec<_>>().await;

        assert!(matches!(events.first(), Some(StreamEvent::Frame(_))));
        assert!(matches!(events.last(), Some(StreamEvent::Done)));
    }

    #[test]
    fn chat_request_applies_logprobs_when_enabled() {
        let request = ChatRequest {
            model: None,
            messages: vec![ChatRequestMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                tool_call_id: None,
                tool_calls: Vec::new(),
                name: None,
            }],
            user: None,
            max_tokens: Some(8),
            min_tokens: Some(1),
            logprobs: Some(true),
            top_logprobs: Some(3),
            n: Some(1),
            stream: false,
            template: None,
            stop: Vec::new(),
            tools: Vec::new(),
            tool_choice: None,
            function_call: None,
            logit_bias: BTreeMap::new(),
            logit_bias_type: None,
            sampling: SamplingRequest::default(),
            structured: StructuredRequest::default(),
        };

        let options = request
            .completion_options(SamplingOptions::chat(), |_| unreachable!())
            .unwrap();

        assert_eq!(options.min_tokens, 1);
        assert_eq!(options.logprobs, Some(3));
    }

    #[test]
    fn chat_request_converts_function_tools_for_prompt_rendering() {
        let request: ChatRequest = serde_json::from_value(json!({
            "messages": [
                {"role": "user", "content": "Weather in Tokyo?"}
            ],
            "template": "chatml",
            "tools": [{
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get weather for a city",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "city": {"type": "string"}
                        },
                        "required": ["city"]
                    }
                }
            }]
        }))
        .unwrap();

        let tools = request.chat_tools().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "get_weather");
        assert_eq!(tools[0].description, "Get weather for a city");
        assert_eq!(tools[0].parameters["properties"]["city"]["type"], "string");

        let prompt = request.chat_prompt().unwrap();
        assert!(prompt.contains("get_weather"));
        assert!(prompt.contains("Weather in Tokyo?"));
    }

    #[test]
    fn chat_request_rejects_unknown_tool_choice() {
        let request: ChatRequest = serde_json::from_value(json!({
            "messages": [
                {"role": "user", "content": "Weather in Tokyo?"}
            ],
            "tools": [{
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get weather for a city"
                }
            }],
            "tool_choice": {
                "type": "function",
                "function": {"name": "get_time"}
            }
        }))
        .unwrap();

        let err = request.chat_prompt().unwrap_err();
        assert!(err.contains("tool_choice references unknown tool: get_time"));
    }

    #[test]
    fn chat_request_rejects_unknown_role_in_prompt() {
        let request: ChatRequest = serde_json::from_value(json!({
            "messages": [
                {"role": "user", "content": "hi"},
                {"role": "wizard", "content": "I cast a spell"}
            ]
        }))
        .unwrap();

        let err = request.chat_prompt().unwrap_err();
        assert!(err.contains("unknown chat role: wizard"));
    }

    #[test]
    fn chat_request_preserves_tool_call_messages() {
        let request: ChatRequest = serde_json::from_value(json!({
            "messages": [
                {"role": "user", "content": "Weather in Tokyo?"},
                {
                    "role": "assistant",
                    "tool_calls": [{
                        "id": "call_weather",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"city\":\"Tokyo\"}"
                        }
                    }]
                },
                {
                    "role": "tool",
                    "tool_call_id": "call_weather",
                    "content": "{\"temperature\":22}"
                }
            ],
            "template": "chatml"
        }))
        .unwrap();

        let messages = convert_messages(request.messages.clone()).unwrap();
        assert_eq!(messages[1].tool_calls.len(), 1);
        assert_eq!(messages[1].tool_calls[0].id, "call_weather");
        assert_eq!(messages[1].tool_calls[0].name, "get_weather");
        assert_eq!(messages[1].tool_calls[0].arguments["city"], "Tokyo");
        assert_eq!(messages[2].tool_call_id.as_deref(), Some("call_weather"));

        let prompt = request.chat_prompt().unwrap();
        assert!(prompt.contains("<tool_call>"));
        assert!(prompt.contains("get_weather"));
        assert!(prompt.contains("\"city\":\"Tokyo\""));
        assert!(prompt.contains("{\"temperature\":22}"));
    }

    #[test]
    fn tokenize_request_accepts_model_and_input() {
        let request: TokenizeRequest = serde_json::from_value(json!({
            "model": "local-model",
            "input": "How many tokens?"
        }))
        .unwrap();

        assert_eq!(request.model.as_deref(), Some("local-model"));
        assert_eq!(request.input, "How many tokens?");
    }

    #[test]
    fn detokenize_response_serializes_text() {
        let response = DetokenizeResponse {
            text: "How many tokens?".to_string(),
        };

        let value = serde_json::to_value(response).unwrap();
        assert_eq!(value, json!({"text": "How many tokens?"}));
    }

    #[test]
    fn model_info_serializes_permissions() {
        let response = ModelList {
            object: "list",
            data: vec![ModelInfo {
                id: "local-model".to_string(),
                object: "model",
                created: 7,
                owned_by: "me",
                permissions: Vec::new(),
            }],
        };

        let value = serde_json::to_value(response).unwrap();
        assert_eq!(value["data"][0]["owned_by"], "me");
        assert_eq!(value["data"][0]["permissions"], json!([]));
    }

    #[test]
    fn completion_request_accepts_prompt_list() {
        let request: CompletionRequest = serde_json::from_value(json!({
            "model": "configured-model",
            "user": "client-user",
            "prompt": ["First prompt", "Second prompt"],
            "max_tokens": 1
        }))
        .unwrap();

        assert_eq!(request.model.as_deref(), Some("configured-model"));
        assert_eq!(request.user.as_deref(), Some("client-user"));
        assert_eq!(request.prompts(), vec!["First prompt", "Second prompt"]);
    }

    #[test]
    fn chat_request_accepts_model_and_user() {
        let request: ChatRequest = serde_json::from_value(json!({
            "model": "configured-model",
            "user": "client-user",
            "messages": [{"role": "user", "content": "Hello"}]
        }))
        .unwrap();

        assert_eq!(request.model.as_deref(), Some("configured-model"));
        assert_eq!(request.user.as_deref(), Some("client-user"));
    }

    #[test]
    fn embedding_request_accepts_model_and_user() {
        let request: EmbeddingRequest = serde_json::from_value(json!({
            "model": "configured-model",
            "user": "client-user",
            "input": "Embed this"
        }))
        .unwrap();

        assert_eq!(request.model.as_deref(), Some("configured-model"));
        assert_eq!(request.user.as_deref(), Some("client-user"));
    }

    #[test]
    fn completion_request_accepts_single_stop_string() {
        let request: CompletionRequest = serde_json::from_value(json!({
            "prompt": "Stop test",
            "stop": "\n"
        }))
        .unwrap();

        assert_eq!(request.stop, vec!["\n"]);
    }

    #[test]
    fn chat_request_accepts_single_stop_string() {
        let request: ChatRequest = serde_json::from_value(json!({
            "messages": [{"role": "user", "content": "Stop test"}],
            "stop": "</s>"
        }))
        .unwrap();

        assert_eq!(request.stop, vec!["</s>"]);
    }

    #[test]
    fn completion_request_applies_logit_bias_by_token_id() {
        let request = CompletionRequest {
            model: None,
            prompt: CompletionPrompt::Single("Question:".to_string()),
            user: None,
            max_tokens: Some(8),
            min_tokens: None,
            logprobs: None,
            n: Some(1),
            best_of: Some(1),
            stop: Vec::new(),
            stream: false,
            echo: false,
            suffix: None,
            logit_bias: std::collections::BTreeMap::from([("42".to_string(), -100.0)]),
            logit_bias_type: None,
            sampling: SamplingRequest::default(),
            structured: StructuredRequest::default(),
        };

        let options = request
            .completion_options(SamplingOptions::default(), |_| unreachable!())
            .unwrap();

        assert_eq!(options.logit_bias.len(), 1);
        assert_eq!(options.logit_bias[0].token, 42);
        assert_eq!(options.logit_bias[0].bias, -100.0);
    }

    #[test]
    fn completion_request_applies_logit_bias_by_token_text() {
        let request = CompletionRequest {
            model: None,
            prompt: CompletionPrompt::Single("Question:".to_string()),
            user: None,
            max_tokens: Some(8),
            min_tokens: None,
            logprobs: None,
            n: Some(1),
            best_of: Some(1),
            stop: Vec::new(),
            stream: false,
            echo: false,
            suffix: None,
            logit_bias: BTreeMap::from([("hello".to_string(), 2.5)]),
            logit_bias_type: Some("tokens".to_string()),
            sampling: SamplingRequest::default(),
            structured: StructuredRequest::default(),
        };

        let options = request
            .completion_options(SamplingOptions::default(), |text| {
                assert_eq!(text, "hello");
                Ok(vec![LlamaToken::from(7), LlamaToken::from(8)])
            })
            .unwrap();

        assert_eq!(options.logit_bias.len(), 2);
        assert_eq!(options.logit_bias[0].token, 7);
        assert_eq!(options.logit_bias[0].bias, 2.5);
        assert_eq!(options.logit_bias[1].token, 8);
        assert_eq!(options.logit_bias[1].bias, 2.5);
    }

    #[test]
    fn completion_request_rejects_invalid_logit_bias_token_id() {
        let request = CompletionRequest {
            model: None,
            prompt: CompletionPrompt::Single("Question:".to_string()),
            user: None,
            max_tokens: Some(8),
            min_tokens: None,
            logprobs: None,
            n: Some(1),
            best_of: Some(1),
            stop: Vec::new(),
            stream: false,
            echo: false,
            suffix: None,
            logit_bias: BTreeMap::from([("bad".to_string(), 1.0)]),
            logit_bias_type: None,
            sampling: SamplingRequest::default(),
            structured: StructuredRequest::default(),
        };

        let err = request
            .completion_options(SamplingOptions::default(), |_| unreachable!())
            .unwrap_err();

        assert!(err.contains("invalid logit_bias token id"));
    }
}
