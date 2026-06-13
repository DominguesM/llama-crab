use anyhow::Result;
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::{Llama, LlamaParams, Role};
use serde_json::json;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: tools <model.gguf>"))?;
    let mut llama = Llama::load(LlamaParams::new(&model).with_n_ctx(2048))?;

    // Tool definitions (JSON schema format).
    let _tools = json!([
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get the current weather for a city",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {"type": "string"}
                    },
                    "required": ["city"]
                }
            }
        }
    ]);

    let messages = vec![
        ChatMessage::new(
            Role::System,
            "You can call functions. Format: {\"name\": \"...\", \"arguments\": {...}}",
        ),
        ChatMessage::new(Role::User, "What's the weather in Tokyo?"),
    ];
    let resp = llama.create_chat_completion(&messages, 64)?;
    println!("assistant> {}", resp.content);
    Ok(())
}
