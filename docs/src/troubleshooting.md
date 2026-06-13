# Troubleshooting / FAQ

Answers to the issues most likely to bite new users. If your problem
isn't listed here, search the [GitHub issues] before opening a new one.

## Build & compile

### CMake / clang errors when building `llama-crab-sys`

`llama-crab-sys` builds `llama.cpp` from source via CMake. You need:

- **CMake** ≥ 3.18
- **A C/C++ compiler** that supports C11 / C++17 (clang 14+, GCC 11+,
  MSVC 2022)
- On macOS: Xcode Command Line Tools (`xcode-select --install`)
- On Linux: `build-essential` (Debian/Ubuntu) or the equivalent

If the build dies in `llama-crab-sys`, rerun with
`cargo build -vv` to see the underlying CMake error.

### First build is slow

Compiling every llama.cpp backend takes ~3 min on a 16-core machine.
Subsequent builds are cached. To speed up the first build, disable
backends you don't need:

```toml
llama-crab = { version = "0.1", default-features = false, features = ["openmp"] }
```

### `avx2 not detected` on older CPUs

Set `LLAMA_NO_AVX2=1` (or any of the `LLAMA_NO_*` flags documented in
`llama.cpp`) before `cargo build`:

```bash
LLAMA_NO_AVX2=1 cargo build --release
```

## Model loading

### `model not found` / `failed to open file`

The GGUF path you passed to `LlamaParams::new(...)` does not exist or
is not readable. Use the convenience script to download known-good
fixtures:

```bash
./scripts/download_models.sh smol    # ~400 MB text model
./scripts/download_models.sh bge     # ~30 MB embedding model
./scripts/download_models.sh gemma4  # ~5 GB vision model + projector
```

### `failed to allocate context` / out-of-memory

The model needs more memory than is available. Mitigations, in order
of impact:

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
other modality). Check that the projector matches the text model and
the modality you want — both Gemma 4 and LFM2.5-VL ship separate
vision projectors.

## Performance

### Generation is slow

Common causes:

- **Not enough layer offload** — increase `n_gpu_layers`.
- **Large `n_ctx`** — long contexts cost memory and time per step.
- **CPU-only build on a Mac** — install the `metal` feature.
- **Debug build** — make sure you `cargo run --release`.

### Embeddings are zero / similarity is `NaN`

You probably forgot `with_embeddings(true)` on the params, or picked
the wrong pooling type for the model. BGE / GTE / E5 expect
`PoolingType::Cls`; sentence-transformers-style models prefer `Mean`.

## Still stuck?

- [Open an issue][GitHub issues] with the output of
  `cargo build -vv` and the output of `llama_crab::LlamaBackend`'s
  capability probes.
- For design questions, the [Discussions] tab is better than issues.

[GitHub issues]: https://github.com/DominguesM/llama-crab/issues
[Discussions]: https://github.com/DominguesM/llama-crab/discussions
