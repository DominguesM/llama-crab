#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

APP="examples/tauri-chat-lfm"

[[ -f "$APP/package.json" ]] || fail "missing Tauri chat example package.json"
[[ -f "$APP/src/main.ts" ]] || fail "missing Tauri chat example frontend"
[[ -f "$APP/src-tauri/Cargo.toml" ]] || fail "missing Tauri chat example Cargo.toml"
[[ -f "$APP/src-tauri/src/lib.rs" ]] || fail "missing Tauri chat example Rust entrypoint"
[[ -f "$APP/README.md" ]] || fail "missing Tauri chat example README"

grep -Fq '"@llama-crab/tauri": "workspace:*"' "$APP/package.json" \
  || fail "example package.json must use the local @llama-crab/tauri workspace package"
grep -Fq 'LiquidAI/LFM2.5-350M-GGUF' "$APP/src/main.ts" \
  || fail "frontend must document the LFM2.5-350M-GGUF model target"
grep -Fq 'LFM2.5-350M-Q4_K_M.gguf' "$APP/src/main.ts" \
  || fail "frontend must use one fixed Q4_K_M model file"
grep -Fq 'new LlamaCrabTauri()' "$APP/src/main.ts" \
  || fail "frontend must use the @llama-crab/tauri facade"
grep -Fq 'chat.completions.create' "$APP/src/main.ts" \
  || fail "frontend must call chat.completions.create"
grep -Fq 'invoke<string>("ensure_lfm_model", { onProgress })' "$APP/src/main.ts" \
  || fail "frontend must call the automatic model download command"
grep -Fq 'Channel<DownloadProgress>' "$APP/src/main.ts" \
  || fail "frontend must receive download progress over a Tauri channel"
grep -Fq 'downloaded_bytes' "$APP/src/main.ts" \
  || fail "frontend must display downloaded byte progress"
grep -Fq 'clear-button' "$APP/src/main.ts" \
  || fail "frontend must expose a clear button"
grep -Fq 'tauri-plugin-llama-crab = { path = "../../../crates/tauri-plugin-llama-crab" }' "$APP/src-tauri/Cargo.toml" \
  || fail "Rust app must depend on the local tauri-plugin-llama-crab crate"
grep -Fq '.plugin(tauri_plugin_llama_crab::init())' "$APP/src-tauri/src/lib.rs" \
  || fail "Rust app must initialize tauri-plugin-llama-crab"
grep -Fq 'ensure_lfm_model' "$APP/src-tauri/src/lib.rs" \
  || fail "Rust app must implement automatic model download"
grep -Fq 'Channel<DownloadProgress>' "$APP/src-tauri/src/lib.rs" \
  || fail "Rust app must publish download progress over a Tauri channel"
grep -Fq 'content_length()' "$APP/src-tauri/src/lib.rs" \
  || fail "Rust app must report total download size when available"
grep -Fq '"llama-crab:default"' "$APP/src-tauri/capabilities/default.json" \
  || fail "Tauri capability must include llama-crab default permissions"
grep -Fq 'download automatico' "$APP/README.md" \
  || fail "README must describe automatic model download"
grep -Fq 'progresso do download' "$APP/README.md" \
  || fail "README must describe download progress"
grep -Fq 'https://huggingface.co/LiquidAI/LFM2.5-350M-GGUF/resolve/main/LFM2.5-350M-Q4_K_M.gguf' "$APP/src-tauri/src/lib.rs" \
  || fail "Rust app must download the fixed Hugging Face model file"

grep -Fq 'tauri_chat_lfm)' examples/run.sh \
  || fail "examples/run.sh must expose tauri_chat_lfm"
grep -Fq 'pnpm --filter tauri-chat-lfm tauri dev' examples/run.sh \
  || fail "examples/run.sh must run the Tauri chat example"
grep -Fq './examples/run.sh tauri_chat_lfm' examples/README.md \
  || fail "examples README must document the Tauri chat example"

grep -Fq '"create_chat_completion"' crates/tauri-plugin-llama-crab/build.rs \
  || fail "plugin permission generation must include create_chat_completion"
grep -Fq '"allow-create-chat-completion"' crates/tauri-plugin-llama-crab/permissions/default.toml \
  || fail "plugin default permission must include create_chat_completion"

echo "tauri chat LFM smoke tests passed"
