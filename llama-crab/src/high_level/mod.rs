//! High-level orchestrator: load model, create context, generate tokens.
//!
//! The API keeps model loading, context ownership and common generation flows
//! behind one safe Rust type.

pub mod chat_completion;
pub mod completion;
pub mod embedding;
pub mod hf_tokenizer;
pub mod infill;
pub mod openai_compat;
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
use crate::model::params::LlamaModelParams;
use crate::model::LlamaModel;

pub use self::chat_completion::{
    create_chat_completion, create_chat_completion_stream, create_chat_completion_stream_with,
    ChatMessage,
};
pub use self::completion::{
    create_completion, create_completion_stream, create_completion_stream_with_sampler,
    create_completion_with_options, create_completion_with_sampler, Completion, CompletionChunk,
    CompletionLogprobs, CompletionOptions, SamplingOptions, StopReason, StreamControl,
    TokenLogprob,
};

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
    /// rejected by llama.cpp, or context creation fails.
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

    /// Synchronous text completion with high-level options.
    pub fn create_completion_with_options(
        &mut self,
        prompt: &str,
        options: CompletionOptions,
    ) -> Result<Completion> {
        create_completion_with_options(self, prompt, options)
    }

    /// Synchronous text completion using a caller-provided sampler.
    pub fn create_completion_with_sampler(
        &mut self,
        prompt: &str,
        options: CompletionOptions,
        sampler: &mut crate::sampling::LlamaSampler,
    ) -> Result<Completion> {
        create_completion_with_sampler(self, prompt, options, sampler)
    }

    /// Synchronous streaming text completion. The callback is invoked as text
    /// becomes available and can return [`StreamControl::Stop`] to end
    /// generation.
    pub fn create_completion_stream<F>(
        &mut self,
        prompt: &str,
        options: CompletionOptions,
        on_chunk: F,
    ) -> Result<Completion>
    where
        F: FnMut(CompletionChunk) -> StreamControl,
    {
        create_completion_stream(self, prompt, options, on_chunk)
    }

    /// Synchronous streaming text completion using a caller-provided sampler.
    pub fn create_completion_stream_with_sampler<F>(
        &mut self,
        prompt: &str,
        options: CompletionOptions,
        sampler: &mut crate::sampling::LlamaSampler,
        on_chunk: F,
    ) -> Result<Completion>
    where
        F: FnMut(CompletionChunk) -> StreamControl,
    {
        create_completion_stream_with_sampler(self, prompt, options, sampler, on_chunk)
    }

    /// Synchronous chat completion. The messages are rendered with the Plain
    /// built-in template and the response is decoded token-by-token.
    pub fn create_chat_completion(
        &mut self,
        messages: &[ChatMessage],
        max_tokens: usize,
    ) -> Result<ChatMessage> {
        create_chat_completion(self, messages, max_tokens)
    }

    /// Synchronous streaming chat completion using the Plain template.
    pub fn create_chat_completion_stream<F>(
        &mut self,
        messages: &[ChatMessage],
        max_tokens: usize,
        on_chunk: F,
    ) -> Result<ChatMessage>
    where
        F: FnMut(CompletionChunk) -> StreamControl,
    {
        create_chat_completion_stream(self, messages, max_tokens, on_chunk)
    }

    /// Synchronous streaming chat completion with a chosen built-in template,
    /// optional tools, and completion options.
    pub fn create_chat_completion_stream_with<F>(
        &mut self,
        messages: &[ChatMessage],
        template: crate::chat::BuiltinTemplate,
        tools: &[crate::chat::ToolDefinition],
        options: CompletionOptions,
        on_chunk: F,
    ) -> Result<ChatMessage>
    where
        F: FnMut(CompletionChunk) -> StreamControl,
    {
        create_chat_completion_stream_with(self, messages, template, tools, options, on_chunk)
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

    /// Configure the pooling type (used by embedding models).
    #[must_use]
    pub fn with_pooling_type(mut self, p: crate::context::params::PoolingType) -> Self {
        self.context = self.context.with_pooling_type(p);
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
