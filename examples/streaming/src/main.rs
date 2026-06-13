use std::error::Error;
use std::io::{self, Write};

use llama_crab::{CompletionOptions, Llama, LlamaParams, StreamControl};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let model = args
        .next()
        .ok_or("usage: run_streaming <model.gguf> [prompt] [max_tokens]")?;
    let prompt = args
        .next()
        .unwrap_or_else(|| "Write one short sentence about Rust.".to_string());
    let max_tokens = args
        .next()
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(64);

    let mut llama = Llama::load(
        LlamaParams::new(&model)
            .with_n_ctx(512)
            .with_n_gpu_layers(99),
    )?;

    let mut stdout = io::stdout().lock();
    let mut write_error: Option<io::Error> = None;
    let _completion =
        llama.create_completion_stream(&prompt, CompletionOptions::new(max_tokens), |chunk| {
            if let Err(err) = write!(stdout, "{}", chunk.text).and_then(|_| stdout.flush()) {
                write_error = Some(err);
                return StreamControl::Stop;
            }
            StreamControl::Continue
        })?;

    if let Some(err) = write_error {
        return Err(err.into());
    }

    writeln!(stdout)?;
    Ok(())
}
