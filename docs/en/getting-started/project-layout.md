# Project layout

A typical `llama-crab` project has three moving parts: a Rust binary
(or library), a GGUF model file, and — optionally — an `mmproj`
projector for vision-language models. This page shows a layout that
keeps the moving parts easy to find and easy to swap.

## Recommended layout

```
my-app/
├── Cargo.toml
├── src/
│   └── main.rs
├── models/                  # GGUF model files (not committed)
│   ├── qwen2.5-7b-instruct-q4_k_m.gguf
│   └── mmproj-qwen2.5-vl-q8_0.gguf
├── prompts/                 # optional: chat templates, system prompts
│   └── system.txt
└── tests/
    └── integration.rs
```

The `models/` directory is intentionally **not** committed: models
are large binaries that you download out-of-band. Add it to
`.gitignore`:

```gitignore title=".gitignore"
/models/
```

## Downloading models

The repository ships a `scripts/download_models.sh` helper that
fetches known-good fixtures from Hugging Face. Copy it into your
own project, or call it from a setup script:

=== "Fetch a small chat model"

    ```bash
    ./scripts/download_models.sh smol
    ```

=== "Fetch an embedding model"

    ```bash
    ./scripts/download_models.sh bge
    ```

=== "Fetch a vision model + projector"

    ```bash
    ./scripts/download_models.sh gemma4
    ```

If you'd rather use the `huggingface_hub` CLI directly:

=== "Python (huggingface_hub)"

    ```bash
    pip install -U "huggingface_hub[cli]"
    hf download Qwen/Qwen2.5-0.5B-Instruct-GGUF \
        qwen2.5-0.5b-instruct-q4_k_m.gguf \
        --local-dir models
    ```

=== "curl fallback"

    ```bash
    mkdir -p models
    curl -L "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf" \
        -o models/qwen2.5-0.5b-instruct-q4_k_m.gguf
    ```

## Cargo.toml boilerplate

A minimal, production-friendly `Cargo.toml`:

```toml title="Cargo.toml"
[package]
name        = "my-app"
version     = "0.1.0"
edition     = "2021"
rust-version = "1.88"

[dependencies]
llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow     = "1"

[profile.release]
opt-level     = 3
lto           = "thin"
codegen-units = 1
strip         = "debuginfo"
```

!!! tip "Pin the model path"

    Prefer to load the model path from an environment variable or a
    CLI flag rather than hard-coding it into the binary. It makes the
    same binary runnable against multiple GGUF files in CI.

## A single-binary skeleton

```rust title="src/main.rs"
use std::env;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_path = env::args()
        .nth(1)
        .or_else(|| env::var("LLAMA_CRAB_MODEL").ok())
        .unwrap_or_else(|| "models/qwen2.5-0.5b-instruct-q4_k_m.gguf".into());

    let mut llama = Llama::load(
        LlamaParams::new(&model_path)
            .with_n_ctx(2048)
            .with_n_threads(4),
    )?;

    let resp = llama.create_completion("Hello, world!", 64)?;
    print!("{}", resp.text);
    Ok(())
}
```

## Multi-binary projects

If you want several binaries sharing the same model loader, split the
code into a library and a thin binary:

```
my-app/
├── Cargo.toml
├── src/
│   ├── lib.rs            # pub fn load_model() -> Result<Llama, _>
│   ├── chat.rs
│   └── server.rs
├── src/bin/
│   ├── chat.rs           # uses my_app::chat
│   └── server.rs         # uses my_app::server
└── models/
```

`Cargo.toml`:

```toml title="Cargo.toml"
[package]
name        = "my-app"
version     = "0.1.0"
edition     = "2021"

[lib]
name = "my_app"
path = "src/lib.rs"

[[bin]]
name = "chat"
path = "src/bin/chat.rs"

[[bin]]
name = "server"
path = "src/bin/server.rs"

[dependencies]
llama-crab = { version = "0.1", features = ["metal", "openmp"] }
```

## Working with the example runner

The repository ships a `examples/run.sh` wrapper that downloads the
right model, builds the right binary and runs it. You can read the
script to learn how to wire your own:

```bash
# What it does:
# 1. Resolve the example name → "download-target|binary-name".
# 2. Call ./scripts/download_models.sh <target>.
# 3. Call cargo run --release --bin <name>.
```

The two environment variables worth knowing about are:

- `LLAMA_CRAB_SKIP_DOWNLOAD=1` — skip the model download step.
- `LLAMA_CRAB_DRY_RUN=1` — print the command without executing it.

## Where to next?

- [Cargo features](cargo-features.md) — tune the build for your
  target.
- [Examples index](../examples/index.md) — copy-paste a starter
  program that already runs.
- [Server](../server/index.md) — if your project needs an HTTP
  endpoint, use `llama-crab-server` directly.
