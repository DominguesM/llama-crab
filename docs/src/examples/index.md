# Examples overview

The repo ships with **14 self-contained example crates** in
[`examples/`], one per public feature. Each is a standalone Cargo
crate you can copy from.

## One-command runner

```bash
./examples/run.sh quickstart            # ~400 MB — text only, smallest demo
./examples/run.sh chat                  # same model — interactive REPL
./examples/run.sh stateful_chat         # multi-turn REPL with /clear, /save
./examples/run.sh embeddings            # ~30 MB — BGE-small embedding
./examples/run.sh embedding_search      # BGE-small + cosine ranking
./examples/run.sh reranker              # cross-encoder scoring
./examples/run.sh vision gemma4         # ~5 GB — vision + text chat
./examples/run.sh vision lfm-vl         # ~1 GB — smaller vision model
./examples/run.sh mtmd gemma4           # raw mtmd.h API
./examples/run.sh tools                 # function calling
./examples/run.sh structured            # JSON-schema grammar
./examples/run.sh speculative           # prompt-lookup draft decoding
```

`run.sh` downloads the right GGUF on first run and is idempotent
afterwards. Without arguments it lists every example.

## Full table

| Example            | Model                                | Size    | What it shows                              |
| ------------------ | ------------------------------------ | ------- | ------------------------------------------ |
| [`quickstart`]     | `Qwen2.5-0.5B-Instruct-GGUF`         | ~400 MB | Load → tokenize → complete → chat → FIM     |
| [`simple`]         | any text GGUF                        | varies  | Plain text completion                        |
| [`streaming`]      | same as `quickstart`                 | ~400 MB | High-level token-by-token output             |
| [`chat`]           | instruct GGUF                        | varies  | One-shot chat with a builtin template        |
| [`stateful_chat`]  | same as `quickstart`                 | ~400 MB | REPL with growing history, `/clear`, `/save` |
| [`embeddings`]     | `bge-small-en-v1.5-gguf`             | ~30 MB  | Embedding extraction + L2 norm               |
| [`embedding_search`] | `bge-small-en-v1.5-gguf`           | ~30 MB  | Semantic search with cosine ranking          |
| [`reranker`]       | embedding GGUF                       | varies  | Bi-encoder ranking by cosine similarity      |
| [`vision`]         | Gemma 4 or LFM2.5-VL + mmproj        | ~1–5 GB | High-level `MtmdContext` vision chat         |
| [`mtmd`]           | Gemma 4 + mmproj                     | ~5 GB   | Raw `mtmd.h` API: bitmap → chunks → eval     |
| [`tools`]          | tool-aware instruct GGUF             | varies  | `ToolDefinition` + 5 `ToolParser` formats    |
| [`structured`]     | any text GGUF                        | varies  | `json_schema_grammar()` + JSON parsing       |
| [`speculative`]    | any text GGUF                        | varies  | `prompt-lookup` n-gram draft                 |

## Passing a different model

Every example accepts the GGUF path as the **first** positional
argument:

```bash
cargo run --release --bin run_quickstart -- models/llama-3.2-1b-instruct-q4_k_m.gguf
```

Vision examples take `<text.gguf> <mmproj.gguf> <image>`.

## Where to next?

Pick an example that matches what you want to build and read its
page — each links to the full `main.rs`.

[`examples/`]: https://github.com/DominguesM/llama-crab/tree/main/examples
[`quickstart`]: ./quickstart.md
[`simple`]: ./simple.md
[`streaming`]: ./streaming.md
[`chat`]: ./chat.md
[`stateful_chat`]: ./stateful_chat.md
[`embeddings`]: ./embeddings.md
[`embedding_search`]: ./embedding_search.md
[`reranker`]: ./reranker.md
[`vision`]: ./vision.md
[`mtmd`]: ./mtmd.md
[`tools`]: ./tools.md
[`structured`]: ./structured.md
[`speculative`]: ./speculative.md
