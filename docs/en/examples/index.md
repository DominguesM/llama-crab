# Examples

The repository ships with **14 self-contained example crates** in
[`examples/`], one per public feature. Each is a standalone Cargo
crate you can copy from.

<div class="grid cards" markdown>

-   :material-rocket-launch: __[Quickstart](quickstart.md)__

    The smallest end-to-end program: load, tokenize, complete, chat,
    FIM. ~80 lines, fully annotated.

-   :material-text-box: __[Plain completion](simple.md)__

    One-shot text completion with a custom sampler chain.

-   :material-broadcast: __[Streaming](streaming.md)__

    Token-by-token output through the high-level callback API.

-   :material-message-text: __[Multi-turn chat](chat.md)__

    A two-turn chat using a built-in template.

-   :material-console: __[Stateful REPL](stateful-chat.md)__

    Interactive multi-turn REPL with `/clear`, `/save`, EOF
    handling.

-   :material-image-multiple: __[Vision (mtmd)](vision.md)__

    Multimodal image + text with the high-level `MtmdContext` API.

-   :material-code-tags: __[Raw mtmd.h API](mtmd.md)__

    Lower-level mtmd.h API: bitmap → chunks → eval.

-   :material-vector-arrange-above: __[Embeddings](embeddings.md)__

    Embedding extraction with L2 normalisation.

-   :material-magnify: __[Semantic search](embedding-search.md)__

    BGE-small + cosine ranking over a small corpus.

-   :material-order-alphabetical-ascending: __[Reranker](reranker.md)__

    Bi-encoder reranker demo.

-   :material-tools: __[Tool calling](tools.md)__

    `ToolDefinition` + five `ToolParser` formats.

-   :material-code-braces: __[Structured output](structured.md)__

    JSON-Schema → GBNF → constrained JSON output.

-   :material-fast-forward: __[Speculative decoding](speculative.md)__

    `PromptLookupDecoding` draft decoding.

</div>

## One-command runner

Every example is wrapped by `examples/run.sh`, which downloads the
right model on first run and is idempotent afterwards:

```bash
./examples/run.sh quickstart            # ~400 MB — text only, smallest demo
./examples/run.sh chat                  # same model — interactive REPL
./examples/run.sh stateful_chat         # multi-turn REPL with /clear, /save
./examples/run.sh embeddings            # ~30 MB — BGE-small embedding
./examples/run.sh embedding_search      # BGE-small + cosine ranking
./examples/run.sh reranker              # bi-encoder scoring
./examples/run.sh vision gemma4         # ~5 GB — vision + text chat
./examples/run.sh vision lfm-vl         # ~1 GB — smaller vision model
./examples/run.sh mtmd gemma4           # raw mtmd.h API
./examples/run.sh tools                 # function calling
./examples/run.sh structured            # JSON-schema grammar
./examples/run.sh speculative           # prompt-lookup draft decoding
```

Without arguments, the script lists every available example.

## Full table

| Example | Model | Size | What it shows |
| --- | --- | --- | --- |
| [`quickstart`](quickstart.md) | `Qwen2.5-0.5B-Instruct-GGUF` | ~400 MB | Load → tokenize → complete → chat → FIM |
| [`simple`](simple.md) | any text GGUF | varies | Plain text completion |
| [`streaming`](streaming.md) | same as `quickstart` | ~400 MB | High-level token-by-token output |
| [`chat`](chat.md) | instruct GGUF | varies | One-shot chat with a builtin template |
| [`stateful_chat`](stateful-chat.md) | same as `quickstart` | ~400 MB | REPL with growing history, `/clear`, `/save` |
| [`vision`](vision.md) | Gemma 4 or LFM2.5-VL + mmproj | ~1–5 GB | High-level `MtmdContext` vision chat |
| [`mtmd`](mtmd.md) | Gemma 4 + mmproj | ~5 GB | Raw `mtmd.h` API: bitmap → chunks → eval |
| [`embeddings`](embeddings.md) | `bge-small-en-v1.5-gguf` | ~30 MB | Embedding extraction + L2 norm |
| [`embedding_search`](embedding-search.md) | `bge-small-en-v1.5-gguf` | ~30 MB | Semantic search with cosine ranking |
| [`reranker`](reranker.md) | embedding GGUF | varies | Bi-encoder ranking by cosine similarity |
| [`tools`](tools.md) | tool-aware instruct GGUF | varies | `ToolDefinition` + 5 `ToolParser` formats |
| [`structured`](structured.md) | any text GGUF | varies | `json_schema_grammar()` + JSON parsing |
| [`speculative`](speculative.md) | any text GGUF | varies | `prompt-lookup` n-gram draft |

## Passing a different model

Every example accepts the GGUF path as the **first** positional
argument:

```bash
cargo run --release --bin run_quickstart -- models/llama-3.2-1b-instruct-q4_k_m.gguf
```

Vision examples take `<text.gguf> <mmproj.gguf> <image>`.

## Adding a new example

The boilerplate for a new example crate is ~15 lines:

```toml title="examples/my_example/Cargo.toml"
[package]
name = "my_example"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish = false

[[bin]]
name = "run_my_example"
path = "src/main.rs"

[dependencies]
llama-crab = { path = "../../llama-crab", version = "0.1.0" }
anyhow = "1"
```

```rust title="examples/my_example/src/main.rs"
use anyhow::Result;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<()> {
    let mut llama = Llama::load(LlamaParams::new("models/your.gguf"))?;
    let resp = llama.create_completion("Hello!", 32)?;
    print!("{}", resp.text);
    Ok(())
}
```

Then add `examples/my_example` to the `members = [...]` list in the
root `Cargo.toml` and a row to the table on this page.

## Where to next?

- [Quickstart](quickstart.md) — the smallest end-to-end program.
- [Streaming](streaming.md) — the most common request from app
  developers.
- [Vision (mtmd)](vision.md) — if you want to feed images to a
  model.
- [Chatbot recipe](../recipes/chatbot.md) — when a single example
  isn't enough and you need to wire a full agent.
