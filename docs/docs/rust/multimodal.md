---
title: Multimodal
---

# Multimodal

Multimodal support is behind the `mtmd` feature and uses a text GGUF plus a
matching `mmproj` projector file.

```toml
[dependencies]
llama-crab = { version = "0.1.4", features = ["mtmd"] }
```

For source checkout examples and model download helpers, see
[Development](../contributing/development.md).

## Core flow

The high-level vision examples follow the same sequence:

1. Load the text model with `Llama::load`.
2. Initialize `MtmdContext` from the projector file.
3. Load an image as `MtmdBitmap`.
4. Insert `default_media_marker()` into the prompt.
5. Tokenize text plus media with `mtmd.tokenize`.
6. Evaluate chunks into the llama context.
7. Sample tokens and feed each token back through the context.

```rust
use llama_crab::multimodal::{default_media_marker, MtmdBitmap, MtmdContext, MtmdInputText};
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("models/model.gguf").with_n_ctx(4096))?;
let mtmd = MtmdContext::init_from_file("models/mmproj.gguf", llama.model())?;

if !mtmd.support_vision() {
    anyhow::bail!("this projector does not support vision");
}

let bitmap = MtmdBitmap::from_file("images/sample.png")?;
let prompt = format!("{}\nDescribe this image.", default_media_marker());
let chunks = mtmd.tokenize(MtmdInputText::new(&prompt), &[&bitmap])?;
```

Use the same flow in your application decode loop: evaluate the multimodal
chunks, sample tokens, and feed each sampled token back through the context.

## Prompt templates

Some vision-language models need model-specific chat framing. The LFM examples
wrap the media marker in ChatML-style turns:

```text
<|im_start|>user
<media marker>
Describe this image.<|im_end|>
<|im_start|>assistant
```

If a model returns empty or irrelevant image answers, verify that the projector
matches the text GGUF and that the prompt format matches the model family.

## HTTP server

The server can also run multimodal chat when installed with `mtmd` and started
with `--mmproj`:

```bash
llama-crab-server \
  --model /models/vision.gguf \
  --mmproj /models/mmproj.gguf
```

Send image content parts to `/v1/chat/completions`.
