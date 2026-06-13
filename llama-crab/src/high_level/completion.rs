//! Text completion driver.

use serde_json::Value;

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
///
/// The KV cache for sequence 0 is cleared before each call, so each call is
/// independent (matching `llama-cpp-python` semantics). For multi-turn
/// conversations, build the full history into the prompt and call again.
pub fn create_completion(llama: &mut Llama, prompt: &str, max_tokens: usize) -> Result<Completion> {
    // Clear sequence 0 so the new batch can start at position 0 regardless
    // of any previous decode. p0 = p1 = -1 means "the entire range".
    let _ = llama.context().seq_rm(0, -1, -1);

    let tokens = llama.model().tokenize(prompt, true, true)?;

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

    let ctx_ptr = llama.context().raw_handle();
    let eos = llama.model().token_eos();
    let eot = llama.model().token_eot();
    let mut generated = String::new();
    let mut last_pos = tokens.len() as i32;
    let mut n_generated = 0_usize;
    let mut stop_reason = StopReason::Length;

    for _ in 0..max_tokens {
        // `idx` is the index in the *current* batch whose logits we sample
        // from. For the initial prompt (all tokens in one batch) the logits
        // are at the last position. For every subsequent single-token batch
        // the logits are at index 0.
        let idx = if n_generated == 0 {
            (tokens.len() as i32) - 1
        } else {
            0
        };
        let next = unsafe { sampler.sample(ctx_ptr, idx) };
        sampler.accept(next);
        if next == eos || next == eot {
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

/// Create a GBNF grammar from a JSON Schema (used with `LlamaSampler::grammar`).
///
/// # Example
///
/// ```no_run
/// use llama_crab::high_level::completion::json_schema_grammar;
/// use serde_json::json;
/// let grammar = json_schema_grammar(&json!({
///     "type": "object",
///     "properties": {
///         "answer": { "type": "string" }
///     },
///     "required": ["answer"]
/// })).unwrap();
/// assert!(grammar.contains("root"));
/// ```
pub fn json_schema_grammar(schema: &Value) -> Result<String> {
    crate::json_schema::schema_to_grammar(schema, "root")
        .map_err(|e| LlamaError::JsonSchemaToGrammar(e.to_string()))
}
