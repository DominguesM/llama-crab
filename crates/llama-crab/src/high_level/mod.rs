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
use std::str::FromStr;

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
    /// Filename within a Hugging Face repo (set via `with_hf_filename`).
    hf_filename: Option<String>,
    /// Revision (branch, tag, or commit) of a Hugging Face repo.
    hf_revision: Option<String>,
    /// Hugging Face access token for gated/private repos.
    hf_token: Option<String>,
    /// Override for the Hugging Face cache directory.
    hf_cache_dir: Option<PathBuf>,
    /// Override for the Hugging Face endpoint (e.g. a mirror).
    hf_endpoint: Option<String>,
}

/// High-level mobile-oriented parameter presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MobilePreset {
    /// Small batches and CPU-only execution for memory-constrained devices.
    LowRam,
    /// Balanced defaults for interactive mobile chat.
    Balanced,
    /// Prefer GPU offload and larger batches for capable devices.
    GpuMax,
}

impl FromStr for MobilePreset {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "low-ram" | "low_ram" | "lowram" => Ok(Self::LowRam),
            "balanced" => Ok(Self::Balanced),
            "gpu-max" | "gpu_max" | "gpumax" => Ok(Self::GpuMax),
            other => Err(format!(
                "unknown mobile preset: {other} (expected low-ram, balanced, or gpu-max)"
            )),
        }
    }
}

impl LlamaParams {
    /// Construct parameters targeting the given model.
    #[must_use]
    pub fn new(model_path: impl AsRef<Path>) -> Self {
        Self {
            model_path: model_path.as_ref().to_path_buf(),
            model: LlamaModelParams::default(),
            context: LlamaContextParams::default(),
            hf_filename: None,
            hf_revision: None,
            hf_token: None,
            hf_cache_dir: None,
            hf_endpoint: None,
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

    /// Configure the logical maximum batch size.
    #[must_use]
    pub fn with_n_batch(mut self, n: u32) -> Self {
        self.context = self.context.with_n_batch(n);
        self
    }

    /// Configure the physical batch size used by a forward pass.
    #[must_use]
    pub fn with_n_ubatch(mut self, n: u32) -> Self {
        self.context = self.context.with_n_ubatch(n);
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

    /// Enable or disable KQV cache offload to the active GPU backend.
    #[must_use]
    pub fn with_offload_kqv(mut self, yes: bool) -> Self {
        self.context = self.context.with_offload_kqv(yes);
        self
    }

    /// Enable or disable flash attention.
    #[must_use]
    pub fn with_flash_attn(mut self, yes: bool) -> Self {
        self.context = self.context.with_flash_attn(yes);
        self
    }

    /// Configure the pooling type (used by embedding models).
    #[must_use]
    pub fn with_pooling_type(mut self, p: crate::context::params::PoolingType) -> Self {
        self.context = self.context.with_pooling_type(p);
        self
    }

    /// Set the filename within a Hugging Face repo. Required when the repo has multiple .gguf files.
    #[must_use]
    pub fn with_hf_filename(mut self, filename: impl Into<String>) -> Self {
        self.hf_filename = Some(filename.into());
        self
    }
    /// Set the revision (branch, tag, or commit) of the Hugging Face repo. Defaults to "main".
    #[must_use]
    pub fn with_hf_revision(mut self, revision: impl Into<String>) -> Self {
        self.hf_revision = Some(revision.into());
        self
    }
    /// Set the Hugging Face access token (for gated/private repos). Equivalent to setting HF_TOKEN.
    #[must_use]
    pub fn with_hf_token(mut self, token: impl Into<String>) -> Self {
        self.hf_token = Some(token.into());
        self
    }
    /// Override the cache directory. Defaults to `~/.cache/huggingface/hub` (or $HF_HOME/hub).
    #[must_use]
    pub fn with_hf_cache_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.hf_cache_dir = Some(dir.into());
        self
    }
    /// Set the Hugging Face endpoint (for mirrors like hf-mirror.com). Equivalent to setting HF_ENDPOINT.
    #[must_use]
    pub fn with_hf_endpoint(mut self, ep: impl Into<String>) -> Self {
        self.hf_endpoint = Some(ep.into());
        self
    }

    /// Apply a mobile-oriented preset. Call explicit setters after this method
    /// to override individual values.
    #[must_use]
    pub fn with_mobile_preset(self, preset: MobilePreset) -> Self {
        match preset {
            MobilePreset::LowRam => self
                .with_n_ctx(2048)
                .with_n_batch(128)
                .with_n_ubatch(128)
                .with_n_threads(4)
                .with_n_threads_batch(4)
                .with_n_gpu_layers(0)
                .with_flash_attn(false)
                .with_use_mmap(true),
            MobilePreset::Balanced => self
                .with_n_ctx(4096)
                .with_n_batch(512)
                .with_n_ubatch(256)
                .with_n_threads(4)
                .with_n_threads_batch(4)
                .with_n_gpu_layers(32)
                .with_flash_attn(true)
                .with_use_mmap(true),
            MobilePreset::GpuMax => self
                .with_n_ctx(4096)
                .with_n_batch(1024)
                .with_n_ubatch(512)
                .with_n_gpu_layers(99)
                .with_flash_attn(true)
                .with_offload_kqv(true)
                .with_use_mmap(true),
        }
    }
}

impl Default for LlamaParams {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            model: LlamaModelParams::default(),
            context: LlamaContextParams::default(),
            hf_filename: None,
            hf_revision: None,
            hf_token: None,
            hf_cache_dir: None,
            hf_endpoint: None,
        }
    }
}

// StopReason is re-exported above for downstream users.
#[doc(inline)]
pub use StopReason as _StopReasonShim;

#[cfg(test)]
mod tests {
    use super::{LlamaParams, MobilePreset};

    #[test]
    fn mobile_preset_can_be_overridden() {
        let params = LlamaParams::new("model.gguf")
            .with_mobile_preset(MobilePreset::Balanced)
            .with_n_ctx(1024)
            .with_n_gpu_layers(0);

        assert_eq!(params.context.build().n_ctx, 1024);
        assert_eq!(params.model.n_gpu_layers(), 0);
    }

    #[test]
    fn mobile_preset_parse_accepts_cli_names() {
        assert_eq!("low-ram".parse(), Ok(MobilePreset::LowRam));
        assert_eq!("balanced".parse(), Ok(MobilePreset::Balanced));
        assert_eq!("gpu-max".parse(), Ok(MobilePreset::GpuMax));
        assert!("fast".parse::<MobilePreset>().is_err());
    }

    #[test]
    fn with_hf_filename_sets_field() {
        let p = LlamaParams::new("foo.gguf").with_hf_filename("model.Q4_K_M.gguf");
        assert_eq!(p.hf_filename.as_deref(), Some("model.Q4_K_M.gguf"));
    }

    #[test]
    fn with_hf_revision_sets_field() {
        let p = LlamaParams::new("foo.gguf").with_hf_revision("refs/pr/42");
        assert_eq!(p.hf_revision.as_deref(), Some("refs/pr/42"));
    }

    #[test]
    fn with_hf_token_sets_field() {
        let p = LlamaParams::new("foo.gguf").with_hf_token("hf_secret");
        assert_eq!(p.hf_token.as_deref(), Some("hf_secret"));
    }

    #[test]
    fn with_hf_cache_dir_sets_field() {
        let p = LlamaParams::new("foo.gguf").with_hf_cache_dir(std::path::PathBuf::from("/tmp/cache"));
        assert_eq!(p.hf_cache_dir.as_deref(), Some(std::path::Path::new("/tmp/cache")));
    }

    #[test]
    fn with_hf_endpoint_sets_field() {
        let p = LlamaParams::new("foo.gguf").with_hf_endpoint("https://hf-mirror.com");
        assert_eq!(p.hf_endpoint.as_deref(), Some("https://hf-mirror.com"));
    }
}
