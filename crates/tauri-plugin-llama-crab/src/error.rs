use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginError {
    pub kind: String,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, PluginError>;

impl PluginError {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("invalidRequest", message)
    }

    pub fn model_not_found(id: &str) -> Self {
        Self::new("modelNotFound", format!("model `{id}` is not loaded"))
    }

    pub fn worker(message: impl Into<String>) -> Self {
        Self::new("worker", message)
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
