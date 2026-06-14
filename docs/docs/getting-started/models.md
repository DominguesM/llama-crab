---
title: Models
---

# Models

`llama-crab` consumes GGUF files. Text-only examples need one model file.
Multimodal examples also need an `mmproj` projector file that matches the text
model.

## Choosing files

Download GGUF files from a source you trust and keep the matching projector file
next to multimodal models. Pass absolute paths in production services so the
working directory does not affect model loading.

Repository download helpers for contributors are covered in
[Development](../contributing/development.md).

## Storage

Large model downloads can exhaust local disk quickly. Check free space before
running the vision examples or broad validation scripts.

```bash
df -h .
```

## Matching models to examples

| Workload | Model type |
| --- | --- |
| Completion and chat | Instruct or base text GGUF |
| Embeddings | Embedding GGUF loaded with embeddings enabled |
| Reranking | Rank/cross-encoder model and rank pooling |
| Vision | Text GGUF plus matching `mmproj` |
