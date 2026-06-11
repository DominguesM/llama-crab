//! Chat completion driver — concatenates messages into a prompt and runs
//! [`create_completion`].

use crate::error::Result;

use super::completion::create_completion;
use super::Llama;
pub use crate::chat::ChatMessage;

/// Run a single-turn chat completion.
///
/// In v0.1 the messages are concatenated in a simple `role: content` format.
/// Real chat-template rendering lands in v0.2.
pub fn create_chat_completion(
    llama: &mut Llama,
    messages: &[ChatMessage],
    max_tokens: usize,
) -> Result<ChatMessage> {
    let mut prompt = String::new();
    for m in messages {
        prompt.push_str(m.role.as_str());
        prompt.push_str(": ");
        prompt.push_str(&m.content);
        prompt.push('\n');
    }
    prompt.push_str("assistant:");
    let resp = create_completion(llama, &prompt, max_tokens)?;
    Ok(ChatMessage::new(crate::chat::Role::Assistant, resp.text))
}
