use std::path::PathBuf;

use llama_crab::{
    chat::{BuiltinTemplate, ToolDefinition},
    context::params::PoolingType,
    high_level::completion::{CompletionLogprobs, CompletionOptions, SamplingOptions},
    json_schema::schema_to_grammar,
    ChatMessage as LlamaChat, LlamaParams, MobilePreset, Role,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{PluginError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MobilePresetName {
    LowRam,
    Balanced,
    GpuMax,
}

impl MobilePresetName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LowRam => "low-ram",
            Self::Balanced => "balanced",
            Self::GpuMax => "gpu-max",
        }
    }

    const fn to_llama(self) -> MobilePreset {
        match self {
            Self::LowRam => MobilePreset::LowRam,
            Self::Balanced => MobilePreset::Balanced,
            Self::GpuMax => MobilePreset::GpuMax,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelKind {
    Chat,
    Completion,
    Embedding,
    Rerank,
    Multimodal,
}

impl Default for ModelKind {
    fn default() -> Self {
        Self::Chat
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PoolingName {
    Unspecified,
    None,
    Mean,
    Cls,
    Last,
    Rank,
}

impl PoolingName {
    const fn to_llama(self) -> PoolingType {
        match self {
            Self::Unspecified => PoolingType::Unspecified,
            Self::None => PoolingType::None,
            Self::Mean => PoolingType::Mean,
            Self::Cls => PoolingType::Cls,
            Self::Last => PoolingType::Last,
            Self::Rank => PoolingType::Rank,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unspecified => "unspecified",
            Self::None => "none",
            Self::Mean => "mean",
            Self::Cls => "cls",
            Self::Last => "last",
            Self::Rank => "rank",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadModelRequest {
    #[serde(default)]
    pub id: Option<String>,
    pub path: String,
    #[serde(default)]
    pub kind: Option<ModelKind>,
    #[serde(default)]
    pub mobile_preset: Option<MobilePresetName>,
    #[serde(default)]
    pub pooling: Option<PoolingName>,
    #[serde(default)]
    pub embeddings: Option<bool>,
    #[serde(default)]
    pub mmproj_path: Option<String>,
    #[serde(default)]
    pub n_ctx: Option<u32>,
    #[serde(default)]
    pub n_batch: Option<u32>,
    #[serde(default)]
    pub n_ubatch: Option<u32>,
    #[serde(default)]
    pub n_gpu_layers: Option<i32>,
    #[serde(default)]
    pub n_threads: Option<i32>,
    #[serde(default)]
    pub n_threads_batch: Option<i32>,
    #[serde(default)]
    pub use_mmap: Option<bool>,
    #[serde(default)]
    pub flash_attn: Option<bool>,
    #[serde(default)]
    pub offload_kqv: Option<bool>,
}

impl LoadModelRequest {
    pub fn llama_params(&self) -> LlamaParams {
        let mut params = LlamaParams::new(PathBuf::from(&self.path));
        if let Some(preset) = self.mobile_preset {
            params = params.with_mobile_preset(preset.to_llama());
        }
        if let Some(n_ctx) = self.n_ctx {
            params = params.with_n_ctx(n_ctx);
        }
        if let Some(n_batch) = self.n_batch {
            params = params.with_n_batch(n_batch);
        }
        if let Some(n_ubatch) = self.n_ubatch {
            params = params.with_n_ubatch(n_ubatch);
        }
        if let Some(n_gpu_layers) = self.n_gpu_layers {
            params = params.with_n_gpu_layers(n_gpu_layers);
        }
        if let Some(n_threads) = self.n_threads {
            params = params.with_n_threads(n_threads);
        }
        if let Some(n_threads_batch) = self.n_threads_batch {
            params = params.with_n_threads_batch(n_threads_batch);
        }
        if let Some(use_mmap) = self.use_mmap {
            params = params.with_use_mmap(use_mmap);
        }
        if let Some(flash_attn) = self.flash_attn {
            params = params.with_flash_attn(flash_attn);
        }
        if let Some(offload_kqv) = self.offload_kqv {
            params = params.with_offload_kqv(offload_kqv);
        }
        if let Some(embeddings) = self.embeddings {
            params = params.with_embeddings(embeddings);
        } else if matches!(self.kind, Some(ModelKind::Embedding | ModelKind::Rerank)) {
            params = params.with_embeddings(true);
        }
        if let Some(pooling) = self.pooling {
            params = params.with_pooling_type(pooling.to_llama());
        } else if matches!(self.kind, Some(ModelKind::Rerank)) {
            params = params.with_pooling_type(PoolingType::Rank);
        }
        params
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadModelResponse {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub owned_by: &'static str,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ModelKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile_preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pooling: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mmproj_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedModelInfo {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub owned_by: &'static str,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<ModelKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobile_preset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pooling: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mmproj_path: Option<String>,
}

impl LoadedModelInfo {
    pub fn new(
        id: String,
        path: String,
        kind: Option<ModelKind>,
        mobile_preset: Option<String>,
        pooling: Option<String>,
        mmproj_path: Option<String>,
    ) -> Self {
        Self {
            id,
            object: "model",
            created: unix_timestamp(),
            owned_by: "llama-crab",
            path,
            kind,
            mobile_preset,
            pooling,
            mmproj_path,
        }
    }

    pub fn load_response(&self) -> LoadModelResponse {
        LoadModelResponse {
            id: self.id.clone(),
            object: "model",
            created: self.created,
            owned_by: "llama-crab",
            path: self.path.clone(),
            kind: self.kind,
            mobile_preset: self.mobile_preset.clone(),
            pooling: self.pooling.clone(),
            mmproj_path: self.mmproj_path.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelListResponse {
    pub object: &'static str,
    pub data: Vec<LoadedModelInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingRequest {
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_k: Option<i32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub tfs_z: Option<f32>,
    #[serde(default)]
    pub min_p: Option<f32>,
    #[serde(default)]
    pub typical_p: Option<f32>,
    #[serde(default)]
    pub min_keep: Option<usize>,
    #[serde(default)]
    pub penalty_last_n: Option<i32>,
    #[serde(default)]
    pub repeat_penalty: Option<f32>,
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    #[serde(default)]
    pub presence_penalty: Option<f32>,
    #[serde(default)]
    pub mirostat_mode: Option<i32>,
    #[serde(default)]
    pub mirostat_tau: Option<f32>,
    #[serde(default)]
    pub mirostat_eta: Option<f32>,
    #[serde(default)]
    pub seed: Option<u32>,
}

impl SamplingRequest {
    fn apply_to(&self, mut sampling: SamplingOptions) -> SamplingOptions {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredRequest {
    #[serde(default)]
    pub grammar: Option<String>,
    #[serde(default)]
    pub grammar_root: Option<String>,
    #[serde(default)]
    pub json_schema: Option<Value>,
    #[serde(default)]
    pub response_format: Option<ResponseFormatRequest>,
}

impl StructuredRequest {
    pub fn grammar_text(&self) -> Result<Option<(String, String)>> {
        let root = self.grammar_root.clone().unwrap_or_else(|| "root".into());
        if let Some(grammar) = &self.grammar {
            return Ok(Some((grammar.clone(), root)));
        }
        if let Some(schema) = &self.json_schema {
            return schema_to_grammar(schema, &root)
                .map(|grammar| Some((grammar, root)))
                .map_err(PluginError::from);
        }
        let Some(response_format) = &self.response_format else {
            return Ok(None);
        };
        if response_format.kind == "json_object" {
            return Ok(Some((json_object_grammar(), root)));
        }
        let schema = response_format.schema.as_ref().or(response_format
            .json_schema
            .as_ref()
            .and_then(|value| value.schema.as_ref()));
        if let Some(schema) = schema {
            return schema_to_grammar(schema, &root)
                .map(|grammar| Some((grammar, root)))
                .map_err(PluginError::from);
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseFormatRequest {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub schema: Option<Value>,
    #[serde(default)]
    pub json_schema: Option<ResponseFormatJsonSchema>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseFormatJsonSchema {
    #[serde(default)]
    pub schema: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub min_tokens: Option<usize>,
    #[serde(default)]
    pub logprobs: Option<bool>,
    #[serde(default)]
    pub top_logprobs: Option<usize>,
    #[serde(default)]
    pub n: Option<usize>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub stop: Vec<String>,
    #[serde(default)]
    pub tools: Vec<ChatToolRequest>,
    #[serde(default)]
    pub tool_choice: Option<Value>,
    #[serde(flatten)]
    pub sampling: SamplingRequest,
    #[serde(flatten)]
    pub structured: StructuredRequest,
}

impl ChatCompletionRequest {
    pub fn completion_options(&self) -> CompletionOptions {
        completion_options(
            self.max_tokens.unwrap_or(128),
            self.min_tokens,
            self.logprobs,
            self.top_logprobs,
            &self.stop,
            self.sampling.apply_to(SamplingOptions::chat()),
            false,
            None,
        )
    }

    pub fn choice_count(&self) -> usize {
        self.n.unwrap_or(1).max(1)
    }

    pub fn template(&self) -> Result<BuiltinTemplate> {
        parse_template(self.template.as_deref())
    }

    pub fn llama_messages(&self) -> Result<Vec<LlamaChat>> {
        self.messages
            .iter()
            .map(ChatMessage::to_llama_message)
            .collect()
    }

    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .iter()
            .filter(|tool| tool.kind == "function")
            .map(|tool| {
                ToolDefinition::new(&tool.function.name, &tool.function.description)
                    .with_parameters(tool.function.parameters.clone())
            })
            .collect()
    }

    pub fn has_media(&self) -> bool {
        self.messages.iter().any(ChatMessage::has_media)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: PromptInput,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub min_tokens: Option<usize>,
    #[serde(default)]
    pub logprobs: Option<usize>,
    #[serde(default)]
    pub n: Option<usize>,
    #[serde(default)]
    pub echo: bool,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(default)]
    pub stop: Vec<String>,
    #[serde(flatten)]
    pub sampling: SamplingRequest,
    #[serde(flatten)]
    pub structured: StructuredRequest,
}

impl CompletionRequest {
    pub fn prompts(&self) -> Vec<String> {
        match &self.prompt {
            PromptInput::Single(value) => vec![value.clone()],
            PromptInput::Many(values) => values.clone(),
        }
    }

    pub fn completion_options(&self) -> CompletionOptions {
        completion_options(
            self.max_tokens.unwrap_or(16),
            self.min_tokens,
            self.logprobs.map(|_| true),
            self.logprobs,
            &self.stop,
            self.sampling.apply_to(SamplingOptions::default()),
            self.echo,
            self.suffix.clone(),
        )
    }

    pub fn choice_count(&self) -> usize {
        self.n.unwrap_or(1).max(1)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PromptInput {
    Single(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    #[serde(default)]
    pub content: ChatContent,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub tool_calls: Vec<ChatMessageToolCallRequest>,
    #[serde(default)]
    pub name: Option<String>,
}

impl ChatMessage {
    pub fn to_llama_message(&self) -> Result<LlamaChat> {
        if self.has_media() {
            return Err(PluginError::invalid_request(
                "multimodal chat requires a plugin build with mtmd support",
            ));
        }
        let role = match self.role.as_str() {
            "developer" => Role::System,
            other => other
                .parse::<Role>()
                .map_err(PluginError::invalid_request)?,
        };
        Ok(LlamaChat::new(role, self.content.text()))
    }

    fn has_media(&self) -> bool {
        self.content.parts.iter().any(|part| {
            matches!(
                part,
                ChatContentPart::ImageUrl { .. } | ChatContentPart::InputAudio { .. }
            )
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChatContent {
    #[serde(deserialize_with = "deserialize_chat_parts")]
    pub parts: Vec<ChatContentPart>,
}

impl ChatContent {
    fn text(&self) -> String {
        self.parts
            .iter()
            .filter_map(|part| match part {
                ChatContentPart::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrlPart },
    InputAudio { input_audio: InputAudioPart },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageUrlPart {
    pub url: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputAudioPart {
    pub data: String,
    pub format: String,
}

fn deserialize_chat_parts<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<ChatContentPart>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(Vec::new());
    }
    if let Some(text) = value.as_str() {
        return Ok(vec![ChatContentPart::Text { text: text.into() }]);
    }
    serde_json::from_value(value).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatToolRequest {
    #[serde(rename = "type")]
    pub kind: String,
    pub function: ChatFunctionToolRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatFunctionToolRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_tool_parameters")]
    pub parameters: Value,
}

fn default_tool_parameters() -> Value {
    serde_json::json!({ "type": "object", "properties": {}, "required": [] })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessageToolCallRequest {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: ChatMessageToolCallFunctionRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessageToolCallFunctionRequest {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatResponseMessage,
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<CompletionLogprobsResponse>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatResponseMessage {
    pub role: &'static str,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ChatResponseToolCall>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatResponseToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub function: ChatResponseToolCallFunction,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatResponseToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChunkChoice>,
    pub usage: Option<Usage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatChunkChoice {
    pub index: u32,
    pub delta: ChatChunkDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct ChatChunkDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ChatStreamToolCall>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatStreamToolCall {
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<&'static str>,
    pub function: ChatStreamToolCallFunction,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ChatStreamToolCallFunction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub model: String,
    pub choices: Vec<CompletionChoice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompletionChoice {
    pub text: String,
    pub index: u32,
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<CompletionLogprobsResponse>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompletionChunkFrame {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub model: String,
    pub choices: Vec<CompletionChunkChoice>,
    pub usage: Option<Usage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompletionChunkChoice {
    pub text: String,
    pub index: u32,
    pub finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<CompletionLogprobsResponse>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompletionLogprobsResponse {
    pub tokens: Vec<String>,
    pub text_offset: Vec<usize>,
    pub token_logprobs: Vec<Option<f32>>,
    pub top_logprobs: Vec<Option<Vec<TokenTopLogprob>>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TokenTopLogprob {
    pub token: String,
    pub logprob: f32,
}

impl From<CompletionLogprobs> for CompletionLogprobsResponse {
    fn from(value: CompletionLogprobs) -> Self {
        Self {
            tokens: value.tokens,
            text_offset: value.text_offset,
            token_logprobs: value.token_logprobs,
            top_logprobs: value
                .top_logprobs
                .into_iter()
                .map(|items| {
                    items.map(|items| {
                        items
                            .into_iter()
                            .map(|item| TokenTopLogprob {
                                token: item.text,
                                logprob: item.logprob,
                            })
                            .collect()
                    })
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    Single(String),
    Many(Vec<String>),
}

impl EmbeddingInput {
    pub fn texts(&self) -> Vec<String> {
        match self {
            Self::Single(text) => vec![text.clone()],
            Self::Many(texts) => texts.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: EmbeddingInput,
    #[serde(default = "default_normalize")]
    pub normalize: bool,
    #[serde(default)]
    pub encoding_format: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
}

fn default_normalize() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EmbeddingResponse {
    pub object: &'static str,
    pub data: Vec<EmbeddingItem>,
    pub model: String,
    pub usage: EmbeddingUsage,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EmbeddingItem {
    pub object: &'static str,
    pub embedding: EmbeddingValue,
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum EmbeddingValue {
    Float(Vec<f32>),
    Base64(String),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EmbeddingUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RerankRequest {
    pub model: String,
    pub query: String,
    pub documents: Vec<String>,
    #[serde(default)]
    pub top_n: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RerankResponse {
    pub model: String,
    pub results: Vec<RerankResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RerankResult {
    pub index: u32,
    pub document: String,
    pub relevance_score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenizeRequest {
    pub model: String,
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TokenizeResponse {
    pub tokens: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TokenizeCountResponse {
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetokenizeRequest {
    pub model: String,
    pub tokens: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DetokenizeResponse {
    pub text: String,
}

fn completion_options(
    max_tokens: usize,
    min_tokens: Option<usize>,
    logprobs_flag: Option<bool>,
    top_logprobs: Option<usize>,
    stop: &[String],
    sampling: SamplingOptions,
    echo: bool,
    suffix: Option<String>,
) -> CompletionOptions {
    let mut options = CompletionOptions::sampled(max_tokens)
        .with_sampling(sampling)
        .with_stop_sequences(stop.iter().cloned())
        .with_echo_prompt(echo);
    if let Some(min_tokens) = min_tokens {
        options = options.with_min_tokens(min_tokens);
    }
    if let Some(suffix) = suffix {
        options = options.with_suffix(suffix);
    }
    if logprobs_flag.unwrap_or(false) || top_logprobs.is_some() {
        options = options.with_logprobs(top_logprobs.unwrap_or(0));
    }
    options
}

pub fn parse_template(template: Option<&str>) -> Result<BuiltinTemplate> {
    match template {
        Some(template) => BuiltinTemplate::from_str_ci(template).ok_or_else(|| {
            PluginError::invalid_request(format!("unknown chat template: {template}"))
        }),
        None => Ok(BuiltinTemplate::Plain),
    }
}

pub fn stop_reason(reason: llama_crab::StopReason) -> &'static str {
    match reason {
        llama_crab::StopReason::Length => "length",
        llama_crab::StopReason::Eos => "stop",
        llama_crab::StopReason::Stop => "stop",
        llama_crab::StopReason::ToolCalls => "tool_calls",
    }
}

pub fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn json_object_grammar() -> String {
    r#"root ::= object
object ::= "{" ws (string ":" ws value ("," ws string ":" ws value)*)? "}"
array ::= "[" ws (value ("," ws value)*)? "]"
value ::= object | array | string | number | "true" | "false" | "null"
string ::= "\"" ([^"\\] | "\\" ["\\/bfnrt])* "\""
number ::= "-"? ([0-9] | [1-9] [0-9]*) ("." [0-9]+)? ([eE] [-+]? [0-9]+)?
ws ::= [ \t\n\r]*"#
        .to_string()
}
