# Troubleshooting

Answers to the issues most likely to bite new users. If your
problem isn't listed here, search the [GitHub issues] before
opening a new one.

## Build & compile

### CMake / clang errors when building `llama-crab-sys`

`llama-crab-sys` builds `llama.cpp` from source via CMake. You need:

- **CMake** ≥ 3.18
- **A C/C++ compiler** that supports C11 / C++17 (clang 14+, GCC
  11+, MSVC 2022)
- On macOS: Xcode Command Line Tools (`xcode-select --install`)
- On Linux: `build-essential` (Debian/Ubuntu) or the equivalent

If the build dies in `llama-crab-sys`, rerun with
`cargo build -vv` to see the underlying CMake error.

### First build is slow

Compiling every llama.cpp backend takes ~3 minutes on a 16-core
machine. Subsequent builds are cached. To speed up the first build,
disable backends you don't need:

```toml
llama-crab = { version = "0.1", default-features = false, features = ["openmp"] }
```

### `avx2 not detected` on older CPUs

Set `LLAMA_NO_AVX2=1` (or any of the `LLAMA_NO_*` flags documented
in `llama.cpp`) before `cargo build`:

```bash
LLAMA_NO_AVX2=1 cargo build --release
```

### Linker errors on macOS

```text
ld: library 'omp' not found
```

Install OpenMP via Homebrew:

```bash
brew install libomp
```

Then re-run `cargo build`. The crate's `build.rs` should pick up
the Homebrew location automatically.

### Linker errors on Linux + CUDA

```text
/usr/bin/ld: cannot find -lcudart
```

The CUDA toolkit is not in the library search path. Either install
the toolkit and ensure `/usr/local/cuda/lib64` is in
`LD_LIBRARY_PATH`, or use the `cuda-no-vmm` feature which links
against a smaller subset.

## Model loading

### `model not found` / `failed to open file`

The GGUF path you passed to `LlamaParams::new(...)` does not exist
or is not readable. Use the convenience script to download
known-good fixtures:

```bash
./scripts/download_models.sh smol    # ~400 MB text model
./scripts/download_models.sh bge     # ~30 MB embedding model
./scripts/download_models.sh gemma4  # ~5 GB vision model + projector
```

### `failed to allocate context` / out-of-memory

The model needs more memory than is available. Mitigations, in
order of impact:

1. Pick a smaller quant (`Q4_K_M` → `Q3_K_M` → `Q2_K`).
2. Lower `n_ctx` (e.g. `4096 → 2048`).
3. Reduce `n_gpu_layers` to keep more layers on CPU.
4. Switch backend (Metal → CPU when VRAM is the bottleneck).

### GPU not detected / `supports_gpu_offload()` returns false

You built without the GPU feature for your platform. On Linux/CUDA:

```toml
llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
```

On macOS the `metal` feature is on by default for `aarch64`. On
Intel macOS you must build CPU-only.

## Multimodal (mtmd)

### `mtmd.h` not found / `MtmdContext` not in scope

The multimodal API is gated by the `mtmd` cargo feature:

```toml
llama-crab = { version = "0.1", features = ["mtmd"] }
```

### `this projector does not support vision`

The `mmproj-*.gguf` file you loaded was trained for audio (or some
other modality). Check that the projector matches the text model
and the modality you want — both Gemma 4 and LFM2.5-VL ship separate
vision projectors.

## Performance

### Generation is slow

Common causes:

- **Not enough layer offload** — increase `n_gpu_layers`.
- **Large `n_ctx`** — long contexts cost memory and time per step.
- **CPU-only build on a Mac** — install the `metal` feature.
- **Debug build** — make sure you `cargo run --release`.

See the [performance tuning recipe](recipes/performance.md) for a
step-by-step guide.

### Embeddings are zero / similarity is `NaN`

You probably forgot `with_embeddings(true)` on the params, or
picked the wrong pooling type for the model. BGE / GTE / E5 expect
`PoolingType::Cls`; sentence-transformers-style models prefer
`Mean`.

### Speculative decoding has no speedup

The draft model and the main model have low agreement. Open-ended
creative writing rarely benefits. Try:

- A different draft model (smaller, faster).
- Skipping speculative decoding entirely.

## Server

### `address already in use` on startup

The port you picked is in use. Either pick another one:

```bash
cargo run -p llama-crab-server -- --port 8081
```

Or find the process holding the port and stop it:

```bash
lsof -i :8080
```

### Server returns 422 on chat requests

The request is well-formed JSON but the server rejected it. The
most common cause is `tool_choice` naming a function that is not
in the `tools` list. Check the server logs for the exact reason.

### Streaming cuts off mid-response

The HTTP client closed the connection early, or the worker
panicked. Enable `RUST_LOG=debug` for a more detailed log.

## Common error messages

| Error | Likely cause | Fix |
| --- | --- | --- |
| `LlamaError::Io(...)` | File not found, permission denied. | Check the path, the CWD, and the file mode. |
| `LlamaError::ModelLoad("unknown architecture")` | The GGUF is for a model family the bundled `llama.cpp` does not recognise. | Update `llama-crab` or use a different GGUF. |
| `LlamaError::ContextCreate("n_ctx too large")` | The KV cache is bigger than VRAM. | Lower `n_ctx` or pick a smaller quant. |
| `LlamaError::Tokenize("invalid utf-8")` | The prompt contains non-UTF-8 bytes. | Sanitise the prompt before tokenising. |
| `LlamaError::BackendNotInitialised` | You called a low-level API without an active `LlamaBackend`. | Hold a `LlamaBackend` guard for the lifetime of the model. |

## Still stuck?

- [Open an issue][GitHub issues] with the output of
  `cargo build -vv` and the output of `llama_crab::LlamaBackend`'s
  capability probes.
- For design questions, the [Discussions] tab is better than issues.
- The [Discord](https://discord.gg/llama-crab) (if it exists) is
  the fastest path to a real-time answer.

[GitHub issues]: https://github.com/DominguesM/llama-crab/issues
[Discussions]: https://github.com/DominguesM/llama-crab/discussions
