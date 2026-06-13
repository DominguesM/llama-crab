# `streaming` - High-level token streaming

Use `Llama::create_completion_stream` when you want synchronous
token-by-token output while still receiving the final `Completion`.
The callback receives text chunks as they become available and returns
`StreamControl::Continue` or `StreamControl::Stop`.

```rust,no_run
use std::io::{self, Write};

use llama_crab::{CompletionOptions, Llama, LlamaParams, StreamControl};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(512))?;
    let prompt = "Write one short sentence about Rust.";
    let mut stdout = io::stdout().lock();

    let mut write_error: Option<io::Error> = None;
    let completion = llama.create_completion_stream(
        prompt,
        CompletionOptions::new(64).with_stop_sequence("\n\n"),
        |chunk| {
            if let Err(err) = write!(stdout, "{}", chunk.text).and_then(|_| stdout.flush()) {
                write_error = Some(err);
                return StreamControl::Stop;
            }
            StreamControl::Continue
        },
    )?;

    if let Some(err) = write_error {
        return Err(err.into());
    }
    writeln!(stdout)?;

    let _ = completion;
    Ok(())
}
```

The callback cannot return `Result`, so capture I/O errors and return
`StreamControl::Stop`; after the stream returns, propagate the captured
error.

```rust,no_run
# use std::io::{self, Write};
# use llama_crab::{CompletionOptions, Llama, LlamaParams, StreamControl};
# let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(512))?;
let mut stdout = io::stdout().lock();
let mut write_error: Option<io::Error> = None;

let completion = llama.create_completion_stream(
    "Write one short sentence about Rust.",
    CompletionOptions::new(64),
    |chunk| {
        if let Err(err) = write!(stdout, "{}", chunk.text).and_then(|_| stdout.flush()) {
            write_error = Some(err);
            return StreamControl::Stop;
        }
        StreamControl::Continue
    },
)?;

if let Some(err) = write_error {
    return Err(err.into());
}
# let _ = completion;
# Ok::<(), Box<dyn std::error::Error>>(())
```

For quick demos where stdout errors are not important, the callback can
ignore them:

```rust,no_run
# use std::io::{self, Write};
# use llama_crab::{CompletionOptions, Llama, LlamaParams, StreamControl};
# let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(512))?;
let mut stdout = io::stdout().lock();
llama.create_completion_stream(
    "Write one short sentence about Rust.",
    CompletionOptions::new(64),
    |chunk| {
        let _ = write!(stdout, "{}", chunk.text);
        let _ = stdout.flush();
        StreamControl::Continue
    },
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Run with the small text model:

```bash
./examples/run.sh streaming
```

This streaming helper uses the same high-level completion path as
`create_completion`: it clears sequence 0 before each call and does
not enable automatic prompt-cache reuse between calls. For custom
sampling, batching, or manual KV/session reuse, use the lower-level
context, batch and sampler APIs directly.

## Full source

[`examples/streaming/src/main.rs`][src].

[src]: https://github.com/DominguesM/llama-crab/tree/main/examples/streaming/src/main.rs
