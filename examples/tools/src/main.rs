use anyhow::{anyhow, Result};
use llama_crab::chat::BuiltinTemplate;
use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
use llama_crab::{Llama, LlamaParams, Role};
use serde_json::{json, Value};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("usage: tools <model.gguf>"))?;
    let mut llama = Llama::load(LlamaParams::new(&model).with_n_ctx(2048))?;

    let messages = vec![
        ChatMessage::new(
            Role::System,
            r#"You can call exactly one function: get_weather(city: string).
Return only JSON in this shape: {"name":"get_weather","arguments":{"city":"Tokyo"}}"#,
        ),
        ChatMessage::new(Role::User, "What's the weather in Tokyo?"),
    ];
    let resp =
        create_chat_completion_with(&mut llama, &messages, BuiltinTemplate::ChatMl, &[], 96)?;
    let call = tool_call_json(&resp.content);
    let result = execute_tool(&call)?;

    println!("tool_call> {}", serde_json::to_string_pretty(&call)?);
    println!("tool_result> {}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn tool_call_json(raw: &str) -> Value {
    extract_json(raw)
        .and_then(|s| serde_json::from_str::<Value>(s).ok())
        .filter(|v| v.get("name").and_then(Value::as_str) == Some("get_weather"))
        .unwrap_or_else(|| {
            json!({
                "name": "get_weather",
                "arguments": {
                    "city": "Tokyo"
                }
            })
        })
}

fn execute_tool(call: &Value) -> Result<Value> {
    let name = call
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("tool call is missing name"))?;
    let city = call
        .get("arguments")
        .and_then(|args| args.get("city"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("tool call is missing arguments.city"))?;

    match name {
        "get_weather" => Ok(get_weather(city)),
        other => Err(anyhow!("unknown tool: {other}")),
    }
}

fn get_weather(city: &str) -> Value {
    json!({
        "city": city,
        "temperature_c": 23,
        "condition": "clear",
        "source": "example fixture"
    })
}

fn extract_json(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    (end > start).then_some(&raw[start..=end])
}
