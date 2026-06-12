//! `LlamaContext` and its parameters.

pub mod embeddings;
pub mod kv_cache;
pub mod params;
pub mod sampling_state;
pub mod session;

use std::ptr::NonNull;

use llama_crab_sys as sys;

use crate::batch::LlamaBatch;
use crate::error::{LlamaError, Result};
use crate::model::LlamaModel;

/// A `llama_context` — the inferencing state for a [`LlamaModel`].
#[derive(Debug)]
pub struct LlamaContext<'a> {
    pub(crate) handle: NonNull<sys::llama_context>,
    pub(crate) model: &'a LlamaModel,
}

impl<'a> LlamaContext<'a> {
    /// Wrap a raw context pointer (internal — used by [`LlamaModel::new_context`]).
    pub(crate) fn from_raw(handle: NonNull<sys::llama_context>, model: &'a LlamaModel) -> Self {
        Self { handle, model }
    }

    /// Configured context size (`n_ctx`).
    #[must_use]
    pub fn n_ctx(&self) -> u32 {
        unsafe { sys::llama_n_ctx(self.handle.as_ptr()) as u32 }
    }

    /// Logical maximum batch size (`n_batch`).
    #[must_use]
    pub fn n_batch(&self) -> u32 {
        unsafe { sys::llama_n_batch(self.handle.as_ptr()) as u32 }
    }

    /// Physical batch size (`n_ubatch`).
    #[must_use]
    pub fn n_ubatch(&self) -> u32 {
        unsafe { sys::llama_n_ubatch(self.handle.as_ptr()) as u32 }
    }

    /// Maximum number of parallel sequences.
    #[must_use]
    pub fn n_seq_max(&self) -> u32 {
        unsafe { sys::llama_n_seq_max(self.handle.as_ptr()) as u32 }
    }

    /// Borrow the underlying raw context handle.
    ///
    /// Useful for FFI interop (e.g. feeding multimodal chunks). The pointer
    /// is valid for the lifetime of `self`.
    #[must_use]
    pub fn raw_handle(&self) -> *mut sys::llama_context {
        self.handle.as_ptr()
    }

    /// Decode a batch of tokens. `clear` must be called or the batch reset
    /// between decode and the next decode.
    ///
    /// # Errors
    /// Returns [`LlamaError::Decode`] if llama.cpp returns a negative code.
    pub fn decode(&mut self, batch: &LlamaBatch) -> Result<()> {
        let rc = unsafe { sys::llama_decode(self.handle.as_ptr(), *batch.raw()) };
        if rc != 0 {
            return Err(LlamaError::Decode(rc));
        }
        Ok(())
    }

    /// Encode a batch of tokens (embedding models).
    ///
    /// # Errors
    /// Returns [`LlamaError::Encode`] on failure.
    pub fn encode(&mut self, batch: &LlamaBatch) -> Result<()> {
        let rc = unsafe { sys::llama_encode(self.handle.as_ptr(), *batch.raw()) };
        if rc != 0 {
            return Err(LlamaError::Encode(rc));
        }
        Ok(())
    }

    /// Borrow the model this context was created from.
    #[must_use]
    pub const fn model(&self) -> &'a LlamaModel {
        self.model
    }

    /// Borrow the underlying C handle (read-only).
    pub(crate) fn raw(&self) -> *mut sys::llama_context {
        self.handle.as_ptr()
    }
}

// Safety: see `LlamaModel` — the context is read-only after init.
unsafe impl Send for LlamaContext<'_> {}
unsafe impl Sync for LlamaContext<'_> {}

impl Drop for LlamaContext<'_> {
    fn drop(&mut self) {
        // Safety: `handle` is exclusively owned and was returned by
        // `llama_new_context_with_model`.
        unsafe { sys::llama_free(self.handle.as_ptr()) };
    }
}

/// Re-export of [`params::LlamaContextParams`].
pub use self::params::LlamaContextParams;
