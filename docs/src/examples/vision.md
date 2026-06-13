# `vision` — Multimodal image+text

Requires the `mtmd` cargo feature and a paired `mmproj` projector file.

```rust,no_run
use llama_crab::multimodal::{MtmdBitmap, MtmdContext, MtmdInputText};
use llama_crab::sampling::LlamaSampler;
use llama_crab::token::LlamaToken;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(
        LlamaParams::new("gemma-4-E4B-it-Q4_K_M.gguf").with_n_ctx(4096),
    )?;
    let mtmd = MtmdContext::init_from_file(
        "gemma-4-E4B-it-mmproj.gguf",
        llama.model(),
    )?;
    let bitmap = MtmdBitmap::from_file("image.png")?;
    let chunks = mtmd.tokenize(
        MtmdInputText::new("Describe this image in one sentence."),
        &[&bitmap],
    )?;
    let ctx_ptr = llama.context().raw_handle();
    let new_n_past = unsafe {
        chunks.eval(&mtmd, ctx_ptr, 0, 0, llama.context().n_batch() as i32, true)?
    };
    let mut sampler = LlamaSampler::greedy().unwrap();
    let mut out = String::new();
    let eos = llama.model().token_eos();
    let mut next_pos = new_n_past;
    for _ in 0..128 {
        let tok: LlamaToken = unsafe { sampler.sample(ctx_ptr, -1) };
        sampler.accept(tok);
        if tok == eos { break; }
        if let Ok(piece) = llama.model().detokenize(&[tok], false) {
            out.push_str(&piece);
        }
        let single = llama_crab::batch::LlamaBatch::one(tok, next_pos, 0, true);
        llama.context().decode(&single)?;
        next_pos += 1;
    }
    println!("assistant> {out}");
    Ok(())
}
```

Run with:

```bash
cargo run --features mtmd --bin vision --release -- \
  gemma-4-E4B-it-Q4_K_M.gguf \
  gemma-4-E4B-it-mmproj.gguf \
  image.png
```

## Expected output

```
assistant> A red-and-blue checker pattern with 16x16 squares on a 256x256 canvas.
```
