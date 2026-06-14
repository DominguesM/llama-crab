use anyhow::{anyhow, Result};
use llama_crab::chat::{render_builtin, BuiltinTemplate};
use llama_crab::high_level::chat_completion::ChatMessage;
use llama_crab::high_level::completion::{json_schema_grammar, CompletionOptions};
use llama_crab::sampling::LlamaSampler;
use llama_crab::{Llama, LlamaParams, Role};
use serde_json::{json, Value};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("usage: structured <model.gguf>"))?;

    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" }
        },
        "required": ["name", "age"]
    });
    let grammar_text = json_schema_grammar(&schema)?;
    let mut llama = Llama::load(LlamaParams::new(&model).with_n_ctx(1024))?;
    let grammar = unsafe { LlamaSampler::grammar(llama.model(), &grammar_text, "root")? };
    let greedy =
        LlamaSampler::greedy().ok_or_else(|| anyhow!("failed to create greedy sampler"))?;
    let mut sampler = LlamaSampler::chain(vec![grammar, greedy], false)
        .ok_or_else(|| anyhow!("failed to create sampler chain"))?;
    let messages = vec![
        ChatMessage::new(
            Role::System,
            "You create compact structured data. Return only JSON.",
        ),
        ChatMessage::new(
            Role::User,
            "Create one fictional person. Return only JSON with keys name and age.",
        ),
    ];
    let prompt = render_builtin(BuiltinTemplate::ChatMl, &messages, &[], true);
    let resp =
        llama.create_completion_with_sampler(&prompt, CompletionOptions::new(96), &mut sampler)?;
    let person: Value = serde_json::from_str(resp.text.trim())?;

    println!("{}", serde_json::to_string_pretty(&person)?);
    Ok(())
}
