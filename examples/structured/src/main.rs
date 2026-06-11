use anyhow::Result;
use llama_crab::{Llama, LlamaParams};
use serde_json::json;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: structured <model.gguf>"))?;
    let _schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer"}
        },
        "required": ["name", "age"]
    });
    let mut llama = Llama::load(LlamaParams::new(&model).with_n_ctx(1024))?;
    let resp = llama.create_completion("Generate a fictional person as JSON: ", 32)?;
    println!("{}", resp.text);
    Ok(())
}
