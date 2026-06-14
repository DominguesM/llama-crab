# `vision` — Multimodal image + text

A high-level multimodal example that pairs a text GGUF with an
`mmproj` projector, loads an image, runs a single inference pass and
prints the assistant turn. Requires the `mtmd` Cargo feature.

## Run

```bash
./examples/run.sh vision gemma4
# or
./examples/run.sh vision lfm-vl
```

Downloads ~1–5 GB depending on the model. The `lfm-vl` target is
smaller and faster; `gemma4` is the heavier, higher-quality option.

## What it does

```rust
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
    let mut sampler = LlamaSampler::greedy()?;
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

## Expected output

```
assistant> A red-and-blue checker pattern with 16x16 squares on a 256x256 canvas.
```

The actual description depends on the test image and the model.

## Test image

The repository ships a synthetic `tests/fixtures/test_image.png` —
a 256×256 RGB image with a checker pattern. Use it to verify the
end-to-end flow without needing a real photo:

```bash
cargo run --features mtmd --bin vision --release -- \
  models/gemma-4-E4B-it-Q4_K_M.gguf \
  models/mmproj-gemma-4-E4B-it-BF16.gguf \
  tests/fixtures/test_image.png
```

## Common variations

=== "Different prompt"

    ```rust
    let chunks = mtmd.tokenize(
        MtmdInputText::new("What are the dominant colors in this image?"),
        &[&bitmap],
    )?;
    ```

=== "Multiple images"

    ```rust
    let bitmap_a = MtmdBitmap::from_file("a.png")?;
    let bitmap_b = MtmdBitmap::from_file("b.png")?;
    let chunks = mtmd.tokenize(
        MtmdInputText::new("Compare the two images."),
        &[&bitmap_a, &bitmap_b],
    )?;
    ```

=== "Different sampler"

    ```rust
    use llama_crab::sampling::SamplerChain;
    let mut sampler = SamplerChain::new()
        .temp(0.7)
        .top_p(0.9, 1)
        .build();
    ```

## Pitfalls

- **Wrong `mmproj`** — Gemma 4 and LFM2.5-VL ship different
  projectors. Use the one that matches the text model.
- **Image too large** — large bitmaps waste memory and slow down
  evaluation. Use `MtmdBitmap::resize_to` to downscale to the VLM's
  optimal resolution (usually 336×336 to 896×896).
- **The `mtmd` feature is not enabled** — the example fails to
  compile. Add `features = ["mtmd"]` to the dependency.

## Full source

[`examples/vision/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/vision/src/main.rs).

## Where to next?

- [Multimodal guide](../features/multimodal.md) — the data flow
  and the chunk-evaluation API.
- [Raw mtmd API](mtmd.md) — when you need more control than the
  high-level helpers expose.
- [Server with vision](../server/api.md#multimodal-chat) — the
  HTTP path.
