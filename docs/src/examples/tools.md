# `tools` — Tool / function calling

The example asks the model to invoke a tool; the response is parsed
back into a `ToolCall` struct.

```rust,no_run
use llama_crab::chat::tool_call::{ToolDefinition, ToolFormat, ToolParser};
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::chat::{BuiltinTemplate, render_builtin};
use llama_crab::{Llama, LlamaParams, Role};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tool = ToolDefinition::new("get_weather", "Get the weather for a city")
        .with_parameters(serde_json::json!({
            "type": "object",
            "properties": { "city": { "type": "string" } },
            "required": ["city"]
        }));
    let prompt = render_builtin(
        BuiltinTemplate::Qwen2_5,
        &[ChatMessage::new(Role::User, "Weather in Tokyo?")],
        &[tool.clone()],
        true,
    );
    let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(2048))?;
    let resp = llama.create_completion(&prompt, 64)?;
    let mut parser = ToolParser::for_chat_format("qwen");
    let calls: Vec<_> = parser
        .feed(&resp.text)
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();
    for call in calls {
        println!("name: {}", call.name);
        println!("args: {}", call.arguments);
    }
    Ok(())
}
```

Run with:

```bash
cargo run --bin tools --release -- model.gguf
```

## Expected output

```
name: get_weather
args: {"city": "Tokyo"}
```
