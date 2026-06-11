use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let _model = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("usage: mtmd <model.gguf> <mmproj.gguf> <image>"))?;
    eprintln!("This example requires the `mtmd` cargo feature:");
    eprintln!("    cargo run --features mtmd --bin mtmd -- ... ");
    Ok(())
}
