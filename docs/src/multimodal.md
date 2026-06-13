# Multimodal (vision + audio)

With the `mtmd` cargo feature, `llama-crab` exposes llama.cpp's
multimodal stack. This lets you pair a text GGUF with an `mmproj`
projector and feed images (or audio) into the same context.

## Loading

```rust,no_run
# #[cfg(feature = "mtmd")] {
use llama_crab::multimodal::MtmdContext;
use llama_crab::Llama;

let mut llama = Llama::load(
    llama_crab::LlamaParams::new("gemma-4-E4B-it-Q4_K_M.gguf").with_n_ctx(4096)
)?;
let mtmd = MtmdContext::init_from_file("gemma-4-E4B-it-mmproj.gguf", llama.model())?;
# let _ = mtmd;
# let _ = llama;
# }
```

## Running a vision prompt

```rust,no_run
# #[cfg(feature = "mtmd")] {
use llama_crab::multimodal::{MtmdBitmap, MtmdInputText};
use llama_crab::sampling::LlamaSampler;
use llama_crab::token::LlamaToken;
# use llama_crab::{Llama, LlamaParams};
# let mut llama = Llama::load(LlamaParams::new("x").with_n_ctx(4096)).unwrap();
# let mtmd = MtmdContext::init_from_file("y", llama.model()).unwrap();
# let mut bitmap = MtmdBitmap::from_image_data(256, 256, &[0u8; 256 * 256 * 3]).unwrap();

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
for _ in 0..64 {
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
println!("{out}");
# }
```

## Tested models

| Model                                                   | Status        |
| ------------------------------------------------------- | ------------- |
| `lmstudio-community/gemma-4-E4B-it-GGUF`                | ✅            |
| `unsloth/LFM2.5-VL-1.6B-GGUF`                           | ✅            |
| `Qwen2.5-VL`, `Llama-3.2-Vision`, `LLaVA-1.5/1.6`, etc. | ✅ (via mtmd) |

See `examples/vision/` for a runnable program and the `tests/`
folder for the integration test suite.
