# `mtmd` — Raw `mtmd.h` API

A lower-level counterpart to [`vision`](./vision.md): loads a text
model and an `mmproj` projector, embeds an image and a prompt into
a chunk list, then drives the decode loop by hand. Use this when you
need direct access to `MtmdBitmap` / `MtmdInputText` / `chunks.eval`.

The higher-level [`vision`](./vision.md) example does the same thing
in fewer lines through the `MtmdContext` helper.

## Run

```bash
./examples/run.sh mtmd gemma4
# or, manually:
./scripts/download_models.sh gemma4
cargo run --release --bin mtmd -- \
  models/gemma-4-E4B-it-Q4_K_M.gguf \
  models/mmproj-gemma-4-E4B-it-BF16.gguf \
  tests/fixtures/test_image.png \
  "Describe this image in one short sentence."
```

Downloads the Gemma 4 text GGUF + its mmproj projector (~5 GB total).

## What it does

```rust,no_run
# #[cfg(feature = "mtmd")] {
use llama_crab::batch::LlamaBatch;
use llama_crab::multimodal::{default_media_marker, MtmdBitmap, MtmdContext, MtmdInputText};
use llama_crab::sampling::LlamaSampler;
use llama_crab::token::LlamaToken;
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(4096))?;
let mtmd = MtmdContext::init_from_file("mmproj.gguf", llama.model())?;

let bitmap = MtmdBitmap::from_file("image.png")?;
let marker = default_media_marker();
let prompt = format!("{marker}\nDescribe this image in one short sentence.");

let chunks = mtmd.tokenize(MtmdInputText::new(&prompt), &[&bitmap])?;

let ctx_ptr = llama.context().raw_handle();
let mut n_past = unsafe {
    chunks.eval(&mtmd, ctx_ptr, 0, 0, llama.context().n_batch() as i32, true)?
};

// Standard decode loop with a greedy sampler.
let mut sampler = LlamaSampler::greedy()?;
let eos = llama.model().token_eos();
let mut out = String::new();
for i in 0..96 {
    let idx = if i == 0 { -1 } else { 0 };
    let tok: LlamaToken = unsafe { sampler.sample(ctx_ptr, idx) };
    sampler.accept(tok);
    if tok == eos { break; }
    out.push_str(&llama.model().detokenize(&[tok], false)?);
    let single = LlamaBatch::one(tok, n_past + i as i32, 0, true);
    llama.context().decode(&single)?;
}
println!("{out}");
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Full source

[`examples/mtmd/src/main.rs`][src]. The same flow is exercised by
the integration tests in `llama-crab/tests/gemma4_vision.rs` and
`llama-crab/tests/lfm_vl_vision.rs`.

[src]: https://github.com/DominguesM/llama-crab/tree/main/examples/mtmd/src/main.rs
