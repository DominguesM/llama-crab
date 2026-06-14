# Caching & session state

`llama-crab` exposes two complementary persistence mechanisms:

- **Prompt / KV cache** — reuse the computed key-value tensors for
  a previously seen prompt *prefix*. Saves the prefill pass.
- **Session state** — serialize the entire KV state (and optionally
  the sampler / optimizer state) of a sequence to bytes.

Both are **manual** in v0.1.x. The high-level `create_completion`
helper clears KV sequence 0 before each call, so it always starts at
position 0 and does not consult the prompt cache automatically. If
you need to use these mechanisms, drive the lower-level API
directly — or use the worker-thread pattern that the
[server](../server/index.md) ships with.

## Prompt cache

Prompt-cache storage lives in the [`cache`] module. Both
implementations follow the [`Cache`] trait, which stores and looks
up opaque session bytes for token prefixes.

| Implementation | Cargo feature | Backing store |
| --- | --- | --- |
| [`RamCache`] | _(always)_ | In-process `BTreeMap`. |
| [`DiskCache`] | `disk-cache` | `sled` database on disk. |

### Using `RamCache`

```rust
use llama_crab::cache::{Cache, CacheEntry, RamCache};

let cache = RamCache::new();
let tokens: Vec<llama_crab::token::LlamaToken> = Vec::new();

// After computing the KV state for `tokens` once:
cache.store(&tokens, CacheEntry {
    state: vec![],
    n_past: tokens.len() as i32,
});

// On the next call with a longer sequence that starts with `tokens`:
if let Some(hit) = cache.lookup(&tokens) {
    println!("cache hit at position {}", hit.n_past);
}
```

The cache is keyed on the **exact token sequence** and returns the
longest matching prefix. If you ask for `[A, B, C, D]` and the
cache has `[A, B]`, the lookup returns the entry for `[A, B]` and
you can use it to skip the prefill of the first two tokens.

### Using `DiskCache`

For persistence across process restarts, enable the `disk-cache`
feature and switch to `DiskCache::open(path)`:

```toml title="Cargo.toml"
[dependencies]
llama-crab = { version = "0.1", features = ["disk-cache"] }
```

```rust
use llama_crab::cache::DiskCache;

let cache = DiskCache::open("./.llama-crab-cache")?;
```

The disk cache is safe to share across `Llama` instances in the same
process; only the last writer wins on key collisions.

### When the prompt cache helps

The cache pays off in three regimes:

- **Manual chat loops** — each turn prepends the full history; if
  you restore cached state yourself, a prefix hit can skip
  re-evaluating previous turns.
- **RAG** — embed the same document chunk multiple times across
  queries.
- **Templated prompts** — system prompt + few-shot examples are
  repeated across queries.

The cache **does not** help when:

- Every call uses a fresh prompt with no common prefix.
- You stay on the high-level `create_completion` path without
  manually restoring cached state.
- The model is small enough that prefill is not the bottleneck
  anyway.

## Session state

The `llama_state_get_data` and `llama_state_set_data` functions
serialise the full KV state (and the model's learned sampling
state, if any) of a sequence to a byte buffer. This is what
`RamCache` and `DiskCache` store under the hood.

The high-level [`Llama`] orchestrator does not yet wrap these calls
behind a typed API; you can drive them through `llama.context()` if
you need byte-exact session snapshots — for example, to suspend and
resume a long agent session.

```rust
// Pseudocode — see llama-crab-sys for the full surface.
let bytes = unsafe { llama.context().session_save(/*seq_id*/ 0)? };
std::fs::write("session.bin", bytes)?;

// Later, in a new process:
let bytes = std::fs::read("session.bin")?;
unsafe { llama.context().session_load(/*seq_id*/ 0, &bytes)? };
```

## Manual KV cache reuse

If you don't want to persist state across processes, you can still
benefit from the KV cache by managing the sequence yourself:

```rust
use llama_crab::batch::LlamaBatch;
use llama_crab::token::LlamaToken;

// 1. Load a long system prompt.
let system_tokens = llama.model().tokenize(SYSTEM_PROMPT, true, true)?;
let mut batch = LlamaBatch::new(system_tokens.len(), 1);
batch.add_sequence(&system_tokens, 0, false);
batch.prepare();
llama.context().decode(&batch)?;
let mut n_past = system_tokens.len() as i32;

// 2. Each turn, decode only the new user message.
loop {
    let user_tokens = llama.model().tokenize(/* prompt */, false, true)?;
    let mut batch = LlamaBatch::new(user_tokens.len(), 1);
    batch.add_sequence(&user_tokens, 0, false);
    batch.prepare();
    llama.context().decode(&batch)?;
    n_past += user_tokens.len() as i32;

    // 3. Sample, append, loop.
    // ...
}
```

The KV cache is *not* cleared between turns, so turn `N+1` only
pays the prefill cost of the new user message. This is the pattern
used by the [`stateful_chat` example](../examples/stateful-chat.md).

## Caching pitfalls

| Pitfall | What goes wrong | Fix |
| --- | --- | --- |
| Two processes share a `DiskCache` | Concurrent writers corrupt the database. | Scope the cache to one process or add a file lock. |
| Cache key uses raw text | A typo causes a miss. | Tokenise the prompt first, key on the token ids. |
| Cache survives a model upgrade | Old KV state is incompatible with the new model. | Include the GGUF hash in the cache key, or wipe on upgrade. |
| Cache grows unboundedly | Out-of-memory crash. | Periodically evict old entries or use `DiskCache`. |

## Where to next?

- [Stateful chat](../features/stateful-chat.md) — multi-turn chat
  with a growing history that doesn't replay the entire context.
- [Server](../server/index.md) — the reference implementation of
  the worker-thread pattern.
- [Caching example](https://github.com/DominguesM/llama-crab/tree/main/examples/stateful_chat) — a runnable program that
  uses manual KV cache reuse.

[`Cache`]: https://docs.rs/llama-crab/latest/llama_crab/cache/trait.Cache.html
[`RamCache`]: https://docs.rs/llama-crab/latest/llama_crab/cache/struct.RamCache.html
[`DiskCache`]: https://docs.rs/llama-crab/latest/llama_crab/cache/struct.DiskCache.html
[`cache`]: https://docs.rs/llama-crab/latest/llama_crab/cache/index.html
[`Llama`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html
