# Caching & session state

`llama-crab` exposes two complementary persistence mechanisms:

- **Prompt / KV cache** — reuse the computed key-value tensors for a
  previously seen prompt *prefix*. Saves the prefill pass.
- **Session state** — serialize the entire KV state (and optionally
  the sampler / optimizer state) of a sequence to bytes.

## Prompt cache

Both implementations live in the [`cache`] module and follow the
[`Cache`] trait. They key on the **exact token sequence** and return
the longest matching prefix.

| Implementation | Feature      | Backing store                  |
| -------------- | ------------ | ------------------------------ |
| `RamCache`     | (always)     | In-process `BTreeMap`.          |
| `DiskCache`    | `disk-cache` | `sled` database on disk.        |

```rust,no_run
use llama_crab::cache::{Cache, CacheEntry, RamCache};

let cache = RamCache::new();
let tokens: Vec<llama_crab::token::LlamaToken> = Vec::new();

// After computing the KV state for `tokens` once:
cache.store(&tokens, CacheEntry { state: vec![], n_past: tokens.len() as i32 });

// On the next call with a longer sequence that starts with `tokens`:
if let Some(hit) = cache.lookup(&tokens) {
    println!("cache hit at position {}", hit.n_past);
}
```

For persistence across process restarts, enable `disk-cache` and
switch to `DiskCache::open(path)`:

```rust,no_run
# #[cfg(feature = "disk-cache")] {
use llama_crab::cache::DiskCache;

let cache = DiskCache::open("./.llama-crab-cache")?;
# let _ = cache;
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

The disk cache is safe to share across `Llama` instances in the same
process; only the last writer wins on key collisions.

### When the prompt cache helps

- **Chat REPLs** — each turn prepends the full history; a prefix hit
  skips re-evaluating every previous turn.
- **RAG** — embed the same document chunk multiple times.
- **Templated prompts** — system prompt + few-shot examples are
  repeated across queries.

The cache does **not** help when every call uses a fresh prompt with
no common prefix.

## Session state

Use `llama_state_get_data` / `llama_state_set_data` on the context to
serialize the full KV state (and the model's learned sampling state,
if any) into a byte buffer. This is what `RamCache` and `DiskCache`
store under the hood.

The high-level [`Llama`] orchestrator does not yet wrap these calls
behind a typed API; you can drive them through `llama.context()` if
you need byte-exact session snapshots (for example, to suspend and
resume a long agent session).

## Where to next?

- [Stateful chat](./stateful_chat.md) — multi-turn chat with growing
  history.
- [Reference](./reference.md)

[`cache`]: https://docs.rs/llama-crab/latest/llama_crab/cache/index.html
[`Cache`]: https://docs.rs/llama-crab/latest/llama_crab/cache/trait.Cache.html
[`Llama`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html
