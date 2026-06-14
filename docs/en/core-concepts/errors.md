# Error handling

`llama-crab` reports every recoverable failure through the
[`LlamaError`] enum. This page explains the variants, when each one
is raised, and the patterns the safe API uses to surface them.

## The `LlamaError` enum

```rust
pub enum LlamaError {
    /// I/O error (model file not found, read failure, etc.).
    Io(std::io::Error),
    /// A GGUF parse error (file present, but invalid).
    Gguf(String),
    /// The model file could not be opened or loaded.
    ModelLoad(String),
    /// The context could not be created (out of memory, n_ctx too large).
    ContextCreate(String),
    /// Tokenisation failure.
    Tokenize(String),
    /// Detokenisation failure.
    Detokenize(String),
    /// Sampler creation or sampling failure.
    Sampling(String),
    /// Multimodal stack failure.
    Multimodal(String),
    /// The backend is not initialised (rare; usually caught at startup).
    BackendNotInitialised,
    /// A custom or unknown error returned by a C++ function.
    Other(String),
}
```

The enum implements `std::error::Error + Send + Sync`, so it composes
with `anyhow::Error` and `thiserror` without ceremony. The high-level
methods on `Llama` return `Result<T, LlamaError>`.

## Common variants in detail

### `LlamaError::Io`

Raised when the model file cannot be opened, when a tokenizer file
is missing, or when the disk cache hits an I/O error. The inner
`std::io::Error` carries the OS-level details:

```rust
match Llama::load(LlamaParams::new("missing.gguf")) {
    Ok(llama) => { /* … */ }
    Err(LlamaError::Io(e)) => eprintln!("file not found: {e}"),
    Err(e) => eprintln!("other error: {e}"),
}
```

Mitigations:

- Make sure the path is correct.
- Re-download the model with `scripts/download_models.sh <target>`.
- For relative paths, double-check the working directory of the
  binary at runtime (it might differ from your shell's `pwd`).

### `LlamaError::ModelLoad`

Raised when the file is open but the model cannot be loaded —
typically because of an unsupported architecture, a corrupt GGUF
file, or a version mismatch with the bundled `llama.cpp`.

Mitigations:

- Re-download the GGUF.
- Confirm the file is not truncated (`ls -lh` vs the expected size).
- Open an issue with the exact error message and the model
  identifier.

### `LlamaError::ContextCreate`

Raised when there isn't enough memory to allocate the KV cache. The
two levers you have are `n_ctx` (KV cache size) and `n_gpu_layers`
(GPU offload).

Mitigations, in order of impact:

1. Lower `n_ctx` (e.g. `4096 → 1024`).
2. Lower `n_gpu_layers` to keep more layers on the CPU (less VRAM,
   more RAM).
3. Switch to a more aggressive quant (`Q4_K_M → Q3_K_M → Q2_K`).
4. Switch backend (Metal → CPU when VRAM is the bottleneck).

### `LlamaError::Tokenize` and `LlamaError::Detokenize`

Raised when the input text contains bytes that cannot be tokenised,
or when the token id is out of range. Very rare in practice.

### `LlamaError::Multimodal`

Raised by the `mtmd` feature. Typical causes:

- The `mmproj` file does not match the text model.
- The image is too large to fit in the context.
- The `mtmd` feature is not enabled in the build (the type doesn't
  exist at all in that case).

### `LlamaError::BackendNotInitialised`

Raised only when the lower-level API is used without an active
`LlamaBackend` guard. The high-level `Llama::load` always
initialises the backend, so users of the high-level API will never
see this variant.

## Patterns for surfacing errors

### Map to user-facing messages

A common pattern is to convert the library error to a flat string for
display:

```rust
fn run() -> Result<String, String> {
    let mut llama = Llama::load(LlamaParams::new("model.gguf"))
        .map_err(|e| format!("could not load model: {e}"))?;
    let resp = llama.create_completion("Hello", 32)
        .map_err(|e| format!("completion failed: {e}"))?;
    Ok(resp.text)
}
```

### Use `anyhow` for application code

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let mut llama = Llama::load(LlamaParams::new("model.gguf"))
        .context("loading the model")?;
    let resp = llama.create_completion("Hello", 32)
        .context("running the completion")?;
    println!("{}", resp.text);
    Ok(())
}
```

### Use `thiserror` for library code

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Llama(#[from] llama_crab::LlamaError),

    #[error("invalid configuration: {0}")]
    Config(String),
}
```

### Recoverable vs unrecoverable

Most `LlamaError` variants are *recoverable* in the sense that the
process can keep running and respond to the next request. The two
exceptions are:

- `ModelLoad` (the model file is unusable; usually a misconfiguration
  that you only fix at startup).
- `ContextCreate` (memory pressure; usually a transient failure that
  retries do not help).

For a server, treat these two as fatal and exit the process so a
supervisor can restart it.

## Where to next?

- [Troubleshooting](../troubleshooting.md) — concrete recipes for
  the most common error messages.
- [Lifecycle](lifecycle.md) — what happens to in-flight requests
  when a worker hits an unrecoverable error.
- [Server](../server/index.md) — how the bundled HTTP server
  converts `LlamaError` into OpenAI-style HTTP status codes.

[`LlamaError`]: https://docs.rs/llama-crab/latest/llama_crab/error/enum.LlamaError.html
