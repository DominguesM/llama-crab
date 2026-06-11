//! Sampling strategies and the [`LlamaSampler`] wrapper.
//!
//! All 17 sampling strategies exposed by `llama.cpp` are available as
//! associated functions on [`LlamaSampler`]. Use [`LlamaSampler::chain`] to
//! compose them into a pipeline (matching `llama-cpp-python`'s `LlamaSampler`).

use llama_crab_sys as sys;

use crate::model::LlamaModel;
use crate::token::LlamaToken;
use crate::token_data::LlamaTokenDataArray;

/// One-shot strategy. Use the associated constructors below.
///
/// `LlamaSampler` is **not** `Send`/`Sync`: llama.cpp samplers carry mutable
/// state (seeds, history) that is not safe to share across threads without
/// external synchronization.
#[derive(Debug)]
pub struct LlamaSampler {
    handle: *mut sys::llama_sampler,
}

impl LlamaSampler {
    // -- Construction ----------------------------------------------------

    /// Wrap a raw `*mut llama_sampler` produced by an `llama_sampler_init_*` call.
    ///
    /// # Safety
    /// The pointer must be a valid, non-null, non-aliased sampler returned by
    /// llama.cpp. After construction, this type takes ownership and frees the
    /// sampler in `Drop`.
    pub(crate) unsafe fn from_raw(ptr: *mut sys::llama_sampler) -> Self {
        Self { handle: ptr }
    }

    /// Construct a [`LlamaSampler`] from an already-initialized raw pointer
    /// *without* taking ownership. The returned sampler must NOT be dropped
    /// (used internally for cloning via `llama_sampler_clone`).
    pub(crate) unsafe fn from_raw_borrowed(ptr: *mut sys::llama_sampler) -> Self {
        Self { handle: ptr }
    }

    /// Borrow the underlying pointer (used by the C API).
    pub(crate) fn as_ptr(&self) -> *mut sys::llama_sampler {
        self.handle
    }

    // -- Core operations -------------------------------------------------

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

    // -- Strategy constructors -------------------------------------------

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

    /// GBNF grammar constrained sampler.
    ///
    /// # Safety
    /// The `grammar` and `grammar_root` C strings must outlive the sampler.
    #[cfg(feature = "common")]
    pub unsafe fn grammar(model: &LlamaModel, grammar: &str, grammar_root: &str) -> Option<Self> {
        let grammar = std::ffi::CString::new(grammar).ok()?;
        let root = std::ffi::CString::new(grammar_root).ok()?;
        let p = sys::llama_sampler_init_grammar(model.raw(), grammar.as_ptr(), root.as_ptr());
        (!p.is_null()).then(|| Self::from_raw(p))
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
        let mut ptrs: Vec<*const std::os::raw::c_char> = breakers.iter().map(|c| c.as_ptr()).collect();
        let p = sys::llama_sampler_init_dry(
            model.vocab(),
            model.n_ctx_train() as i32,
            multiplier,
            base,
            allowed_length,
            penalty_last_n,
            ptrs.as_mut_ptr(),
            ptrs.len(),
        );
        (!p.is_null()).then(|| Self::from_raw(p))
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
    pub unsafe fn logit_bias(
        n_vocab: i32,
        biases: &[sys::llama_logit_bias],
    ) -> Option<Self> {
        let p = sys::llama_sampler_init_logit_bias(n_vocab, biases.len() as i32, biases.as_ptr());
        (!p.is_null()).then(|| Self::from_raw(p))
    }

    /// Code-infill sampler (requires the model vocab).
    ///
    /// # Safety
    /// `model` must be a valid, live, unaliased `LlamaModel`.
    pub unsafe fn infill(model: &LlamaModel) -> Option<Self> {
        let p = sys::llama_sampler_init_infill(model.vocab());
        (!p.is_null()).then(|| Self::from_raw(p))
    }

    // -- Composition -----------------------------------------------------

    /// Compose a chain of samplers into a single sampler. The order matters:
    /// each stage sees the candidates as transformed by the previous stage.
    ///
    /// The inner samplers are consumed (their raw pointers are moved into the
    /// chain), so calling `chain.add(...)` invalidates them.
    #[must_use]
    pub fn chain(samplers: Vec<LlamaSampler>, no_perf: bool) -> Option<Self> {
        let mut chain_params = unsafe { sys::llama_sampler_chain_default_params() };
        chain_params.no_perf = no_perf;
        let chain = unsafe { sys::llama_sampler_chain_init(chain_params) };
        if chain.is_null() {
            return None;
        }
        for mut s in samplers {
            unsafe { sys::llama_sampler_chain_add(chain, s.handle) };
            // The chain now owns the inner sampler; prevent double-free.
            s.handle = std::ptr::null_mut();
        }
        Some(unsafe { Self::from_raw(chain) })
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

/// Builder for a sampler chain.
///
/// Convenience wrapper around [`LlamaSampler::chain`] with a fluent API.
///
/// # Example
///
/// ```no_run
/// use llama_crab::sampling::SamplerChain;
/// let chain = SamplerChain::new()
///     .temp(0.8)
///     .top_p(0.95, 1)
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct SamplerChain {
    samplers: Vec<LlamaSampler>,
    no_perf: bool,
}

impl SamplerChain {
    /// Construct a new empty chain.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            samplers: Vec::new(),
            no_perf: false,
        }
    }

    /// Disable performance counters in the chain.
    #[must_use]
    pub const fn with_no_perf(mut self, yes: bool) -> Self {
        self.no_perf = yes;
        self
    }

    /// Add a temperature stage.
    #[must_use]
    pub fn temp(mut self, t: f32) -> Self {
        if let Some(s) = LlamaSampler::temp(t) {
            self.samplers.push(s);
        }
        self
    }

    /// Add a top-K stage.
    #[must_use]
    pub fn top_k(mut self, k: i32) -> Self {
        if let Some(s) = LlamaSampler::top_k(k) {
            self.samplers.push(s);
        }
        self
    }

    /// Add a top-P stage.
    #[must_use]
    pub fn top_p(mut self, p: f32, min_keep: usize) -> Self {
        if let Some(s) = LlamaSampler::top_p(p, min_keep) {
            self.samplers.push(s);
        }
        self
    }

    /// Add a min-P stage.
    #[must_use]
    pub fn min_p(mut self, p: f32, min_keep: usize) -> Self {
        if let Some(s) = LlamaSampler::min_p(p, min_keep) {
            self.samplers.push(s);
        }
        self
    }

    /// Add a penalties stage.
    #[must_use]
    pub fn penalties(
        mut self,
        last_n: i32,
        repeat: f32,
        freq: f32,
        present: f32,
    ) -> Self {
        if let Some(s) = LlamaSampler::penalties(last_n, repeat, freq, present) {
            self.samplers.push(s);
        }
        self
    }

    /// Add a greedy sampler.
    #[must_use]
    pub fn greedy(mut self) -> Self {
        if let Some(s) = LlamaSampler::greedy() {
            self.samplers.push(s);
        }
        self
    }

    /// Consume the chain and return a single [`LlamaSampler`].
    #[must_use]
    pub fn build(self) -> Option<LlamaSampler> {
        LlamaSampler::chain(self.samplers, self.no_perf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greedy_does_not_panic() {
        let s = LlamaSampler::greedy();
        assert!(s.is_some());
    }

    #[test]
    fn dist_with_seed() {
        let s = LlamaSampler::dist(42);
        assert!(s.is_some());
    }

    #[test]
    fn chain_constructs() {
        let chain = SamplerChain::new()
            .temp(0.8)
            .top_p(0.95, 1)
            .greedy()
            .build();
        assert!(chain.is_some());
    }
}
