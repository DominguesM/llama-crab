//! Speculative decoding abstractions.

/// A draft model used during speculative decoding.
///
/// Implementors produce a sequence of candidate token IDs given the current
/// input. The main model then validates them in a single forward pass.
pub trait DraftModel {
    /// Produce up to `n` candidate tokens for the current input.
    fn draft(&self, input: &[crate::token::LlamaToken], n: usize) -> Vec<crate::token::LlamaToken>;
}

/// N-gram prompt-lookup speculative decoding.
///
/// Repeatedly scans the prompt for the longest matching n-gram and emits the
/// tokens that followed it as the draft.
#[derive(Debug, Clone)]
pub struct PromptLookupDecoding {
    max_ngram_size: usize,
    num_pred_tokens: usize,
}

impl PromptLookupDecoding {
    /// Construct a new prompt-lookup draft model.
    ///
    /// * `max_ngram_size` — maximum length of the n-gram to look up.
    /// * `num_pred_tokens` — how many tokens to emit per draft.
    #[must_use]
    pub const fn new(max_ngram_size: usize, num_pred_tokens: usize) -> Self {
        Self {
            max_ngram_size,
            num_pred_tokens,
        }
    }
}

impl Default for PromptLookupDecoding {
    fn default() -> Self {
        Self::new(2, 10)
    }
}

impl DraftModel for PromptLookupDecoding {
    fn draft(
        &self,
        input: &[crate::token::LlamaToken],
        n: usize,
    ) -> Vec<crate::token::LlamaToken> {
        if input.len() < self.max_ngram_size + 1 || n == 0 {
            return Vec::new();
        }
        // Find the longest matching n-gram suffix.
        for ngram in (1..=self.max_ngram_size.min(input.len() - 1)).rev() {
            let start = input.len() - ngram;
            let pat = &input[start..];
            for candidate_start in 0..=input.len().saturating_sub(ngram + 1) {
                if &input[candidate_start..candidate_start + ngram] == pat {
                    let mut out = Vec::with_capacity(n.min(self.num_pred_tokens));
                    for j in 0..self.num_pred_tokens.min(n) {
                        if let Some(&t) = input.get(candidate_start + ngram + j) {
                            out.push(t);
                        } else {
                            break;
                        }
                    }
                    if !out.is_empty() {
                        return out;
                    }
                }
            }
        }
        Vec::new()
    }
}
