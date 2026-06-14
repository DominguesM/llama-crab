# Crate layout

The `llama-crab` workspace contains two library crates and one
binary crate. This page is the map you need to navigate the source
tree.

```
llama-crab/
├── crates/
│   ├── llama-crab-sys/      # Raw FFI (bindgen + CMake)
│   ├── llama-crab/          # 100 % safe Rust API
│   │   ├── backend          # LlamaBackend + NumaStrategy
│   │   ├── model            # LlamaModel + LlamaModelParams
│   │   ├── context          # LlamaContext + params + embeddings + session
│   │   ├── batch            # LlamaBatch
│   │   ├── sampling         # LlamaSampler + SamplerChain (17 strategies)
│   │   ├── chat             # ChatMessage + templates + tool calling
│   │   ├── speculative      # PromptLookupDecoding + speculative_decode
│   │   ├── multimodal       # MtmdContext + MtmdBitmap (feature mtmd)
│   │   ├── cache            # RamCache + DiskCache
│   │   ├── json_schema      # JSON-Schema -> GBNF
│   │   ├── high_level       # Llama orchestrator + create_completion
│   │   ├── error            # LlamaError enum
│   │   └── log              # tracing integration
│   └── llama-crab-server/   # HTTP binary built on top of llama-crab
├── packages/
│   ├── core/                # Reserved for @llama-crab/core
│   └── tauri/               # Reserved for @llama-crab/tauri
├── examples/                # Runnable example crates
└── docs/                    # User guide and website source
```

## `llama-crab-sys`

The low-level FFI package. Contains:

- The `wrapper.h` header that selects the llama.cpp C API to
  expose.
- The `build.rs` that runs `bindgen` against the wrapper and `cmake`
  against the bundled `llama.cpp/` source tree.
- The generated `bindings.rs` (do not edit — it is regenerated on
  every build).
- A handful of safe wrappers for the most-used FFI calls, so
  consumers of the safe crate can stay in safe Rust.

Most applications should depend on `llama-crab` instead. Use
`llama-crab-sys` only when you need direct access to a llama.cpp
symbol that the safe crate does not (yet) wrap.

## `llama-crab`

The safe high-level API. Each module is documented on
[docs.rs/llama-crab](https://docs.rs/llama-crab); this page gives
the high-level responsibilities.

| Module | Responsibility |
| --- | --- |
| `backend` | `LlamaBackend`, the GGML backend handle, NUMA placement. |
| `model` | `LlamaModel` — the loaded weights + tokenizer + metadata. |
| `context` | `LlamaContext` — the KV cache + the forward-pass driver. |
| `batch` | `LlamaBatch` — the typed batch builder for `decode`. |
| `sampling` | `LlamaSampler` + `SamplerChain` — 17 sampling strategies. |
| `chat` | `ChatMessage`, `Role`, `BuiltinTemplate`, `render_builtin`, the tool-call parser. |
| `speculative` | `PromptLookupDecoding`, `DraftModel` trait, `speculative_decode`. |
| `multimodal` (feature `mtmd`) | `MtmdContext`, `MtmdBitmap`, `MtmdInputText`, `chunks.eval`. |
| `cache` | `RamCache`, `DiskCache` (feature `disk-cache`), the `Cache` trait. |
| `json_schema` | The JSON-Schema → GBNF converter. |
| `high_level` | `Llama`, the orchestrator that owns the model + context + sampler state. |
| `error` | `LlamaError` — the single error type for the safe API. |
| `log` | `tracing` integration — info/warn logs around model load. |

### The `Llama` orchestrator

The most-used type in the safe API. It owns:

- A `LlamaBackend` guard (initialised on `Llama::load`).
- A `LlamaModel` (the weights + tokenizer).
- A `LlamaContext` (the KV cache).
- A default `SamplerChain` (greedy by default, configurable through
  `Llama::create_*_with_sampler`).

The high-level methods (`create_completion`, `create_chat_completion`,
`embed`, `rerank`, `complete_infill`) hide the loop illustrated in
the [architecture guide](../core-concepts/architecture.md) behind a
single function call.

## `llama-crab-server`

A thin HTTP binary built on top of the safe API. It keeps inference
inside the Rust binding and uses a worker thread that owns the model
and context. See the [server guide](../server/index.md) for the
runtime shape and the API surface.

## Examples

The [`examples/`](https://github.com/DominguesM/llama-crab/tree/main/examples)
directory contains 14 self-contained Cargo crates that exercise
every public feature. Each one is a `[[bin]]` of its own crate and
can be copied into another project without modification. See the
[examples index](../examples/index.md) for the table.

## Integration tests

The [`crates/llama-crab/tests/`](https://github.com/DominguesM/llama-crab/tree/main/crates/llama-crab/tests)
directory contains the same examples in test form. They skip cleanly
when the model is not on disk, so a fresh clone can build the test
binary without owning the model.

| Test | Model | What it verifies |
| --- | --- | --- |
| `gemma4_text.rs` | Gemma 4 (text-only) | Text generation, no vision. |
| `gemma4_vision.rs` | Gemma 4 + mmproj + test image | The high-level `MtmdContext` API. |
| `lfm_vl_vision.rs` | LFM2.5-VL + mmproj + test image | Multimodal on a smaller model. |

## Where to next?

- [Architecture](../core-concepts/architecture.md) — the data flow
  inside a single forward pass.
- [Cargo features](cargo-features.md) — what each feature toggles.
- [API on docs.rs](https://docs.rs/llama-crab) — the auto-generated
  rustdoc.
