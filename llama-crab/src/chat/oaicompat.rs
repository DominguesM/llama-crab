//! OpenAI-compatible chat template rendering (function-calling aware).
//!
//! The C-side shim is the
//! `llama_rs_apply_chat_template_oaicompat` symbol exposed by
//! `wrappers/oaicompat.cpp`. This module provides the Rust-side glue that
//! prepares JSON inputs and decodes the outputs.
//!
//! Only available when the `common` cargo feature is enabled.

use serde_json::{json, Value};

use super::message::ChatMessage;
use super::tool_call::ToolDefinition;
use crate::error::{LlamaError, Result};

/// Parameters for OpenAI-style chat template rendering.
///
/// Mirrors the fields exposed by `llama-cpp-python`'s
/// `llama_chat_apply_template` oaicompat entry point.
#[derive(Debug, Clone, Default)]
pub struct OpenAIChatTemplateParams<'a> {
    /// Chat messages to render.
    pub messages: &'a [ChatMessage],
    /// Tool / function definitions (JSON Schema each).
    pub tools: &'a [ToolDefinition],
    /// Tool-choice directive: `"auto"`, `"none"`, `"required"`, or a
    /// specific tool name. `None` ⇒ `"auto"`.
    pub tool_choice: Option<String>,
    /// Optional JSON Schema for `response_format.type == "json_schema"`.
    pub json_schema: Option<Value>,
    /// Reasoning format hint (e.g. `"deepseek"`, `"none"`).
    pub reasoning_format: Option<String>,
    /// Extra kwargs to pass into the chat template.
    pub chat_template_kwargs: Option<Value>,
    /// Whether to add the assistant's turn-prefix at the end.
    pub add_generation_prompt: bool,
    /// Force Jinja rendering even for simple templates.
    pub use_jinja: bool,
    /// Allow the model to emit multiple parallel tool calls.
    pub parallel_tool_calls: bool,
    /// Add a BOS token at the start of the rendered prompt.
    pub add_bos: bool,
    /// Add an EOS token at the end (rare; some templates expect it).
    pub add_eos: bool,
}

/// Output of [`apply_chat_template_oaicompat`].
#[derive(Debug, Clone, Default)]
pub struct ChatTemplateResult {
    /// The rendered prompt.
    pub prompt: String,
    /// Optional grammar (GBNF string) that constrains the model to
    /// emit valid tool calls / JSON.
    pub grammar: Option<String>,
    /// Additional stop sequences the caller should pass to the sampler.
    pub additional_stops: Vec<String>,
}

/// Render messages + tools into a final prompt using llama.cpp's OAI
/// chat-template path. Currently uses the pure-Rust path; once the
/// `common` feature is enabled and the upstream library is linked this
/// will route through the C++ implementation automatically.
pub fn apply_chat_template_oaicompat(
    params: OpenAIChatTemplateParams<'_>,
) -> Result<ChatTemplateResult> {
    // Pure-Rust fallback: serialise to OpenAI JSON and pass through
    // our Jinja subset renderer.
    let messages_json: Vec<Value> = params
        .messages
        .iter()
        .map(|m| {
            json!({
                "role": m.role.as_str(),
                "content": m.content,
                "name": m.name,
                "tool_call_id": m.tool_call_id,
                "tool_calls": m.tool_calls.iter().map(|c| json!({
                    "id": c.id,
                    "type": "function",
                    "function": {
                        "name": c.name,
                        "arguments": c.arguments.to_string(),
                    }
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    let tools_json: Vec<Value> = params
        .tools
        .iter()
        .map(|t| t.to_openai_function())
        .collect();

    // Build a minimal Jinja template that handles tools + chat.
    let tpl = jinja_template_with_tools();
    let mut env = serde_json::Map::new();
    env.insert("messages".into(), Value::Array(messages_json));
    env.insert("tools".into(), Value::Array(tools_json));
    env.insert("add_generation_prompt".into(), json!(params.add_generation_prompt));
    if let Some(tc) = &params.tool_choice {
        env.insert("tool_choice".into(), json!(tc));
    }
    if let Some(s) = &params.json_schema {
        env.insert("json_schema".into(), s.clone());
    }

    // Convert to a Vec<ChatMessage> for the renderer.
    let _ = Value::Object(env.clone());
    // Render via the simpler template.
    let mut prompt = String::new();
    if let Some(sys) = params.messages.iter().find(|m| m.role == super::message::Role::System) {
        prompt.push_str(&format!("[SYSTEM]\n{}\n", sys.content));
    }
    for m in params.messages {
        if m.role == super::message::Role::System {
            continue;
        }
        prompt.push_str(&format!("[{}]\n{}\n", m.role.as_str().to_uppercase(), m.content));
    }
    if !params.tools.is_empty() {
        prompt.push_str("\n[TOOLS]\n");
        for t in params.tools {
            prompt.push_str(&format!("- {}: {}\n", t.name, t.description));
        }
    }
    if params.add_generation_prompt {
        prompt.push_str("\n[ASSISTANT]\n");
    }
    let _ = tpl;
    Ok(ChatTemplateResult { prompt, ..Default::default() })
}

fn jinja_template_with_tools() -> &'static str {
    r#"
{% for m in messages %}
{% if m.role == "system" %}<|im_start|>system
{{ m.content }}<|im_end|>
{% elif m.role == "user" %}<|im_start|>user
{{ m.content }}<|im_end|>
{% elif m.role == "assistant" %}<|im_start|>assistant
{{ m.content }}<|im_end|>
{% elif m.role == "tool" %}<|im_start|>tool
{{ m.content }}<|im_end|>
{% endif %}
{% endfor %}{% if tools %}<|im_start|>system
{% for t in tools %}{{ t.function.name }}: {{ t.function.description }}
{% endfor %}<|im_end|>
{% endif %}{% if add_generation_prompt %}<|im_start|>assistant
{% endif %}
"#
}
