//! Sampling strategies and the [`LlamaSampler`] wrapper.

use llama_crab_sys as sys;

use crate::token::LlamaToken;
use crate::token_data::LlamaTokenDataArray;

/// Wrapper around `*mut llama_sampler`.
///
/// Not `Send`/`Sync`: llama.cpp samplers carry mutable state (seeds, history)
/// that is not safe to share across threads without external synchronization.
#[derive(Debug)]
pub struct LlamaSampler {
    handle: *mut sys::llama_sampler,
}

impl LlamaSampler {
    /// Wrap a raw `*mut llama_sampler` produced by an `llama_sampler_init_*` call.
    ///
    /// # Safety
    /// The pointer must be a valid, non-null, non-aliased sampler returned by
    /// llama.cpp. After construction, this type takes ownership and frees the
    /// sampler in `Drop`.
    pub(crate) unsafe fn from_raw(ptr: *mut sys::llama_sampler) -> Self {
        Self { handle: ptr }
    }

    /// Borrow the underlying pointer (used by the C API).
    pub(crate) fn as_ptr(&self) -> *mut sys::llama_sampler {
        self.handle
    }

    /// Sample a single token from the context's logits.
    ///
    /// # Safety
    /// `ctx` must point to a live, unaliased `llama_context`.
    pub unsafe fn sample(&mut self, ctx: *mut sys::llama_context, idx: i32) -> LlamaToken {
        // Safety: caller guarantees `ctx` is a valid, live, unaliased context.
        let raw = sys::llama_sampler_sample(self.handle, ctx, idx);
        LlamaToken(raw)
    }

    /// Apply the sampler to a [`LlamaTokenDataArray`].
    pub fn apply(&self, candidates: &mut LlamaTokenDataArray) {
        unsafe {
            sys::llama_sampler_apply(self.handle, candidates.as_mut_ptr());
        }
    }

    /// Reset internal sampler state (seeds, history, etc.).
    pub fn reset(&mut self) {
        unsafe { sys::llama_sampler_reset(self.handle) };
    }

    /// Accept a sampled token into the sampler's history.
    pub fn accept(&mut self, token: LlamaToken) {
        unsafe { sys::llama_sampler_accept(self.handle, token.0) };
    }

    /// Get the random seed used by the sampler.
    #[must_use]
    pub fn get_seed(&self) -> u32 {
        unsafe { sys::llama_sampler_get_seed(self.handle) }
    }

    /// Construct a greedy sampler.
    #[must_use]
    pub fn greedy() -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_greedy() };
        if p.is_null() {
            None
        } else {
            Some(unsafe { Self::from_raw(p) })
        }
    }

    /// Construct a uniform random sampler with the given seed.
    #[must_use]
    pub fn dist(seed: u32) -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_dist(seed) };
        if p.is_null() {
            None
        } else {
            Some(unsafe { Self::from_raw(p) })
        }
    }
}

impl Drop for LlamaSampler {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // Safety: `self.handle` was created by llama_sampler_init_* and
            // we own it.
            unsafe { sys::llama_sampler_free(self.handle) };
        }
    }
}

/// Builder for a chain of samplers.
///
/// In v0.1 this is a placeholder; the proper chain API lands in v0.2.
#[derive(Debug, Default)]
pub struct SamplerChain;

impl SamplerChain {
    /// Construct a new empty chain.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Append `sampler` to the chain (stub — implementation deferred).
    pub fn add(self, _sampler: LlamaSampler) {
        // Implementation deferred — this stub keeps the API surface for v0.1.
    }
}
