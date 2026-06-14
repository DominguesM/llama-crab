---
title: Troubleshooting Models
---

# Troubleshooting models

Most runtime issues come from a mismatch between model file, feature flags,
backend, context size, or command arguments. Start from the smallest model that
exercises the feature you need.

```bash
llama-crab-server --model /models/small-chat.gguf
```

## Model file not found

Download the model files required by the feature you are testing, then pass
their absolute paths to your application or server process.

```bash
llama-crab-server --model /absolute/path/to/model.gguf
```

## First build is slow

The first build compiles `llama-crab-sys` and the selected llama.cpp backends.
Subsequent builds should reuse Cargo artifacts unless features or source files
change.

## Allocation failures

If loading or context allocation fails:

1. Use a smaller quantized model.
2. Reduce `n_ctx`.
3. Reduce `n_batch` and `n_ubatch`.
4. Lower `n_gpu_layers`.
5. Disable optional features you do not need.

## Vision model does not see the image

Check all of these:

- The text GGUF and `mmproj` file come from the same model family.
- The binary was built with the `mtmd` feature.
- The prompt contains `default_media_marker()`.
- The image path is readable.
- The prompt template matches the model family.

Start with one known-good image and one short prompt before increasing image
size or context length:

```bash
llama-crab-server \
  --model /models/vision.gguf \
  --mmproj /models/mmproj.gguf
```

## Embeddings or reranking look wrong

Embedding models are more sensitive to pooling and backend details than simple
text completion. If vector dimensions, norms, ranking scores, or crashes differ
between environments:

1. Confirm the model path is the intended embedding or reranker GGUF.
2. Confirm embeddings are enabled with `with_embeddings(true)` or `--embeddings`.
3. For BGE-small semantic search, confirm `PoolingType::Cls`.
4. Compare backend features such as `metal`, `openmp`, or other accelerators.
5. Validate embeddings separately from text generation; a chat model pass does
   not prove that pooling, vector dimensions, or ranking scores are configured
   correctly.

## Server route returns disabled feature errors

`/v1/embeddings` requires the server to start with `--embeddings`.
`/v1/rerank` requires `--reranking`. Multimodal chat requires the `mtmd` feature
and `--mmproj`.

Examples:

```bash
llama-crab-server \
  --model models/bge-small-en-v1.5-q4_k_m.gguf \
  --embeddings

llama-crab-server \
  --model models/reranker.gguf \
  --reranking \
  --pooling rank
```

Repository wrappers and model download scripts are documented in
[Development](../contributing/development.md).
