//! Text completion driver.

use crate::error::{LlamaError, Result};

use super::Llama;

/// The result of a text completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    /// Concatenated generated text.
    pub text: String,
    /// Number of tokens generated.
    pub n_tokens: usize,
    /// Reason generation stopped (`"length"`, `"eos"`, or `"stop"`).
    pub stop_reason: StopReason,
}

/// Why a completion ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StopReason {
    /// Reached the requested `max_tokens`.
    Length,
    /// Model emitted an end-of-sequence token.
    Eos,
    /// Custom stop string matched.
    Stop,
}

/// Generate a single completion for `prompt`.
///
/// This is the simplest possible inference loop: tokenize → decode the prompt
/// → sample → decode one token at a time. For more control, use the lower
/// level [`crate::LlamaContext`] / [`crate::sampling::LlamaSampler`] APIs.
pub fn create_completion(llama: &mut Llama, prompt: &str, max_tokens: usize) -> Result<Completion> {
    let tokens = llama.model().tokenize(prompt, true, false)?;

    // Build a batch with the prompt; only the last token produces logits.
    let mut batch = crate::batch::LlamaBatch::new(tokens.len(), 1);
    for (i, &t) in tokens.iter().enumerate() {
        batch
            .add(t, i as i32, &[0], i + 1 == tokens.len())
            .map_err(LlamaError::from)?;
    }
    llama.context().decode(&batch)?;

    // Allocate a sampler — start with greedy, the simplest option.
    let mut sampler = crate::sampling::LlamaSampler::greedy()
        .ok_or_else(|| LlamaError::Batch("sampler_init_greedy returned null".into()))?;

    let ctx_ptr = llama.context().raw();
    let eos = llama.model().token_eos();
    let mut generated = String::new();
    let mut last_pos = tokens.len() as i32;
    let mut n_generated = 0_usize;
    let mut stop_reason = StopReason::Length;

    for _ in 0..max_tokens {
        let next = unsafe { sampler.sample(ctx_ptr, last_pos - 1) };
        sampler.accept(next);
        if next == eos {
            stop_reason = StopReason::Eos;
            break;
        }
        let piece = llama.model().detokenize(&[next], false)?;
        generated.push_str(&piece);
        n_generated += 1;
        // Feed back the new token.
        let mut single = crate::batch::LlamaBatch::new(1, 1);
        single
            .add(next, last_pos, &[0], true)
            .map_err(LlamaError::from)?;
        llama.context().decode(&single)?;
        last_pos += 1;
    }

    Ok(Completion {
        text: generated,
        n_tokens: n_generated,
        stop_reason,
    })
}
