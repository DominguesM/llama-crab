//! Streaming state for the OpenAI-compat chat-template parser.
//!
//! In v0.1 the streaming side is fully handled by the C-ABI shim
//! exposed by `llama-crab-sys::llama_rs_chat_parse_state_*` (linked in
//! from `wrappers/oaicompat.cpp` when the `common` feature is enabled).
//! The Rust side exposes a thin convenience wrapper that hides the
//! raw C pointers and gives a safe iterator-style interface.
//!
//! For tool-call streaming (different concern, same theme), see
//! [`super::tool_call::ToolParser`].

use serde_json::Value;

use crate::error::{LlamaError, Result};

/// A streaming JSON parser for the OAI-compat chat-template output.
///
/// Currently the Rust side just buffers the entire input and parses
/// it as a single JSON document; the C++ shim has a token-by-token
/// streaming variant. This wrapper keeps the public surface stable
/// while we work on a pure-Rust streaming implementation.
#[derive(Debug, Default)]
pub struct ChatParseState {
    buffer: String,
}

impl ChatParseState {
    /// Construct a fresh empty state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Feed a chunk of model output and obtain the most recent parsed
    /// message (if any).
    pub fn feed(&mut self, chunk: &str) -> Result<Option<Value>> {
        self.buffer.push_str(chunk);
        // Try to parse incrementally by looking for a complete JSON
        // object starting at the last "{" before the closing "}".
        if let Some(end) = self.find_object_end() {
            let candidate = &self.buffer[..=end];
            if let Ok(v) = serde_json::from_str::<Value>(candidate) {
                return Ok(Some(v));
            }
        }
        Ok(None)
    }

    /// Finalize the state and return the last parsed value, if any.
    pub fn finish(&mut self) -> Result<Option<Value>> {
        let buf = std::mem::take(&mut self.buffer);
        if buf.trim().is_empty() {
            return Ok(None);
        }
        match serde_json::from_str(&buf) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Ok(None),
        }
    }

    /// Find the index of the closing `}` of the **last** balanced
    /// top-level `{ … }` substring.
    fn find_object_end(&self) -> Option<usize> {
        let bytes = self.buffer.as_bytes();
        let mut depth: i32 = 0;
        let mut last_close: Option<usize> = None;
        for (i, &b) in bytes.iter().enumerate() {
            match b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        last_close = Some(i);
                    }
                }
                _ => {}
            }
        }
        last_close
    }
}

/// Convert a parser error into a [`crate::error::LlamaError`].
pub fn parse_error(e: serde_json::Error) -> LlamaError {
    LlamaError::JsonSchemaToGrammar(format!("chat parser: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_state() {
        let mut s = ChatParseState::new();
        assert!(s.feed("").unwrap().is_none());
        assert!(s.finish().unwrap().is_none());
    }

    #[test]
    fn single_object_incremental() {
        let mut s = ChatParseState::new();
        assert!(s.feed(r#"{"role":"#).unwrap().is_none());
        let v = s
            .feed(r#""assistant","content":"hi"}"#)
            .unwrap()
            .expect("should parse a complete object");
        assert_eq!(v, json!({"role": "assistant", "content": "hi"}));
    }

    #[test]
    fn nested_object() {
        let mut s = ChatParseState::new();
        let v = s
            .feed(r#"{"outer": {"inner": 1}, "tail": "x"}"#)
            .unwrap()
            .expect("nested object should parse");
        assert_eq!(v, json!({"outer": {"inner": 1}, "tail": "x"}));
    }
}
