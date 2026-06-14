#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

[[ ! -e examples ]] || fail "examples/ must live in the llama-crab-examples repository"

if rg -n 'examples/run\.sh|examples/README\.md|examples/tauri-chat-lfm|examples/[A-Za-z0-9_-]+/Cargo\.toml|"\s*examples/' \
  Cargo.toml pnpm-workspace.yaml package.json .github tests README.md docs \
  -g '!tests/examples_migration_smoke.sh' -g '!target/**' -g '!node_modules/**' \
  -g '!docs/build/**' -g '!docs/.docusaurus/**' -g '!docs/static/api/**' \
  -g '!docs/docs/api/**' >/tmp/llama-crab-examples-migration-rg.out 2>/dev/null; then
  cat /tmp/llama-crab-examples-migration-rg.out >&2
  fail "old in-repository examples references remain"
fi

grep -Fq 'llama-crab-examples' README.md \
  || fail "README must point users to the external examples repository"

echo "llama-crab examples migration smoke tests passed"
