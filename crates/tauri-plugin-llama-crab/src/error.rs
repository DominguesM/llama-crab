use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginError {
    pub kind: &'static str,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, PluginError>;

impl PluginError {
    pub fn new(kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("invalidRequest", message)
    }

    pub fn model_not_found(id: &str) -> Self {
        Self::new("modelNotFound", format!("model `{id}` is not loaded"))
    }

    pub fn worker_spawn_failed(message: impl Into<String>) -> Self {
        Self::new("workerSpawnFailed", message)
    }

    pub fn worker_disconnected() -> Self {
        Self::new(
            "workerDisconnected",
            "the worker thread is no longer running",
        )
    }

    pub fn worker_panicked(message: impl Into<String>) -> Self {
        Self::new("workerPanicked", message)
    }

    pub fn multimodal_not_enabled() -> Self {
        Self::new(
            "multimodalNotEnabled",
            "multimodal chat requires the `mtmd` feature on tauri-plugin-llama-crab",
        )
    }

    pub fn multimodal_setup(message: impl Into<String>) -> Self {
        Self::new("multimodalSetup", message)
    }

    pub fn media_decode(message: impl Into<String>) -> Self {
        Self::new("mediaDecode", message)
    }

    pub fn inference(error: llama_crab::LlamaError) -> Self {
        Self::new("inference", error.to_string())
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for PluginError {}

impl From<llama_crab::LlamaError> for PluginError {
    fn from(value: llama_crab::LlamaError) -> Self {
        Self::inference(value)
    }
}

impl From<std::sync::mpsc::RecvError> for PluginError {
    fn from(_: std::sync::mpsc::RecvError) -> Self {
        Self::worker_disconnected()
    }
}
