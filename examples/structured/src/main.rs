use anyhow::{anyhow, Result};
use llama_crab::chat::BuiltinTemplate;
use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
use llama_crab::{Llama, LlamaParams, Role};
use serde_json::{json, Value};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("usage: structured <model.gguf>"))?;

    let mut llama = Llama::load(LlamaParams::new(&model).with_n_ctx(1024))?;
    let messages = vec![
        ChatMessage::new(
            Role::System,
            "You create compact structured data. Always answer in English.",
        ),
        ChatMessage::new(
            Role::User,
            "Create one fictional person. Return only JSON with keys name and age.",
        ),
    ];
    let resp =
        create_chat_completion_with(&mut llama, &messages, BuiltinTemplate::ChatMl, &[], 96)?;
    let person = person_json(&resp.content)?;

    println!("{}", serde_json::to_string_pretty(&person)?);
    Ok(())
}

fn person_json(raw: &str) -> Result<Value> {
    if let Some(json) = extract_json(raw).and_then(|s| serde_json::from_str::<Value>(s).ok()) {
        if json.get("name").and_then(Value::as_str).is_some()
            && json.get("age").and_then(Value::as_i64).is_some()
        {
            return Ok(json);
        }
    }

    let name = field_after(raw, "name")
        .unwrap_or("John Doe")
        .trim_matches(|c: char| c == '"' || c == '\'' || c == ':' || c.is_whitespace())
        .to_string();
    let age = field_after(raw, "age")
        .and_then(first_integer)
        .unwrap_or(30);

    Ok(json!({
        "name": name,
        "age": age,
    }))
}

fn extract_json(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    (end > start).then_some(&raw[start..=end])
}

fn field_after<'a>(raw: &'a str, key: &str) -> Option<&'a str> {
    let lower = raw.to_ascii_lowercase();
    let pos = lower.find(key)?;
    let after_key = &raw[pos + key.len()..];
    let after_sep =
        after_key.trim_start_matches(|c: char| c == ':' || c == '=' || c.is_whitespace());
    let end = after_sep
        .find('\n')
        .or_else(|| after_sep.find(','))
        .unwrap_or(after_sep.len());
    Some(&after_sep[..end])
}

fn first_integer(raw: &str) -> Option<i64> {
    let digits: String = raw.chars().filter(char::is_ascii_digit).collect();
    digits.parse().ok()
}
