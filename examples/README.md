# `llama-crab` examples

A collection of self-contained programs that demonstrate every public
feature of the `llama-crab` library. Each example lives in its own
Cargo crate (one `[[bin]]` per crate) so you can copy-paste the parts
you need into your own project without dragging in the rest.

## TL;DR — one-command run

The repo ships with a thin `run.sh` wrapper that downloads the model
required by the example (if missing) and then builds + runs it:

```bash
./examples/run.sh quickstart            # 0.5B text-only model
./examples/run.sh stateful_chat         # interactive multi-turn REPL
./examples/run.sh embedding_search      # BGE-small + cosine ranking
./examples/run.sh vision gemma4         # Gemma 4 text + image
./examples/run.sh vision lfm-vl         # LFM2.5-VL 1.6B text + image
./examples/run.sh lfm_vl                # REPL against the LFM VL model
./examples/run.sh server_lfm            # llama-crab-server w/ LFM2.5-VL
./examples/run.sh tauri_chat_lfm        # Tauri chat app w/ automatic download progress
./examples/run.sh multimodal_http       # mtmd-enabled HTTP chat server
./examples/run.sh rerank                # HTTP reranking server
./examples/run.sh tools                 # function-calling demo
./examples/run.sh tool_calls_qwen       # Qwen tool-call demo alias
./examples/run.sh structured            # JSON-schema grammar
```

If the model is already in `./models/` the download is a no-op.

Without an example name the script prints a list of everything it knows.

## Directory layout

```
examples/
├── run.sh                       # one-shot download + run wrapper
├── README.md                    # this file
│
├── quickstart/                  # smallest end-to-end demo (text only)
├── streaming/                   # high-level token-by-token output
├── stateful_chat/               # interactive multi-turn chat REPL
├── embedding_search/            # cosine-similarity semantic search
│
├── simple/                      # one-shot text completion
├── chat/                        # one-shot chat completion
├── embeddings/                  # raw embedding extraction
├── reranker/                    # embedding-based ranking demo
├── speculative/                 # speculative decoding
├── tools/                       # function-calling + tool parser
├── structured/                  # JSON-schema constrained decoding
├── mtmd/                        # multimodal (vision) via mtmd.h
├── vision/                      # vision via the high-level API
├── lfm_vl_vision/               # LFM2.5-VL multimodal REPL
├── server_lfm/                  # llama-crab-server wired for LFM2.5-VL
└── tauri-chat-lfm/              # one-page Tauri chat app for LFM2.5 350M
```

## Per-example guide

| Example              | Model                                  | Size  | What it shows |
| -------------------- | -------------------------------------- | ----- | ------------- |
| `quickstart`         | `Qwen/Qwen2.5-0.5B-Instruct-GGUF`      | ~400 MB | Load → tokenize → complete → chat → FIM |
| `streaming`          | same as `quickstart`                   | ~400 MB | High-level token-by-token output |
| `stateful_chat`      | same as `quickstart`                   | ~400 MB | REPL with growing history, `/clear`, `/save` |
| `embedding_search`   | `CompendiumLabs/bge-small-en-v1.5-gguf` | ~30 MB | L2-normalized embeddings + cosine ranking |
| `simple`             | any text GGUF                          | varies | Low-level decode loop with custom sampler chain |
| `chat`               | instruct-tuned GGUF                    | varies | Built-in chat templates + `BuiltinTemplate::ChatML` |
| `embeddings`         | an embedding GGUF                      | varies | `embed()`, L2 normalize, raw `llama_get_embeddings` |
| `reranker`           | an embedding GGUF                      | varies | embedding cosine ranking |
| `speculative`        | draft + target GGUF                    | varies | `prompt-lookup` and small-model draft decoding |
| `tools`              | a tool-aware instruct GGUF             | varies | `ToolDefinition` + 5 `ToolParser` formats |
| `structured`         | any text GGUF                          | varies | `json_schema_grammar()` + `Sampler::grammar` |
| `mtmd`               | `lmstudio-community/gemma-4-E4B-it-GGUF` | ~5 GB | Raw `mtmd.h` API: bitmap → chunks → eval |
| `vision`             | same model (or `LFM2.5-VL-1.6B`)       | ~5 GB | High-level `MtmdContext` API |
| `lfm_vl`             | `unsloth/LFM2.5-VL-1.6B-GGUF`          | ~1 GB | LFM2.5-VL 1.6B multimodal REPL |
| `server_lfm`         | `unsloth/LFM2.5-VL-1.6B-GGUF`          | ~1 GB | Boots `llama-crab-server` pre-wired for LFM2.5-VL |
| `tauri_chat_lfm`     | `LiquidAI/LFM2.5-350M-GGUF`            | ~229 MB | One-page Tauri chat app with automatic download progress |
| `multimodal_http`    | same as `server_lfm`                   | ~1 GB | Boots `llama-crab-server --features mtmd -- --mmproj ...` |
| `rerank`             | `turingevo/bge-reranker-base-Q4_K_M-GGUF` | varies | Boots `llama-crab-server --reranking --pooling rank` |

The two vision examples both need a vision-language GGUF **and** its
`mmproj-*.gguf` projector file. `download_models.sh` downloads both.
The `tauri_chat_lfm` example is different: the Tauri app downloads its
fixed LFM2.5 350M GGUF on first use and shows download progress
(`progresso do download`) in the app window.

## Running an example by hand

The convenience script does three things:

1. Resolves which model the example needs.
2. Calls `./scripts/download_models.sh <target>` (idempotent — skips files already on disk).
3. Calls `cargo run --release --bin <name>`.

You can replicate the same steps manually:

```bash
# 1. Download a model (only the first time).
./scripts/download_models.sh smol        # text-only models
./scripts/download_models.sh gemma4      # text + mmproj
./scripts/download_models.sh bge         # embeddings
./scripts/download_models.sh bge-reranker # rerank server
./scripts/download_models.sh test-image  # synthetic PNG fixture

# 2. Run the example.
cargo run --release --bin run_quickstart
cargo run --release --bin run_streaming
cargo run --release --bin run_chat
cargo run --release --bin run_embeddings
```

Available download targets (see `./scripts/download_models.sh all` for
the full list):

| Target        | Repo                                            | Files |
| ------------- | ----------------------------------------------- | ----- |
| `smol`        | `Qwen/Qwen2.5-0.5B-Instruct-GGUF`               | 1 × GGUF |
| `gemma4`      | `lmstudio-community/gemma-4-E4B-it-GGUF`        | 1 × text GGUF + 1 × mmproj GGUF |
| `lfm-vl`      | `unsloth/LFM2.5-VL-1.6B-GGUF`                   | 1 × text GGUF + 1 × mmproj GGUF |
| `bge`         | `CompendiumLabs/bge-small-en-v1.5-gguf`         | 1 × GGUF |
| `bge-reranker` | `turingevo/bge-reranker-base-Q4_K_M-GGUF`      | 1 × GGUF |
| `test-image`  | (synthetic, no download)                        | `tests/fixtures/test_image.png` |
| `all`         | everything                                      | ~7 GB total |

## Using a different model

Every example accepts the GGUF path as the **first** positional
argument. The default is whatever the example's `run.sh` target
downloads, so the simplest override is:

```bash
cargo run --release --bin run_quickstart -- models/llama-3.2-1b-instruct-q4_k_m.gguf
```

For vision examples, the first positional argument of the binary is
the text GGUF and the second is the `mmproj` GGUF:

```bash
cargo run --release --bin run_mtmd -- \
  models/gemma-4-E4B-it-Q4_K_M.gguf \
  models/mmproj-gemma-4-E4B-it-BF16.gguf \
  tests/fixtures/test_image.png "What is in this image?"
```

## Downloading with `curl` instead of `hf`

`download_models.sh` prefers `huggingface-cli` (`hf download ...`) but
falls back to `curl` automatically. To force curl-only, install
nothing — just run the script. To force the HF CLI:

```bash
pip install --upgrade huggingface_hub
hf auth login   # optional, only needed for gated repos
```

## Troubleshooting

* **`model not found`** — The example is looking for a file in `./models/`.
  Run `./scripts/download_models.sh <target>` or pass the path explicitly.
* **`failed to allocate context`** — The model needs more memory than is
  available. Try a smaller quant (`Q4_K_M` → `Q3_K_M` → `Q2_K`) or
  reduce `n_ctx` / `n_gpu_layers`.
* **`avx2 not detected`** on older CPUs — set
  `LLAMA_NO_AVX2=1 cargo run --release …` (or the equivalent llama.cpp
  flag) before running the example.
* **First run is slow** — the build of the `llama-crab-sys` crate
  compiles all 17 llama.cpp backends (~3 min on a 16-core machine).
  Subsequent runs are cached.
* **The first download is large** — `gemma4` is ~5 GB. Start with
  `smol` (~400 MB) and the `quickstart` example to confirm everything
  works.

## What the integration tests use

`../crates/llama-crab/tests/` contains the same examples in test form:

* `crates/llama-crab/tests/gemma4_text.rs`        — text-only generation, no vision.
* `crates/llama-crab/tests/gemma4_vision.rs`      — Gemma 4 + mmproj + test image.
* `crates/llama-crab/tests/lfm_vl_vision.rs`      — LFM2.5-VL + mmproj + test image.

They skip cleanly when the model is not present, so a fresh clone can
build the test binary without owning the model.

## Adding a new example

The boilerplate for a new example crate is ~15 lines:

```toml
# examples/my_example/Cargo.toml
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
llama-crab = { path = "../../crates/llama-crab", version = "0.1.0" }
anyhow = "1"
```

```rust
// examples/my_example/src/main.rs
use anyhow::Result;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<()> {
    let mut llama = Llama::load(LlamaParams::new("models/your.gguf"))?;
    let resp = llama.create_completion("Hello!", 32)?;
    print!("{}", resp.text);
    Ok(())
}
```

Then add `examples/my_example` to the `members = [...]` list in
`../Cargo.toml` and a row to the table above.
