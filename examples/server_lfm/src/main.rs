//! `server_lfm` — launches the `llama-crab-server` HTTP binary pre-configured
//! for the Liquid AI LFM2.5-VL 1.6B model.
//!
//! This is a thin wrapper around `cargo run -p llama-crab-server` so the
//! model path is filled in from the `lfm-vl` download target and any
//! extra arguments (host, port, context size, embeddings flag, …) are
//! forwarded unchanged.
//!
//! Usage:
//!
//! ```bash
//! ./examples/run.sh server_lfm
//! ```
//!
//! or, after `./scripts/download_models.sh lfm-vl`:
//!
//! ```bash
//! cargo run --release --bin run_server_lfm -- \
//!   models/LFM2.5-VL-1.6B-Q4_K_M.gguf \
//!   --host 127.0.0.1 \
//!   --port 8080 \
//!   --n-ctx 2048
//! ```
//!
//! While the server is running, see the README of this example for ready
//! to copy `curl` invocations against `/v1/chat/completions` and the
//! other routes.

use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let model = match args.next() {
        Some(m) => m,
        None => {
            eprintln!("usage: run_server_lfm <model.gguf> [-- extra llama-crab-server args...]");
            return ExitCode::from(2);
        }
    };

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--release", "-p", "llama-crab-server", "--"])
        .arg("--model")
        .arg(&model);
    for arg in args {
        cmd.arg(arg);
    }

    let status = match cmd.status() {
        Ok(s) => s,
        Err(err) => {
            eprintln!("failed to spawn cargo: {err}");
            return ExitCode::from(1);
        }
    };

    ExitCode::from(status.code().unwrap_or(1) as u8)
}
