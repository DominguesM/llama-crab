#!/usr/bin/env bash
# Download the GGUF models used by `llama-crab`'s integration tests.
#
# Usage:
#   ./scripts/download_models.sh                 # downloads both
#   ./scripts/download_models.sh gemma4         # just Gemma 4
#   ./scripts/download_models.sh lfm-vl         # just LFM2.5-VL
#   ./scripts/download_models.sh test-image     # just the test PNG
#
# Models are placed in `./models/` (the conventional path the tests look
# at). The script is idempotent: files that already exist are skipped.
#
# Requirements: `huggingface-cli` (from `pip install huggingface_hub`) or
# `curl`. Set `HF_TOKEN` if you want to use a private/gated model.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODELS_DIR="$ROOT/models"
FIXTURE_DIR="$ROOT/tests/fixtures"
mkdir -p "$MODELS_DIR" "$FIXTURE_DIR"

# ---- helpers --------------------------------------------------------------

have() { command -v "$1" >/dev/null 2>&1; }

download() {
  local url="$1" dest="$2"
  if [[ -f "$dest" ]] && [[ "$(stat -f%z "$dest" 2>/dev/null || stat -c%s "$dest")" -gt 1000000 ]]; then
    echo "✓ already have $(basename "$dest")"
    return 0
  fi
  echo "↓ downloading $(basename "$dest")"
  if have hf; then
    hf download "$url" --local-dir "$(dirname "$dest")" --token "${HF_TOKEN:-}"
  elif have curl; then
    curl -fL --retry 3 -o "$dest.part" "$url"
    mv "$dest.part" "$dest"
  else
    echo "ERROR: need either 'hf' (huggingface-cli) or 'curl' on PATH" >&2
    return 1
  fi
}

# ---- test image (256×256 PNG) -------------------------------------------

test_image() {
  if [[ -f "$FIXTURE_DIR/test_image.png" ]]; then
    echo "✓ test image present"
    return
  fi
  python3 - <<'PY' || true
import struct, zlib, os
# Write a 256×256 RGB PNG with a 4×4 checker pattern (red/blue).
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

gemma4() {
  # LM Studio community repack of Gemma 4 E4B Instruct — chat-tuned,
  # multimodal. We grab the Q4_K_M quant (~4 GB) and the matching mmproj.
  local repo="lmstudio-community/gemma-4-E4B-it-GGUF"
  download \
    "https://huggingface.co/${repo}/resolve/main/gemma-4-E4B-it-Q4_K_M.gguf" \
    "$MODELS_DIR/gemma-4-E4B-it-Q4_K_M.gguf"
  download \
    "https://huggingface.co/${repo}/resolve/main/gemma-4-E4B-it-mmproj.gguf" \
    "$MODELS_DIR/gemma-4-E4B-it-mmproj.gguf"
}

lfm_vl() {
  # Unsloth's Q4_K_M repack of Liquid AI's LFM2.5-VL 1.6B.
  local repo="unsloth/LFM2.5-VL-1.6B-GGUF"
  download \
    "https://huggingface.co/${repo}/resolve/main/LFM2.5-VL-1.6B-Q4_K_M.gguf" \
    "$MODELS_DIR/LFM2.5-VL-1.6B-Q4_K_M.gguf"
  download \
    "https://huggingface.co/${repo}/resolve/main/LFM2.5-VL-1.6B-mmproj.gguf" \
    "$MODELS_DIR/LFM2.5-VL-1.6B-mmproj.gguf"
}

# ---- dispatch -------------------------------------------------------------

target="${1:-all}"
case "$target" in
  gemma4)       gemma4 ;;
  lfm-vl|lfmvl) lfm_vl ;;
  test-image)   test_image ;;
  all)
    gemma4
    lfm_vl
    test_image
    ;;
  *)
    echo "unknown target: $target" >&2
    echo "valid targets: gemma4, lfm-vl, test-image, all" >&2
    exit 2
    ;;
esac

echo
echo "All requested files in place."
echo "Run the tests with:"
echo "  cargo test --workspace --features mtmd -- --nocapture"
