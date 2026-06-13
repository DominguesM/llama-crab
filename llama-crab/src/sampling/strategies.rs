//! All 17 sampling strategies exposed by `llama.cpp` as associated
//! constructors on [`LlamaSampler`].

use llama_crab_sys as sys;

#[allow(unused_imports)]
use super::LlamaSampler;
#[allow(unused_imports)]
use crate::model::LlamaModel;
#[allow(unused_imports)]
use crate::token::LlamaToken;

impl LlamaSampler {
    /// Always pick the highest-probability token.
    #[must_use]
    pub fn greedy() -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_greedy() };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Uniform random sampling with a fixed seed.
    #[must_use]
    pub fn dist(seed: u32) -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_dist(seed) };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Top-K sampling.
    #[must_use]
    pub fn top_k(k: i32) -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_top_k(k) };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Nucleus (top-P) sampling.
    #[must_use]
    pub fn top_p(p: f32, min_keep: usize) -> Option<Self> {
        let ptr = unsafe { sys::llama_sampler_init_top_p(p, min_keep) };
        (!ptr.is_null()).then(|| unsafe { Self::from_raw(ptr) })
    }

    /// Min-P sampling.
    #[must_use]
    pub fn min_p(p: f32, min_keep: usize) -> Option<Self> {
        let ptr = unsafe { sys::llama_sampler_init_min_p(p, min_keep) };
        (!ptr.is_null()).then(|| unsafe { Self::from_raw(ptr) })
    }

    /// Locally-typical sampling.
    #[must_use]
    pub fn typical(p: f32, min_keep: usize) -> Option<Self> {
        let ptr = unsafe { sys::llama_sampler_init_typical(p, min_keep) };
        (!ptr.is_null()).then(|| unsafe { Self::from_raw(ptr) })
    }

    /// Temperature scaling.
    #[must_use]
    pub fn temp(t: f32) -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_temp(t) };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Dynamic temperature (scaled by entropy).
    #[must_use]
    pub fn temp_ext(t: f32, delta: f32, exponent: f32) -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_temp_ext(t, delta, exponent) };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Exclude Top Choices (XTC).
    #[must_use]
    pub fn xtc(p: f32, t: f32, min_keep: usize, seed: u32) -> Option<Self> {
        let ptr = unsafe { sys::llama_sampler_init_xtc(p, t, min_keep, seed) };
        (!ptr.is_null()).then(|| unsafe { Self::from_raw(ptr) })
    }

    /// Top-N-Sigma sampling.
    #[must_use]
    pub fn top_n_sigma(n: f32) -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_top_n_sigma(n) };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Mirostat v1 sampling. Requires the model's vocab size.
    #[must_use]
    pub fn mirostat(n_vocab: i32, seed: u32, tau: f32, eta: f32, m: i32) -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_mirostat(n_vocab, seed, tau, eta, m) };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Mirostat v2 sampling.
    #[must_use]
    pub fn mirostat_v2(seed: u32, tau: f32, eta: f32) -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_mirostat_v2(seed, tau, eta) };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Repetition / frequency / presence penalties.
    #[must_use]
    pub fn penalties(
        penalty_last_n: i32,
        penalty_repeat: f32,
        penalty_freq: f32,
        penalty_present: f32,
    ) -> Option<Self> {
        let p = unsafe {
            sys::llama_sampler_init_penalties(
                penalty_last_n,
                penalty_repeat,
                penalty_freq,
                penalty_present,
            )
        };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Don't Repeat Yourself (DRY) sampler.
    ///
    /// # Safety
    /// `seq_breakers` must outlive the sampler and remain valid.
    pub unsafe fn dry(
        model: &LlamaModel,
        multiplier: f32,
        base: f32,
        allowed_length: i32,
        penalty_last_n: i32,
        seq_breakers: &[&str],
    ) -> Option<Self> {
        let breakers: Vec<std::ffi::CString> = seq_breakers
            .iter()
            .map(|s| std::ffi::CString::new(*s).unwrap())
            .collect();
        let mut ptrs: Vec<*const std::os::raw::c_char> =
            breakers.iter().map(|c| c.as_ptr()).collect();
        let p = unsafe {
            sys::llama_sampler_init_dry(
                model.vocab(),
                model.n_ctx_train() as i32,
                multiplier,
                base,
                allowed_length,
                penalty_last_n,
                ptrs.as_mut_ptr(),
                ptrs.len(),
            )
        };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Adaptive probabilistic sampling.
    #[must_use]
    pub fn adaptive_p(target: f32, decay: f32, seed: u32) -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_adaptive_p(target, decay, seed) };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Logit-bias sampler.
    ///
    /// # Safety
    /// The `biases` slice must outlive the sampler.
    pub unsafe fn logit_bias(n_vocab: i32, biases: &[sys::llama_logit_bias]) -> Option<Self> {
        let p = unsafe {
            sys::llama_sampler_init_logit_bias(n_vocab, biases.len() as i32, biases.as_ptr())
        };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }

    /// Code-infill sampler (requires the model vocab).
    ///
    /// # Safety
    /// `model` must be a valid, live, unaliased `LlamaModel`.
    pub unsafe fn infill(model: &LlamaModel) -> Option<Self> {
        let p = unsafe { sys::llama_sampler_init_infill(model.vocab()) };
        (!p.is_null()).then(|| unsafe { Self::from_raw(p) })
    }
}

/// Required to expose `LlamaModel` to `dry` and `infill` via the
/// `super::LlamaSampler` type — they call `model.vocab()` which is
/// `pub(crate)` on `LlamaModel`.
#[doc(hidden)]
pub struct _ModelTokenBridge(LlamaToken);
