#!/usr/bin/env bash
# Download the GGUF models used by `llama-crab`'s examples and integration tests.
#
# Usage:
#   ./scripts/download_models.sh                       # download everything
#   ./scripts/download_models.sh smol                  # tiny text-only (~400 MB)
#   ./scripts/download_models.sh gemma4                 # Gemma 4 text + mmproj (~5 GB)
#   ./scripts/download_models.sh lfm-vl                # LFM2.5-VL text + mmproj (~2 GB)
#   ./scripts/download_models.sh lfm-text              # LFM2.5-VL text only (~1 GB)
#   ./scripts/download_models.sh bge                    # embedding model (~25 MB)
#   ./scripts/download_models.sh bge-reranker           # cross-encoder rerank (~220 MB)
#   ./scripts/download_models.sh test-image             # the synthetic PNG fixture
#
# Models land in `./models/` (the conventional path the examples look at).
# The script is idempotent: files that already exist are skipped.
#
# Requirements: `hf` (`pip install --upgrade huggingface_hub`).
# Set `HF_TOKEN` to access private/gated models.
#
# The first run is slow (multiple GB). Subsequent runs are instant.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODELS_DIR="$ROOT/models"
FIXTURE_DIR="$ROOT/tests/fixtures"
mkdir -p "$MODELS_DIR" "$FIXTURE_DIR"

# ---- helpers --------------------------------------------------------------

have() { command -v "$1" >/dev/null 2>&1; }

dry_run() { [[ "${LLAMA_CRAB_DRY_RUN:-0}" == "1" ]]; }

hf_cli() {
  if have hf && hf --help >/dev/null 2>&1; then
    HF_CLI=(hf)
    return 0
  fi
  local py="${PYTHON:-python}"
  if command -v "$py" >/dev/null 2>&1 && "$py" -m huggingface_hub.cli.hf --help >/dev/null 2>&1; then
    HF_CLI=("$py" -m huggingface_hub.cli.hf)
    return 0
  fi
  return 1
}

# Get the size of a file in bytes (cross-platform).
fsize() {
  if have stat; then
    stat -f%z "$1" 2>/dev/null || stat -c%s "$1" 2>/dev/null || echo 0
  else
    wc -c <"$1" | tr -d ' '
  fi
}

download_hf() {
  local repo="$1" filename="$2" dest="$3"
  local dir
  dir="$(dirname "$dest")"
  local downloaded="$dir/$filename"
  display_cmd=(hf download "$repo" "$filename" --local-dir "$dir")
  cmd=(hf download "$repo" "$filename" --local-dir "$dir")
  if [[ -n "${HF_TOKEN:-}" ]]; then
    display_cmd+=(--token "$HF_TOKEN")
    cmd+=(--token "$HF_TOKEN")
  fi
  if dry_run; then
    echo "↓ $(basename "$dest")  ($filename)"
    echo "${display_cmd[*]}"
    if [[ "$downloaded" != "$dest" ]]; then
      echo "mv $downloaded $dest"
    fi
    return 0
  fi

  if [[ -f "$dest" ]] && [[ "$(fsize "$dest")" -gt 1000000 ]]; then
    echo "✓ $(basename "$dest")  ($(numfmt "$(fsize "$dest")"))"
    return 0
  fi
  if ! hf_cli; then
    echo "ERROR: need the Hugging Face CLI. Install with: pip install --upgrade huggingface_hub" >&2
    return 1
  fi
  cmd=("${HF_CLI[@]}" download "$repo" "$filename" --local-dir "$dir")
  if [[ -n "${HF_TOKEN:-}" ]]; then
    cmd+=(--token "$HF_TOKEN")
  fi

  # Drop any leftover `.part` file from a previous curl-based interrupted run.
  [[ -f "$dest.part" ]] && rm -f "$dest.part"

  echo "↓ $(basename "$dest")  ($filename)"
  "${cmd[@]}"
  if [[ "$downloaded" != "$dest" ]]; then
    mv "$downloaded" "$dest"
  fi
}

# Pretty-print a byte count.
numfmt() {
  # Use a here-string for portability and assign to awk variables via `-v`
  # because BSD awk on macOS does not read stdin inside `BEGIN` blocks.
  awk -v bytes="$1" 'BEGIN {
    n = bytes
    split("B KB MB GB TB", u, " ")
    i = 1
    while (n >= 1024 && i < 5) { n /= 1024; i++ }
    printf "%.1f %s", n, u[i]
  }'
}

# ---- test image (256×256 PNG) --------------------------------------------

test_image() {
  if [[ -f "$FIXTURE_DIR/test_image.png" ]]; then
    echo "✓ test_image.png  ($(numfmt "$(fsize "$FIXTURE_DIR/test_image.png")"))"
    return
  fi
  echo "↓ generating test_image.png (256×256 red/blue checker)"
  if dry_run; then
    echo "python3 - <<'PY' > $FIXTURE_DIR/test_image.png"
    return
  fi
  python3 - <<'PY'
import struct, zlib, os
W = H = 256
def chunk(tag, data):
    crc = zlib.crc32(tag + data) & 0xffffffff
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)
sig = b"\x89PNG\r\n\x1a\n"
ihdr = chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 2, 0, 0, 0))
rows = []
for y in range(H):
    row = b"\x00"  # filter byte
    for x in range(W):
        r = 200 if ((x // 32) + (y // 32)) % 2 == 0 else 30
        g = 30  if ((x // 32) + (y // 32)) % 2 == 0 else 60
        b = 60  if ((x // 32) + (y // 32)) % 2 == 0 else 200
        row += bytes([r, g, b])
    rows.append(row)
idat = chunk(b"IDAT", zlib.compress(b"".join(rows), 9))
iend = chunk(b"IEND", b"")
out = sig + ihdr + idat + iend
path = os.environ.get("OUT", "tests/fixtures/test_image.png")
os.makedirs(os.path.dirname(path), exist_ok=True)
with open(path, "wb") as f:
    f.write(out)
print("wrote", path)
PY
}

# ---- models ---------------------------------------------------------------

# Smol text-only model used by examples/quickstart and examples/chat.
# Qwen2.5 0.5B Instruct, Q4_K_M (~400 MB). Tiny enough to download in <1 min.
smol() {
  local repo="Qwen/Qwen2.5-0.5B-Instruct-GGUF"
  download_hf "$repo" "qwen2.5-0.5b-instruct-q4_k_m.gguf" \
    "$MODELS_DIR/qwen2.5-0.5b-instruct-q4_k_m.gguf"
  test_image
}

# LM Studio's Q4_K_M repack of Gemma 4 E4B Instruct — text + vision.
# ~4 GB text model + ~1.4 GB mmproj projector.
gemma4() {
  local repo="lmstudio-community/gemma-4-E4B-it-GGUF"
  download_hf "$repo" "gemma-4-E4B-it-Q4_K_M.gguf" \
    "$MODELS_DIR/gemma-4-E4B-it-Q4_K_M.gguf"
  download_hf "$repo" "mmproj-gemma-4-E4B-it-BF16.gguf" \
    "$MODELS_DIR/mmproj-gemma-4-E4B-it-BF16.gguf"
  test_image
}

# Unsloth's Q4_K_M repack of Liquid AI's LFM2.5-VL 1.6B — text + vision.
# ~1 GB text model + ~340 MB mmproj projector.
lfm_vl() {
  local repo="unsloth/LFM2.5-VL-1.6B-GGUF"
  download_hf "$repo" "LFM2.5-VL-1.6B-Q4_K_M.gguf" \
    "$MODELS_DIR/LFM2.5-VL-1.6B-Q4_K_M.gguf"
  download_hf "$repo" "mmproj-BF16.gguf" \
    "$MODELS_DIR/LFM2.5-VL-1.6B-mmproj-BF16.gguf"
  test_image
}

# LFM2.5-VL text side only — used by examples that don't need vision
# (e.g. the text-only `llama-crab-server` example). The mmproj from
# `lfm-vl` is skipped to keep the download small.
lfm_text() {
  local repo="unsloth/LFM2.5-VL-1.6B-GGUF"
  download_hf "$repo" "LFM2.5-VL-1.6B-Q4_K_M.gguf" \
    "$MODELS_DIR/LFM2.5-VL-1.6B-Q4_K_M.gguf"
}

# Embedding model used by examples/embeddings (small BGE).
bge_small() {
  local repo="CompendiumLabs/bge-small-en-v1.5-gguf"
  download_hf "$repo" "bge-small-en-v1.5-q4_k_m.gguf" \
    "$MODELS_DIR/bge-small-en-v1.5-q4_k_m.gguf"
}

# Cross-encoder rerank model used by /v1/rerank (BGE-reranker-base Q4_K_M).
# BERT encoder-only architecture; requires pooling_type = Rank.
bge_reranker() {
  local repo="turingevo/bge-reranker-base-Q4_K_M-GGUF"
  download_hf "$repo" "bge-reranker-base-q4_k_m.gguf" \
    "$MODELS_DIR/bge-reranker-base-q4_k_m.gguf"
}

# ---- dispatch -------------------------------------------------------------

target="${1:-all}"
case "$target" in
  smol)         smol ;;
  gemma4)       gemma4 ;;
  lfm-vl|lfmvl) lfm_vl ;;
  lfm-text)     lfm_text ;;
  bge)          bge_small ;;
  bge-reranker) bge_reranker ;;
  test-image)   test_image ;;
  all)
    smol
    gemma4
    lfm_vl
    lfm_text
    bge_small
    bge_reranker
    test_image
    ;;
  *)
    echo "unknown target: $target" >&2
    echo "valid targets: smol, gemma4, lfm-vl, lfm-text, bge, bge-reranker, test-image, all" >&2
    exit 2
    ;;
esac

echo
echo "✓ All requested files in place."
echo
du -h "$MODELS_DIR" 2>/dev/null | sort -k2 || ls -lh "$MODELS_DIR"
