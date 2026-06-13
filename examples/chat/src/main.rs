use anyhow::Result;
use llama_crab::chat::BuiltinTemplate;
use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
use llama_crab::{Llama, LlamaParams, Role};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: chat <model.gguf>"))?;

    let mut llama = Llama::load(
        LlamaParams::new(&model)
            .with_n_ctx(2048)
            .with_n_gpu_layers(99),
    )?;
    let history = vec![
        ChatMessage::new(
            Role::System,
            "You are a helpful assistant. Always answer in English. Be concise.",
        ),
        ChatMessage::new(
            Role::User,
            "Introduce yourself in one short English sentence.",
        ),
    ];
    let resp =
        create_chat_completion_with(&mut llama, &history, BuiltinTemplate::ChatMl, &[], 128)?;
    println!("assistant> {}", resp.content);
    Ok(())
}
