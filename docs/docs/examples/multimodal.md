---
title: Multimodal Examples
---

# Multimodal examples

Vision examples require two model files:

- a text GGUF,
- the matching `mmproj` projector GGUF.

The wrapper downloads both for supported targets.

## High-level vision

```bash
./examples/run.sh vision gemma4
./examples/run.sh vision lfm-vl
```

The binary loads the model, initializes `MtmdContext`, loads
`tests/fixtures/test_image.png`, tokenizes prompt plus image, evaluates chunks,
then samples an answer.

## Raw mtmd flow

```bash
./examples/run.sh mtmd gemma4
./examples/run.sh mtmd lfm-vl
```

This example stays closer to the `mtmd.h` flow and is useful when you need to
understand chunk evaluation and manual token feedback.

## LFM2.5-VL REPL

```bash
./examples/run.sh lfm_vl
```

Starts an interactive vision-language REPL using
`models/LFM2.5-VL-1.6B-Q4_K_M.gguf` and its projector. Commands:

- `/image <path>` changes the active image.
- `/clear` resets chat history.
- `/exit` quits.

You can also run one prompt directly by passing model, projector, image, and
prompt to `run_lfm_vl`.

## Multimodal HTTP

```bash
./examples/run.sh multimodal_http
```

Starts `llama-crab-server` with `--features mtmd`, the LFM text model, and the
matching projector. Send image content parts to `/v1/chat/completions`.
