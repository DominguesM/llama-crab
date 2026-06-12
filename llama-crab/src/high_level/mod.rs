//! High-level orchestrator: load model, create context, generate tokens.
//!
//! This module mirrors the public surface of `llama-cpp-python`'s `Llama`
//! class, but stays 100% safe Rust and uses idiomatic builders instead of
//! `__init__` parameters.

pub mod chat_completion;
pub mod completion;
pub mod embedding;
pub mod hf_tokenizer;
pub mod infill;
pub mod rerank;
pub mod tokenizer;

#[cfg(feature = "hf-tokenizer")]
#[cfg_attr(docsrs, doc(cfg(feature = "hf-tokenizer")))]
pub use self::hf_tokenizer::HfTokenizer;

pub use self::tokenizer::{FimTokens, LlamaTokenizer, Tokenizer};

use std::path::{Path, PathBuf};

use crate::backend::LlamaBackend;
use crate::context::{LlamaContext, LlamaContextParams};
use crate::error::Result;
use crate::model::LlamaModel;
use crate::model::params::LlamaModelParams;

pub use self::chat_completion::{create_chat_completion, ChatMessage};
pub use self::completion::{create_completion, Completion, StopReason};

/// Top-level orchestrator. Owns the backend, the model and the context.
#[derive(Debug)]
pub struct Llama {
    _backend: LlamaBackend,
    model: LlamaModel,
    context: LlamaContext<'static>,
    _not_send_sync: std::marker::PhantomData<*mut ()>,
}

impl Llama {
    /// Load a GGUF model with the given parameters.
    ///
    /// # Errors
    /// Returns an error if the file cannot be loaded, the model is
    /// incompatible, or context creation fails.
    pub fn load(params: LlamaParams) -> Result<Self> {
        let backend = LlamaBackend::init()?;
        let model = LlamaModel::load_from_file(&backend, &params.model_path, &params.model)?;
        // We transmute the lifetime of the context to `'static` because
        // `Llama` owns the model and outlives the context. The PhantomData
        // marker keeps `Llama` !Send/!Sync to mirror llama.cpp's thread model.
        let ctx = model.new_context(&backend, params.context.clone())?;
        let ctx: LlamaContext<'static> =
            unsafe { std::mem::transmute::<LlamaContext<'_>, LlamaContext<'static>>(ctx) };
        Ok(Self {
            _backend: backend,
            model,
            context: ctx,
            _not_send_sync: std::marker::PhantomData,
        })
    }

    /// Borrow the inner model.
    #[must_use]
    pub const fn model(&self) -> &LlamaModel {
        &self.model
    }

    /// Borrow the inner context.
    #[must_use]
    pub const fn context(&mut self) -> &mut LlamaContext<'static> {
        &mut self.context
    }

    /// Synchronous text completion. Generates up to `max_tokens` tokens.
    pub fn create_completion(&mut self, prompt: &str, max_tokens: usize) -> Result<Completion> {
        create_completion(self, prompt, max_tokens)
    }

    /// Synchronous chat completion. The messages are rendered through a
    /// minimal `role: content\n` template (real chat-template rendering lands
    /// in v0.2) and the response is decoded token-by-token.
    pub fn create_chat_completion(
        &mut self,
        messages: &[ChatMessage],
        max_tokens: usize,
    ) -> Result<ChatMessage> {
        create_chat_completion(self, messages, max_tokens)
    }
}

/// All parameters accepted by [`Llama::load`].
#[derive(Debug, Clone)]
pub struct LlamaParams {
    /// Path to the GGUF file.
    pub model_path: PathBuf,
    /// Model-side params (GPU offload, mmap, etc.).
    pub model: LlamaModelParams,
    /// Context-side params (n_ctx, embeddings, etc.).
    pub context: LlamaContextParams,
}

impl LlamaParams {
    /// Construct parameters targeting the given model.
    #[must_use]
    pub fn new(model_path: impl AsRef<Path>) -> Self {
        Self {
            model_path: model_path.as_ref().to_path_buf(),
            model: LlamaModelParams::default(),
            context: LlamaContextParams::default(),
        }
    }

    /// Set the path to the GGUF file.
    #[must_use]
    pub fn with_model_path(mut self, p: impl AsRef<Path>) -> Self {
        self.model_path = p.as_ref().to_path_buf();
        self
    }

    /// Number of layers to offload to the GPU.
    #[must_use]
    pub fn with_n_gpu_layers(mut self, n: i32) -> Self {
        self.model = self.model.with_n_gpu_layers(n);
        self
    }

    /// Memory-map the model file.
    #[must_use]
    pub fn with_use_mmap(mut self, yes: bool) -> Self {
        self.model = self.model.with_use_mmap(yes);
        self
    }

    /// Set the context size.
    #[must_use]
    pub fn with_n_ctx(mut self, n: u32) -> Self {
        self.context = self.context.with_n_ctx(n);
        self
    }

    /// Enable embeddings collection.
    #[must_use]
    pub fn with_embeddings(mut self, yes: bool) -> Self {
        self.context = self.context.with_embeddings(yes);
        self
    }

    /// Configure the number of CPU threads.
    #[must_use]
    pub fn with_n_threads(mut self, n: i32) -> Self {
        self.context = self.context.with_n_threads(n);
        self
    }

    /// Configure the number of batch CPU threads.
    #[must_use]
    pub fn with_n_threads_batch(mut self, n: i32) -> Self {
        self.context = self.context.with_n_threads_batch(n);
        self
    }
}

impl Default for LlamaParams {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            model: LlamaModelParams::default(),
            context: LlamaContextParams::default(),
        }
    }
}

// StopReason is re-exported above for downstream users.
#[doc(inline)]
pub use StopReason as _StopReasonShim;
