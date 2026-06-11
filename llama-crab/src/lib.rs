//! `llama-crab` — safe, ergonomic Rust bindings to `llama.cpp`.
//!
//! ## Quickstart
//!
//! ```no_run
//! use llama_crab::{Llama, LlamaParams};
//!
//! let mut llama = Llama::load(LlamaParams::new("model.gguf").with_n_ctx(2048))?;
//! let resp = llama.create_completion("Hello, world!", 64)?;
//! println!("{}", resp.text);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![doc(html_logo_url = "https://raw.githubusercontent.com/DominguesM/llama-crab/main/docs/src/assets/logo.png")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::needless_doctest_main)]

pub mod backend;
pub mod batch;
pub mod cache;
pub mod chat;
pub mod context;
pub mod error;
pub mod high_level;
pub mod log;
pub mod model;
pub mod multimodal;
pub mod sampling;
pub mod speculative;
pub mod token;
pub mod token_data;
pub mod util;

#[cfg(feature = "hf-tokenizer")]
#[cfg_attr(docsrs, doc(cfg(feature = "hf-tokenizer")))]
pub mod hf_tokenizer;

pub use crate::backend::{LlamaBackend, NumaStrategy};
pub use crate::batch::{BatchAddError, LlamaBatch};
pub use crate::chat::Role;
pub use crate::context::{LlamaContext, LlamaContextParams};
pub use crate::error::{LlamaError, Result};
pub use crate::high_level::chat_completion::ChatMessage;
pub use crate::high_level::completion::{Completion, StopReason};
pub use crate::high_level::{Llama, LlamaParams};
pub use crate::log::{LogOptions, send_logs_to_tracing};
pub use crate::model::{params::LlamaModelParams, LlamaModel};
pub use crate::sampling::{LlamaSampler, SamplerChain};
pub use crate::token::LlamaToken;
pub use crate::token_data::{LlamaTokenData, LlamaTokenDataArray};
