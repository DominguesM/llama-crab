//! Tool / function-calling types and parsers.
//!
//! [`ToolDefinition`] describes a callable tool (JSON-Schema shape).
//! [`ToolCall`] represents the model's request to invoke one.
//! [`ToolParser`] is a stateful parser that extracts tool calls from a
//! streaming model response.

use serde_json::{json, Value};

use super::message::ChatMessage;

/// A tool the model is allowed to call.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    /// Unique name (e.g. `get_weather`).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema describing the arguments shape.
    pub parameters: Value,
}

impl ToolDefinition {
    /// Construct a new tool definition.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    /// Set the JSON Schema for the tool's arguments.
    #[must_use]
    pub fn with_parameters(mut self, params: Value) -> Self {
        self.parameters = params;
        self
    }

    /// JSON-Schema style: serialize to OpenAI's `tools[].function` shape.
    #[must_use]
    pub fn to_openai_function(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters,
            }
        })
    }

    /// Internal: convert to a JSON object for the template renderer.
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters,
        })
    }
}

/// A single tool invocation request emitted by the model.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    /// Unique id (e.g. `call_abc123`).
    pub id: String,
    /// Name of the tool.
    pub name: String,
    /// Arguments (must conform to the tool's schema).
    pub arguments: Value,
}

impl ToolCall {
    /// Construct a new tool call.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: Value,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments,
        }
    }

    /// JSON shape used by the template renderer.
    pub(crate) fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "arguments": self.arguments,
        })
    }
}

/// Errors produced while parsing a model response for tool calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolParseError {
    /// Model produced a call but its JSON was malformed.
    InvalidJson(String),
    /// Call had a missing `name` field.
    MissingName,
    /// Call had a missing `arguments` field.
    MissingArguments,
}

impl std::fmt::Display for ToolParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(s) => write!(f, "invalid JSON: {s}"),
            Self::MissingName => write!(f, "missing `name` field"),
            Self::MissingArguments => write!(f, "missing `arguments` field"),
        }
    }
}

impl std::error::Error for ToolParseError {}

/// Format the model was trained to use for tool calling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolFormat {
    /// ChatML + JSON function calls (Qwen 2.5, Hermes, etc.).
    /// Format: `<tool_call>{"name": "...", "arguments": {...}}</tool_call>`
    #[default]
    ChatMl,
    /// Mistral / Mixtral `[]` syntax.
    /// Format: `[TOOL_CALLS][{"name": "...", "arguments": {...}}]`
    Mistral,
    /// Llama-3 / Firefunction-v2: `<|python_tag|>` prefix.
    /// Format: `<|python_tag|>{"name": "...", "arguments": {...}}`
    Llama3,
    /// Plain JSON object (no wrapping tokens).
    Plain,
    /// Functionary: `<|start|>function<|message|>...<|call|>`
    Functionary,
}

impl ToolFormat {
    /// Try to auto-detect from the chat template name.
    #[must_use]
    pub fn from_chat_format(name: &str) -> Self {
        let n = name.to_ascii_lowercase();
        if n.contains("qwen") || n.contains("hermes") || n.contains("chatml") {
            Self::ChatMl
        } else if n.contains("mistral") || n.contains("mixtral") {
            Self::Mistral
        } else if n.contains("llama-3") || n.contains("llama3") || n.contains("firefunction") {
            Self::Llama3
        } else if n.contains("functionary") {
            Self::Functionary
        } else {
            Self::Plain
        }
    }
}

/// Stateful tool-call parser.
///
/// ```rust
/// use llama_crab::chat::tool_call::ToolParser;
/// let mut p = ToolParser::new(Default::default());
/// let calls = p.feed("<tool_call>{\"name\": \"foo\", \"arguments\": {}}</tool_call>");
/// assert_eq!(calls.len(), 1);
/// ```
#[derive(Debug)]
pub struct ToolParser {
    format: ToolFormat,
    /// Buffered partial JSON.
    buffer: String,
    /// Number of opening braces seen minus closing braces.
    brace_depth: i32,
    /// True while inside a `<tool_call>` block.
    in_call: bool,
    next_id: u32,
}

impl ToolParser {
    /// Construct a parser for the given format.
    #[must_use]
    pub fn new(format: ToolFormat) -> Self {
        Self {
            format,
            buffer: String::new(),
            brace_depth: 0,
            in_call: false,
            next_id: 0,
        }
    }

    /// Construct from a chat-format name.
    #[must_use]
    pub fn for_chat_format(name: &str) -> Self {
        Self::new(ToolFormat::from_chat_format(name))
    }

    /// Feed a chunk of model output; return the list of tool calls
    /// completed during this chunk.
    pub fn feed(&mut self, chunk: &str) -> Vec<Result<ToolCall, ToolParseError>> {
        let mut out = Vec::new();
        for c in chunk.chars() {
            self.feed_char(c, &mut out);
        }
        out
    }

    fn feed_char(&mut self, c: char, out: &mut Vec<Result<ToolCall, ToolParseError>>) {
        let s = c.to_string();
        match self.format {
            ToolFormat::ChatMl => {
                if !self.in_call {
                    // Looking for `<tool_call>` token.
                    if self.buffer.len() < 12 {
                        self.buffer.push(c);
                        if self.buffer.ends_with("<tool_call>") {
                            self.buffer.clear();
                            self.in_call = true;
                        }
                    } else {
                        // Discard stale buffer.
                        self.buffer.clear();
                        self.buffer.push(c);
                    }
                } else {
                    // Inside call: collect JSON until matching brace.
                    self.buffer.push(c);
                    match c {
                        '{' => self.brace_depth += 1,
                        '}' => {
                            self.brace_depth -= 1;
                            if self.brace_depth == 0 && self.buffer.trim().ends_with('}') {
                                let raw = std::mem::take(&mut self.buffer);
                                if let Some(call) = self.parse_json_call(&raw) {
                                    out.push(Ok(call));
                                }
                                self.in_call = false;
                            }
                        }
                        _ => {}
                    }
                }
            }
            ToolFormat::Mistral => {
                if !self.in_call {
                    self.buffer.push(c);
                    if self.buffer.contains("[TOOL_CALLS]") {
                        // Start scanning for the `[`.
                        self.buffer.clear();
                        self.in_call = true;
                        self.brace_depth = 0;
                    }
                    if self.buffer.len() > 64 {
                        // Truncate to avoid runaway.
                        self.buffer.drain(..self.buffer.len() - 32);
                    }
                } else {
                    self.buffer.push(c);
                    match c {
                        '[' => self.brace_depth += 1,
                        ']' => {
                            self.brace_depth -= 1;
                            if self.brace_depth <= 0 {
                                let raw = std::mem::take(&mut self.buffer);
                                // Try to parse as `[ {...} ]` or `[ {...}, {...} ]`.
                                if let Ok(Value::Array(items)) =
                                    serde_json::from_str::<Value>(&raw)
                                {
                                    for item in items {
                                        if let Some(call) = self.parse_call_obj(&item) {
                                            out.push(Ok(call));
                                        }
                                    }
                                }
                                self.in_call = false;
                                self.brace_depth = 0;
                            }
                        }
                        _ => {}
                    }
                }
            }
            ToolFormat::Llama3 => {
                if !self.in_call {
                    self.buffer.push(c);
                    if self.buffer.ends_with("<|python_tag|>") {
                        self.buffer.clear();
                        self.in_call = true;
                        self.brace_depth = 0;
                    }
                } else {
                    self.buffer.push(c);
                    match c {
                        '{' => self.brace_depth += 1,
                        '}' => {
                            self.brace_depth -= 1;
                            if self.brace_depth == 0 {
                                let raw = std::mem::take(&mut self.buffer);
                                if let Some(call) = self.parse_json_call(&raw) {
                                    out.push(Ok(call));
                                }
                                self.in_call = false;
                            }
                        }
                        _ => {}
                    }
                }
            }
            ToolFormat::Plain => {
                self.buffer.push(c);
                match c {
                    '{' => self.brace_depth += 1,
                    '}' => {
                        self.brace_depth -= 1;
                        if self.brace_depth == 0 && self.buffer.trim().starts_with('{') {
                            let raw = std::mem::take(&mut self.buffer);
                            if let Some(call) = self.parse_json_call(&raw) {
                                out.push(Ok(call));
                            }
                        }
                    }
                    _ => {}
                }
            }
            ToolFormat::Functionary => {
                if !self.in_call {
                    self.buffer.push(c);
                    if self.buffer.contains("<|call|>") {
                        // The payload is the JSON before this tag.
                        let raw = self
                            .buffer
                            .replace("<|call|>", "")
                            .replace("<|start|>function<|message|>", "")
                            .trim()
                            .to_string();
                        self.buffer.clear();
                        if let Some(call) = self.parse_json_call(&raw) {
                            out.push(Ok(call));
                        }
                    }
                    if self.buffer.len() > 1024 {
                        self.buffer.clear();
                    }
                }
                let _ = s;
            }
        }
    }

    /// Flush remaining buffered content as a final call (used at end-of-stream).
    pub fn finish(&mut self) -> Vec<Result<ToolCall, ToolParseError>> {
        let mut out = Vec::new();
        let buf = std::mem::take(&mut self.buffer);
        if !buf.is_empty() && buf.trim().starts_with('{') && buf.trim().ends_with('}') {
            if let Some(call) = self.parse_json_call(&buf) {
                out.push(Ok(call));
            }
        }
        out
    }

    fn parse_json_call(&mut self, raw: &str) -> Option<ToolCall> {
        let v: Value = serde_json::from_str(raw).ok()?;
        self.parse_call_obj(&v)
    }

    fn parse_call_obj(&mut self, v: &Value) -> Option<ToolCall> {
        let name = v.get("name")?.as_str()?.to_string();
        let arguments = v.get("arguments")?.clone();
        self.next_id += 1;
        let id = format!("call_{}", self.next_id);
        Some(ToolCall::new(id, name, arguments))
    }
}

/// Helper: extract tool calls from a complete (non-streaming) response.
pub fn extract_tool_calls(format: ToolFormat, text: &str) -> Vec<Result<ToolCall, ToolParseError>> {
    let mut p = ToolParser::new(format);
    p.feed(text)
}

/// Convenience: turn tool calls into a synthetic assistant message.
///
/// Useful when you want to feed tool calls back into a chat history.
pub fn tool_calls_to_message(calls: &[ToolCall]) -> ChatMessage {
    use super::message::Role;
    let mut m = ChatMessage::new(Role::Assistant, String::new());
    for c in calls {
        m = m.with_tool_call(c.clone());
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chatml() {
        let s = r#"<tool_call>{"name": "get_weather", "arguments": {"city": "Tokyo"}}</tool_call>"#;
        let mut p = ToolParser::new(ToolFormat::ChatMl);
        let calls: Vec<_> = p.feed(s).into_iter().filter_map(|r| r.ok()).collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "get_weather");
    }

    #[test]
    fn parse_mistral() {
        let s = r#"[TOOL_CALLS][{"name": "x", "arguments": {}}]"#;
        let mut p = ToolParser::new(ToolFormat::Mistral);
        let calls: Vec<_> = p.feed(s).into_iter().filter_map(|r| r.ok()).collect();
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn parse_llama3() {
        let s = r#"<|python_tag|>{"name": "x", "arguments": {}}"#;
        let mut p = ToolParser::new(ToolFormat::Llama3);
        let calls: Vec<_> = p.feed(s).into_iter().filter_map(|r| r.ok()).collect();
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn parse_plain() {
        let s = r#"{"name": "x", "arguments": {}}"#;
        let mut p = ToolParser::new(ToolFormat::Plain);
        let calls: Vec<_> = p.feed(s).into_iter().filter_map(|r| r.ok()).collect();
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn auto_detect_format() {
        assert_eq!(ToolFormat::from_chat_format("qwen"), ToolFormat::ChatMl);
        assert_eq!(ToolFormat::from_chat_format("llama-3"), ToolFormat::Llama3);
        assert_eq!(ToolFormat::from_chat_format("mistral"), ToolFormat::Mistral);
        assert_eq!(ToolFormat::from_chat_format("plain"), ToolFormat::Plain);
    }
}
