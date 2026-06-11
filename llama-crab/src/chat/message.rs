//! [`ChatMessage`] and [`Role`] types.

/// Role of a chat message in an OpenAI-style conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Role {
    /// System prompt.
    #[serde(rename = "system")]
    System,
    /// User turn.
    #[serde(rename = "user")]
    User,
    /// Assistant turn.
    #[serde(rename = "assistant")]
    Assistant,
    /// Tool/function result.
    #[serde(rename = "tool")]
    Tool,
}

impl Role {
    /// Conventional wire name (`system`, `user`, `assistant`, `tool`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Role {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "system" => Self::System,
            "user" => Self::User,
            "assistant" => Self::Assistant,
            "tool" => Self::Tool,
            other => return Err(format!("unknown chat role: {other}")),
        })
    }
}

/// A single chat message.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    /// Role (system/user/assistant/tool).
    pub role: Role,
    /// Textual content. Multimodal content is supported by
    /// `crate::multimodal` (feature `mtmd`).
    pub content: String,
    /// Optional tool call id (only meaningful when `role == Tool`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Optional tool calls emitted by the assistant in this message.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    /// Optional name (for function-style messages).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ChatMessage {
    /// Create a new message.
    #[must_use]
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_call_id: None,
            tool_calls: Vec::new(),
            name: None,
        }
    }

    /// Mark this message as a tool result (sets `role = Tool` and the id).
    #[must_use]
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
            tool_calls: Vec::new(),
            name: None,
        }
    }

    /// Builder-style: add a tool call.
    #[must_use]
    pub fn with_tool_call(mut self, call: ToolCall) -> Self {
        self.tool_calls.push(call);
        self
    }
}

// Avoid orphan-rule violations; we need `ToolCall` in scope.
use crate::chat::tool_call::ToolCall;
