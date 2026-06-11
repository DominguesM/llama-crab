//! Chat message types and a thin re-export of the model-side helpers.

/// Role of a chat message in an OpenAI-style conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// System prompt.
    System,
    /// User turn.
    User,
    /// Assistant turn.
    Assistant,
    /// Tool/function result.
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

/// A single chat message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    /// Role (system/user/assistant/tool).
    pub role: Role,
    /// Textual content. Multimodal content is supported by `multimodal::MtmdChatMessage`.
    pub content: String,
    /// Optional tool call id (only meaningful when `role == Tool`).
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    /// Create a new message.
    #[must_use]
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_call_id: None,
        }
    }

    /// Mark this message as a tool result (sets `role = Tool` and the id).
    #[must_use]
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}
