#!/usr/bin/env bash
# Wipe every build artifact, downloaded model, JS dependency, generated
# documentation, IDE folder and Cargo cache under the repository.
#
# Usage:
#   ./scripts/clean.sh                 # full repo clean (asks first)
#   ./scripts/clean.sh --dry-run       # show what would be removed
#   ./scripts/clean.sh --help          # this message
#
# Designed to back `make clean` — there are no other flags.  Anything
# `cargo build`, `pnpm install` or `docusaurus build` can recreate is
# considered disposable.  The bundled llama.cpp submodule is never
# touched, and `.git/` is never inspected.  The script always asks
# before deleting anything (unless --dry-run is passed).
#
# Written for bash 3.2, the version Apple ships in /bin/bash.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# --- parse args ---------------------------------------------------------

DRY_RUN=0

for arg in "$@"; do
  case "$arg" in
    --dry-run)     DRY_RUN=1 ;;
    -h|--help)
      sed -n '2,11p' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

# --- helpers ------------------------------------------------------------

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

# find wrapper that prunes .git directories and the bundled llama.cpp
# submodule so we never inspect (or delete) versioned code.  The
# *contents* of `node_modules` and `target` directories are also pruned
# (the parents themselves are still enumerated and later removed) so
# the secondary passes don't have to walk thousands of nested
# node_modules created by pnpm's symlink layout.
find_clean() {
  find "$ROOT" \
    \( \
      -path "$ROOT/.git" -o \
      -path "*/.git" -o \
      -path "$ROOT/crates/llama-crab-sys/llama.cpp" -o \
      -path "*/llama-crab-sys/llama.cpp" -o \
      -path "*/llama-crab-sys/llama.cpp/*" -o \
      -path "*/node_modules/*" -o \
      -path "*/target/*" \
    \) -prune -o \
    "$@" -print 2>/dev/null
}

# Like find_clean but additionally prunes the parent node_modules /
# target directories themselves (their contents are already gone) so
# the secondary passes don't print them as `dist`/`gen` matches.
find_shallow() {
  find "$ROOT" \
    \( \
      -path "$ROOT/.git" -o \
      -path "*/.git" -o \
      -path "$ROOT/crates/llama-crab-sys/llama.cpp" -o \
      -path "*/llama-crab-sys/llama.cpp" -o \
      -path "*/llama-crab-sys/llama.cpp/*" -o \
      -path "*/node_modules" -o \
      -path "*/node_modules/*" -o \
      -path "*/target" -o \
      -path "*/target/*" \
    \) -prune -o \
    "$@" -print 2>/dev/null
}

# --- inventory -----------------------------------------------------------

build_paths=()
node_paths=()
docs_paths=()
temp_paths=()
ide_paths=()
model_paths=()
cargo_paths=()

# Cargo target/ directories anywhere in the tree (workspace root,
# src-tauri/ inside the Tauri example, etc.).
while IFS= read -r d; do [[ -n "$d" ]] && build_paths+=("$d"); done \
  < <(find_clean -type d -name target)

# JS dependencies and build outputs.
while IFS= read -r d; do [[ -n "$d" ]] && node_paths+=("$d"); done \
  < <(find_clean -type d -name node_modules)
while IFS= read -r d; do [[ -n "$d" ]] && node_paths+=("$d"); done \
  < <(find_shallow -type d -name dist)
while IFS= read -r d; do [[ -n "$d" ]] && node_paths+=("$d"); done \
  < <(find_shallow -type d -name gen)

# Generated documentation trees (mdBook, Docusaurus, rustdoc mirror).
for d in \
  "$ROOT/docs/book" \
  "$ROOT/docs/.docusaurus" \
  "$ROOT/docs/build" \
  "$ROOT/docs/docs/api" \
  "$ROOT/docs/static/api"
do
  [[ -e "$d" ]] && docs_paths+=("$d")
done
while IFS= read -r d; do [[ -n "$d" ]] && docs_paths+=("$d"); done \
  < <(find_shallow -type d -name book)

# Scattered scratch files: macOS metadata, log output, rustfmt backups,
# MSVC debug symbols, cargo-mutants data.
while IFS= read -r f; do [[ -n "$f" ]] && temp_paths+=("$f"); done \
  < <(find_shallow -type f -name ".DS_Store")
while IFS= read -r f; do [[ -n "$f" ]] && temp_paths+=("$f"); done \
  < <(find_shallow -type f -name "*.log")
while IFS= read -r f; do [[ -n "$f" ]] && temp_paths+=("$f"); done \
  < <(find_shallow -type f -name "*.rs.bk")
while IFS= read -r f; do [[ -n "$f" ]] && temp_paths+=("$f"); done \
  < <(find_shallow -type f -name "*.pdb")
while IFS= read -r d; do [[ -n "$d" ]] && temp_paths+=("$d"); done \
  < <(find_shallow -type d -name "mutants.out*")

# IDE folders.
while IFS= read -r d; do [[ -n "$d" ]] && ide_paths+=("$d"); done \
  < <(find_shallow -type d -name ".idea")
while IFS= read -r d; do [[ -n "$d" ]] && ide_paths+=("$d"); done \
  < <(find_shallow -type d -name ".vscode")

# Downloaded GGUF models and test fixtures.
[[ -d "$ROOT/models" ]] && model_paths+=("$ROOT/models")
for f in "$ROOT"/tests/fixtures/*.gguf; do
  [[ -f "$f" ]] && model_paths+=("$f")
done

# The cargo registry cache, shared across all Rust projects on the
# machine.  Wiping it forces cargo to re-download every crate on the
# next build.
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
[[ -d "$cargo_home/registry" ]] && cargo_paths+=("$cargo_home/registry")
[[ -d "$cargo_home/git" ]]      && cargo_paths+=("$cargo_home/git")
for d in "$ROOT/.cargo" "$ROOT/.sccache"; do
  [[ -d "$d" ]] && cargo_paths+=("$d")
done

# --- report -------------------------------------------------------------

build_kb=$(sum_paths "${build_paths[@]+"${build_paths[@]}"}")
node_kb=$(sum_paths  "${node_paths[@]+"${node_paths[@]}"}")
docs_kb=$(sum_paths  "${docs_paths[@]+"${docs_paths[@]}"}")
temp_kb=$(sum_paths  "${temp_paths[@]+"${temp_paths[@]}"}")
ide_kb=$(sum_paths   "${ide_paths[@]+"${ide_paths[@]}"}")
model_kb=$(sum_paths "${model_paths[@]+"${model_paths[@]}"}")
cargo_kb=$(sum_paths "${cargo_paths[@]+"${cargo_paths[@]}"}")
total_kb=$((build_kb + node_kb + docs_kb + temp_kb + ide_kb + model_kb + cargo_kb))

echo "🦀 llama-crab cleanup"
echo
echo "  Build artifacts (target/):  $(human $build_kb) across ${#build_paths[@]} path(s)"
echo "  JS deps/builds:             $(human $node_kb) across ${#node_paths[@]} path(s)"
echo "  Generated docs:             $(human $docs_kb) across ${#docs_paths[@]} path(s)"
echo "  Temp / scratch files:       $(human $temp_kb) across ${#temp_paths[@]} path(s)"
echo "  IDE folders:                $(human $ide_kb) across ${#ide_paths[@]} path(s)"
echo "  Downloaded models:          $(human $model_kb) across ${#model_paths[@]} path(s)"
echo "  Cargo registry + cache:     $(human $cargo_kb) across ${#cargo_paths[@]} path(s)"
echo "  ----------------------------------------"
echo "  Total to remove:            $(human $total_kb)"
echo

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
for p in "${node_paths[@]+"${node_paths[@]}"}";  do remove "$p"; done
for p in "${docs_paths[@]+"${docs_paths[@]}"}";  do remove "$p"; done
for p in "${temp_paths[@]+"${temp_paths[@]}"}";  do remove "$p"; done
for p in "${ide_paths[@]+"${ide_paths[@]}"}";    do remove "$p"; done
for p in "${model_paths[@]+"${model_paths[@]}"}"; do remove "$p"; done
for p in "${cargo_paths[@]+"${cargo_paths[@]}"}"; do remove "$p"; done

# Recreate the model directory so the download script can write into it
# without a re-creation step.
mkdir -p "$ROOT/models"

echo
echo "✓ Done. Disk freed: $(human $total_kb)"
df -h / | tail -1
