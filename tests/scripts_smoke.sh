#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

run_dry() {
  LLAMA_CRAB_SKIP_DOWNLOAD=1 LLAMA_CRAB_DRY_RUN=1 ./examples/run.sh "$@"
}

out="$(run_dry chat)"
grep -Fq "cargo run --release --bin chat -- models/qwen2.5-0.5b-instruct-q4_k_m.gguf" <<<"$out" \
  || fail "chat dry-run did not include the smol model path"

out="$(run_dry vision lfm-vl)"
grep -Fq "cargo run --release --bin vision -- models/LFM2.5-VL-1.6B-Q4_K_M.gguf models/LFM2.5-VL-1.6B-mmproj-BF16.gguf tests/fixtures/test_image.png" <<<"$out" \
  || fail "vision dry-run did not include model, mmproj and image paths"

out="$(run_dry embedding_search 'What is safe systems programming?')"
grep -Fq "cargo run --release --bin run_embeddings -- models/bge-small-en-v1.5-q4_k_m.gguf What is safe systems programming?" <<<"$out" \
  || fail "embedding_search dry-run did not preserve extra query args"

out="$(LLAMA_CRAB_DRY_RUN=1 ./scripts/download_models.sh smol)"
grep -Fq "hf download Qwen/Qwen2.5-0.5B-Instruct-GGUF qwen2.5-0.5b-instruct-q4_k_m.gguf" <<<"$out" \
  || fail "download dry-run did not use hf download for smol"

out="$(printf 'y\n' | ./scripts/clean.sh --dry-run)"
grep -Fq "dry-run" <<<"$out" || fail "clean dry-run did not complete"

echo "scripts smoke tests passed"
