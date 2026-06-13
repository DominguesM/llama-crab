# Chat & tool calling

`llama-crab` provides a full chat pipeline: messages, templates,
tool/function calling and a streaming parser.

## Messages

```rust,no_run
use llama_crab::chat::{ChatMessage, Role};
let messages = vec![
    ChatMessage::new(Role::System, "You are a helpful assistant."),
    ChatMessage::new(Role::User, "Hi!"),
];
```

## Templates

`llama-crab` ships with a **Jinja2 subset** renderer that supports
the templating primitives used by 95% of real chat models (`if`,
`for`, `set`, attribute/subscript access, filters, list/dict literals,
`and`/`or`/`not`/`in`). It also includes 14 built-in templates that
cover the most popular open-weights models:

```rust,no_run
use llama_crab::chat::{BuiltinTemplate, render_builtin, ChatMessage, Role};

let prompt = render_builtin(
    BuiltinTemplate::Llama3,
    &[ChatMessage::new(Role::User, "Hi")],
    &[],  // no tools
    true, // add the assistant turn-prefix
);
```

Auto-detect the format from GGUF metadata:

```rust,no_run
use llama_crab::chat::detect_chat_format;
use std::collections::BTreeMap;

let mut md = BTreeMap::new();
md.insert("general.architecture".into(), "gemma4".into());
let tpl = detect_chat_format(&md);
```

## Tool calling

Define a tool and pass it to the renderer:

```rust,no_run
use llama_crab::chat::{ToolDefinition, ToolParser, ToolFormat, extract_tool_calls};
use serde_json::json;

let tool = ToolDefinition::new("get_weather", "Get the weather for a city")
    .with_parameters(json!({
        "type": "object",
        "properties": { "city": { "type": "string" } },
        "required": ["city"]
    }));

// The model response is parsed for tool calls.
let response = r#"<tool_call>{"name": "get_weather", "arguments": {"city": "Tokyo"}}</tool_call>"#;
let mut parser = ToolParser::for_chat_format("qwen");
let calls: Vec<_> = parser.feed(response).into_iter().filter_map(|r| r.ok()).collect();
assert_eq!(calls.len(), 1);
```

Supported formats:

| Format        | Trigger syntax                 |
| ------------- | ------------------------------ | ---------- | ---------- | ------- | ----- | ---- | --- |
| `ChatMl`      | `<tool_call>{...}</tool_call>` |
| `Mistral`     | `[TOOL_CALLS][{...}]`          |
| `Llama3`      | `<                             | python_tag | >{...}`    |
| `Plain`       | `{...}` (any JSON object)      |
| `Functionary` | `<                             | start      | >function< | message | >...< | call | >`  |

The parser is **stateful**: feed it token-by-token as the model
generates, and it will emit completed calls as they appear.
