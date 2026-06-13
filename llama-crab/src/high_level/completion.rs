//! Text completion driver.

use serde_json::Value;

use crate::error::{LlamaError, Result};
use crate::token::LlamaToken;

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

/// Options for high-level text completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionOptions {
    /// Maximum number of generated tokens.
    pub max_tokens: usize,
    /// Stop strings that terminate generation without being emitted.
    pub stop_sequences: Vec<String>,
}

impl CompletionOptions {
    /// Create completion options with no stop sequences.
    #[must_use]
    pub const fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            stop_sequences: Vec::new(),
        }
    }

    /// Add one stop sequence.
    #[must_use]
    pub fn with_stop_sequence(mut self, stop_sequence: impl Into<String>) -> Self {
        let stop_sequence = stop_sequence.into();
        if !stop_sequence.is_empty() {
            self.stop_sequences.push(stop_sequence);
        }
        self
    }

    /// Add multiple stop sequences.
    #[must_use]
    pub fn with_stop_sequences<I, S>(mut self, stop_sequences: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.stop_sequences.extend(
            stop_sequences
                .into_iter()
                .map(Into::into)
                .filter(|s: &String| !s.is_empty()),
        );
        self
    }
}

/// A synchronous streaming completion callback chunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionChunk {
    /// Text made available by this callback invocation.
    pub text: String,
    /// Token that produced this chunk, if the chunk corresponds to a sampled token.
    ///
    /// Terminal flush chunks can carry no token when they only report a final
    /// [`StopReason`].
    pub token: Option<LlamaToken>,
    /// Cumulative number of generated non-EOS tokens.
    pub n_tokens: usize,
    /// Terminal reason when this is the last chunk.
    pub stop_reason: Option<StopReason>,
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

/// Return value from synchronous streaming callbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamControl {
    /// Continue generation.
    Continue,
    /// Stop generation after the current callback.
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
    create_completion_with_options(llama, prompt, CompletionOptions::new(max_tokens))
}

/// Generate a single completion for `prompt` using high-level options.
pub fn create_completion_with_options(
    llama: &mut Llama,
    prompt: &str,
    options: CompletionOptions,
) -> Result<Completion> {
    create_completion_stream(llama, prompt, options, |_| StreamControl::Continue)
}

/// Generate a completion and synchronously call `on_chunk` as text becomes
/// available.
pub fn create_completion_stream<F>(
    llama: &mut Llama,
    prompt: &str,
    options: CompletionOptions,
    mut on_chunk: F,
) -> Result<Completion>
where
    F: FnMut(CompletionChunk) -> StreamControl,
{
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
    let mut stop_buffer = StopBuffer::new(options.stop_sequences);
    let mut last_pos = tokens.len() as i32;
    let mut n_generated = 0_usize;
    let mut stop_reason = StopReason::Length;

    for _ in 0..options.max_tokens {
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
        n_generated += 1;
        let step = stop_buffer.push(&piece);
        if step.stopped {
            stop_reason = StopReason::Stop;
            if emit_chunk(
                &mut on_chunk,
                &mut generated,
                step.text,
                Some(next),
                n_generated,
                Some(StopReason::Stop),
            ) == StreamControl::Stop
            {
                stop_reason = StopReason::Stop;
            }
            break;
        }
        if emit_chunk(
            &mut on_chunk,
            &mut generated,
            step.text,
            Some(next),
            n_generated,
            None,
        ) == StreamControl::Stop
        {
            stop_reason = StopReason::Stop;
            break;
        }
        // Feed back the new token.
        let mut single = crate::batch::LlamaBatch::new(1, 1);
        single
            .add(next, last_pos, &[0], true)
            .map_err(LlamaError::from)?;
        llama.context().decode(&single)?;
        last_pos += 1;
    }

    if stop_reason != StopReason::Stop {
        let pending = stop_buffer.flush();
        if emit_chunk(
            &mut on_chunk,
            &mut generated,
            pending,
            None,
            n_generated,
            Some(stop_reason),
        ) == StreamControl::Stop
        {
            stop_reason = StopReason::Stop;
        }
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

fn emit_chunk<F>(
    on_chunk: &mut F,
    generated: &mut String,
    text: String,
    token: Option<LlamaToken>,
    n_tokens: usize,
    stop_reason: Option<StopReason>,
) -> StreamControl
where
    F: FnMut(CompletionChunk) -> StreamControl,
{
    if text.is_empty() && stop_reason.is_none() {
        return StreamControl::Continue;
    }

    generated.push_str(&text);
    on_chunk(CompletionChunk {
        text,
        token,
        n_tokens,
        stop_reason,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StopBuffer {
    pending: String,
    stop_sequences: Vec<String>,
    stopped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StopBufferStep {
    text: String,
    stopped: bool,
}

impl StopBuffer {
    fn new(stop_sequences: Vec<String>) -> Self {
        Self {
            pending: String::new(),
            stop_sequences: stop_sequences
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect(),
            stopped: false,
        }
    }

    fn push(&mut self, text: &str) -> StopBufferStep {
        if self.stopped {
            return StopBufferStep {
                text: String::new(),
                stopped: true,
            };
        }
        if self.stop_sequences.is_empty() {
            return StopBufferStep {
                text: text.to_string(),
                stopped: false,
            };
        }

        self.pending.push_str(text);
        if let Some(stop_start) = self.find_stop_start() {
            self.stopped = true;
            let text = self.pending[..stop_start].to_string();
            self.pending.clear();
            return StopBufferStep {
                text,
                stopped: true,
            };
        }

        let hold_start = self.longest_stop_prefix_suffix_start();
        let text = self.pending[..hold_start].to_string();
        self.pending = self.pending[hold_start..].to_string();
        StopBufferStep {
            text,
            stopped: false,
        }
    }

    fn flush(&mut self) -> String {
        std::mem::take(&mut self.pending)
    }

    fn find_stop_start(&self) -> Option<usize> {
        self.stop_sequences
            .iter()
            .filter_map(|stop| self.pending.find(stop))
            .min()
    }

    fn longest_stop_prefix_suffix_start(&self) -> usize {
        let mut hold_start = self.pending.len();
        for (start, _) in self.pending.char_indices() {
            let suffix = &self.pending[start..];
            if self
                .stop_sequences
                .iter()
                .any(|stop| stop.starts_with(suffix))
            {
                hold_start = start;
                break;
            }
        }
        hold_start
    }
}

#[cfg(test)]
mod tests {
    use super::StopBuffer;

    #[test]
    fn stop_buffer_holds_stop_prefix_across_token_boundaries() {
        let mut buffer = StopBuffer::new(vec!["</stop>".to_string()]);

        let first = buffer.push("hello </");
        assert_eq!(first.text, "hello ");
        assert!(!first.stopped);

        let second = buffer.push("stop> ignored");
        assert_eq!(second.text, "");
        assert!(second.stopped);
    }

    #[test]
    fn stop_buffer_removes_stop_sequence_inside_chunk() {
        let mut buffer = StopBuffer::new(vec!["END".to_string()]);

        let step = buffer.push("answerEND trailing");

        assert_eq!(step.text, "answer");
        assert!(step.stopped);
    }

    #[test]
    fn stop_buffer_flushes_pending_prefix_when_generation_finishes_without_stop() {
        let mut buffer = StopBuffer::new(vec!["foobar".to_string()]);

        let step = buffer.push("hello foo");
        assert_eq!(step.text, "hello ");
        assert!(!step.stopped);

        assert_eq!(buffer.flush(), "foo");
    }
}
