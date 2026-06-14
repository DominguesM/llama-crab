#!/usr/bin/env bash
# Clean up build artifacts, downloaded GGUF models and Cargo cache.
#
# Usage:
#   ./scripts/clean.sh                # build artifacts + models
#   ./scripts/clean.sh --keep-models  # just build artifacts
#   ./scripts/clean.sh --only-models  # just downloaded GGUF files
#   ./scripts/clean.sh --all          # everything, including cargo registry
#   ./scripts/clean.sh --dry-run      # show what would be removed
#
# After the first `cargo build` the `target/` directory typically holds
# 5-15 GB; downloaded GGUF models another 5-8 GB.  `clean.sh --all` is
# the nuclear option and frees the most space (it also wipes the cargo
# registry cache, which cargo will re-download next build).
#
# The script always asks for confirmation before deleting anything.
# Written for bash 3.2, the version Apple ships in /bin/bash.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# --- parse args ---------------------------------------------------------

KEEP_MODELS=0
ONLY_MODELS=0
ALL=0
DRY_RUN=0

for arg in "$@"; do
  case "$arg" in
    --keep-models) KEEP_MODELS=1 ;;
    --only-models) ONLY_MODELS=1 ;;
    --all)         ALL=1 ;;
    --dry-run)     DRY_RUN=1 ;;
    -h|--help)
      sed -n '2,12p' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

if [[ $KEEP_MODELS -eq 1 && $ONLY_MODELS -eq 1 ]]; then
  echo "--keep-models and --only-models are mutually exclusive" >&2
  exit 2
fi

# --- inventory -----------------------------------------------------------

# Sum the on-disk size of every path that would be removed.  Uses
# du -sk because both BSD and GNU du accept it.  Paths are passed as
# positional args so the function works on bash 3.2.
sum_paths() {
  local total=0
  local p
  for p in "$@"; do
    if [[ -e "$p" ]]; then
      local kb
      kb=$(du -sk "$p" 2>/dev/null | awk '{print $1}')
      total=$((total + kb))
    fi
  done
  echo $total
}

human() {
  awk -v kb="$1" 'BEGIN {
    n = kb * 1024
    split("B KB MB GB TB", u, " ")
    i = 1
    while (n >= 1024 && i < 5) { n /= 1024; i++ }
    printf "%.1f %s", n, u[i]
  }'
}

build_paths=()
if [[ $ONLY_MODELS -eq 0 ]]; then
  build_paths+=("$ROOT/target")
fi

model_paths=()
if [[ $KEEP_MODELS -eq 0 ]]; then
  model_paths+=("$ROOT/models")
  # Test fixture GGUF (if any)
  for f in "$ROOT"/tests/fixtures/*.gguf; do
    [[ -f "$f" ]] && model_paths+=("$f")
  done
fi

cargo_paths=()
if [[ $ALL -eq 1 && $ONLY_MODELS -eq 0 ]]; then
  # The cargo registry cache, shared across all Rust projects on the
  # machine.  Wiping it will force cargo to re-download every crate on
  # the next build.
  cargo_home="${CARGO_HOME:-$HOME/.cargo}"
  cargo_paths+=("$cargo_home/registry")
  cargo_paths+=("$cargo_home/git")
  # Sccache / build cache if present
  for d in "$ROOT/.cargo" "$ROOT/.sccache"; do
    [[ -d "$d" ]] && cargo_paths+=("$d")
  done
fi

# --- report -------------------------------------------------------------

build_kb=$(sum_paths "${build_paths[@]+"${build_paths[@]}"}")
model_kb=$(sum_paths "${model_paths[@]+"${model_paths[@]}"}")
cargo_kb=$(sum_paths "${cargo_paths[@]+"${cargo_paths[@]}"}")
total_kb=$((build_kb + model_kb + cargo_kb))

echo "🦀 llama-crab cleanup"
echo
echo "  Build artifacts (target/):  $(human $build_kb) across ${#build_paths[@]} path(s)"
echo "  Downloaded models:          $(human $model_kb) across ${#model_paths[@]} path(s)"
echo "  Cargo registry + cache:     $(human $cargo_kb) across ${#cargo_paths[@]} path(s)"
echo "  ----------------------------------------"
echo "  Total to remove:            $(human $total_kb)"
echo

# Confirm before deleting.
if [[ $DRY_RUN -eq 1 ]]; then
  echo "(dry-run — no changes made)"
  exit 0
fi

if [[ $total_kb -eq 0 ]]; then
  echo "✓ Nothing to clean."
  exit 0
fi

read -r -p "Proceed? [y/N] " ans
if [[ ! "$ans" =~ ^[Yy]$ ]]; then
  echo "aborted."
  exit 0
fi

# --- remove -------------------------------------------------------------

remove() {
  local p="$1"
  if [[ -e "$p" ]]; then
    echo "  rm -rf $p"
    rm -rf "$p"
  fi
}

for p in "${build_paths[@]+"${build_paths[@]}"}"; do remove "$p"; done
for p in "${model_paths[@]+"${model_paths[@]}"}"; do remove "$p"; done
for p in "${cargo_paths[@]+"${cargo_paths[@]}"}"; do remove "$p"; done

# Recreate the model directory so the download script can write into it
# without a re-creation step.
[[ $KEEP_MODELS -eq 0 && $ONLY_MODELS -eq 0 ]] && mkdir -p "$ROOT/models"

echo
echo "✓ Done. Disk freed: $(human $total_kb)"
df -h / | tail -1
