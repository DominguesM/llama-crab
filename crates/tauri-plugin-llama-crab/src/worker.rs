use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

use llama_crab::{
    chat::{
        render_builtin,
        tool_call::{extract_tool_calls, ToolCallStream, ToolFormat},
    },
    Completion, CompletionChunk, CompletionOptions, Llama, LlamaParams, LlamaSampler, LlamaToken,
    StreamControl,
};
use tauri::ipc::Channel;
use uuid::Uuid;

use crate::{
    error::{PluginError, Result},
    models::{
        stop_reason, ChatChoice, ChatChunkChoice, ChatChunkDelta, ChatCompletionChunk,
        ChatCompletionRequest, ChatCompletionResponse, ChatResponseMessage, ChatResponseToolCall,
        ChatResponseToolCallFunction, ChatStreamToolCall, ChatStreamToolCallFunction,
        CompletionChoice, CompletionChunkChoice, CompletionChunkFrame, CompletionLogprobsResponse,
        CompletionRequest, CompletionResponse, DetokenizeRequest, DetokenizeResponse,
        EmbeddingItem, EmbeddingRequest, EmbeddingResponse, EmbeddingUsage, EmbeddingValue,
        RerankRequest, RerankResponse, RerankResult, StructuredRequest, TokenizeCountResponse,
        TokenizeRequest, TokenizeResponse, Usage,
    },
};

#[derive(Debug, Clone)]
pub(crate) struct WorkerHandle {
    tx: mpsc::Sender<WorkerCommand>,
}

enum WorkerCommand {
    Chat {
        request: ChatCompletionRequest,
        reply: mpsc::Sender<Result<ChatCompletionResponse>>,
    },
    ChatStream {
        request_id: String,
        request: ChatCompletionRequest,
        cancel: Arc<AtomicBool>,
        on_chunk: Channel<ChatCompletionChunk>,
        reply: mpsc::Sender<Result<()>>,
    },
    Complete {
        request: CompletionRequest,
        reply: mpsc::Sender<Result<CompletionResponse>>,
    },
    CompleteStream {
        request_id: String,
        request: CompletionRequest,
        cancel: Arc<AtomicBool>,
        on_chunk: Channel<CompletionChunkFrame>,
        reply: mpsc::Sender<Result<()>>,
    },
    Embed {
        request: EmbeddingRequest,
        reply: mpsc::Sender<Result<EmbeddingResponse>>,
    },
    Rerank {
        request: RerankRequest,
        reply: mpsc::Sender<Result<RerankResponse>>,
    },
    Tokenize {
        request: TokenizeRequest,
        reply: mpsc::Sender<Result<TokenizeResponse>>,
    },
    TokenizeCount {
        request: TokenizeRequest,
        reply: mpsc::Sender<Result<TokenizeCountResponse>>,
    },
    Detokenize {
        request: DetokenizeRequest,
        reply: mpsc::Sender<Result<DetokenizeResponse>>,
    },
    Shutdown,
}

impl WorkerHandle {
    pub(crate) fn load(params: LlamaParams) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<WorkerCommand>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<()>>();

        thread::Builder::new()
            .name("llama-crab-model-worker".into())
            .spawn(move || {
                let mut llama = match Llama::load(params) {
                    Ok(llama) => {
                        let _ = ready_tx.send(Ok(()));
                        llama
                    }
                    Err(error) => {
                        let _ = ready_tx.send(Err(PluginError::from(error)));
                        return;
                    }
                };

                while let Ok(command) = rx.recv() {
                    match command {
                        WorkerCommand::Chat { request, reply } => {
                            let _ = reply.send(run_chat(&mut llama, request));
                        }
                        WorkerCommand::ChatStream {
                            request_id,
                            request,
                            cancel,
                            on_chunk,
                            reply,
                        } => {
                            let _ = reply.send(run_chat_stream(
                                &mut llama, request_id, request, cancel, on_chunk,
                            ));
                        }
                        WorkerCommand::Complete { request, reply } => {
                            let _ = reply.send(run_completion(&mut llama, request));
                        }
                        WorkerCommand::CompleteStream {
                            request_id,
                            request,
                            cancel,
                            on_chunk,
                            reply,
                        } => {
                            let _ = reply.send(run_completion_stream(
                                &mut llama, request_id, request, cancel, on_chunk,
                            ));
                        }
                        WorkerCommand::Embed { request, reply } => {
                            let _ = reply.send(run_embedding(&mut llama, request));
                        }
                        WorkerCommand::Rerank { request, reply } => {
                            let _ = reply.send(run_rerank(&mut llama, request));
                        }
                        WorkerCommand::Tokenize { request, reply } => {
                            let _ = reply.send(run_tokenize(&mut llama, request));
                        }
                        WorkerCommand::TokenizeCount { request, reply } => {
                            let _ = reply.send(run_tokenize_count(&mut llama, request));
                        }
                        WorkerCommand::Detokenize { request, reply } => {
                            let _ = reply.send(run_detokenize(&mut llama, request));
                        }
                        WorkerCommand::Shutdown => break,
                    }
                }
            })
            .map_err(|error| PluginError::worker(error.to_string()))?;

        ready_rx
            .recv()
            .map_err(|error| PluginError::worker(error.to_string()))??;
        Ok(Self { tx })
    }

    pub(crate) fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        request_reply(&self.tx, |reply| WorkerCommand::Chat { request, reply })
    }

    pub(crate) fn stream_chat_completion(
        &self,
        request_id: String,
        request: ChatCompletionRequest,
        cancel: Arc<AtomicBool>,
        on_chunk: Channel<ChatCompletionChunk>,
    ) -> Result<()> {
        request_reply(&self.tx, |reply| WorkerCommand::ChatStream {
            request_id,
            request,
            cancel,
            on_chunk,
            reply,
        })
    }

    pub(crate) fn create_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse> {
        request_reply(&self.tx, |reply| WorkerCommand::Complete { request, reply })
    }

    pub(crate) fn stream_completion(
        &self,
        request_id: String,
        request: CompletionRequest,
        cancel: Arc<AtomicBool>,
        on_chunk: Channel<CompletionChunkFrame>,
    ) -> Result<()> {
        request_reply(&self.tx, |reply| WorkerCommand::CompleteStream {
            request_id,
            request,
            cancel,
            on_chunk,
            reply,
        })
    }

    pub(crate) fn create_embedding(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        request_reply(&self.tx, |reply| WorkerCommand::Embed { request, reply })
    }

    pub(crate) fn create_rerank(&self, request: RerankRequest) -> Result<RerankResponse> {
        request_reply(&self.tx, |reply| WorkerCommand::Rerank { request, reply })
    }

    pub(crate) fn tokenize(&self, request: TokenizeRequest) -> Result<TokenizeResponse> {
        request_reply(&self.tx, |reply| WorkerCommand::Tokenize { request, reply })
    }

    pub(crate) fn tokenize_count(&self, request: TokenizeRequest) -> Result<TokenizeCountResponse> {
        request_reply(&self.tx, |reply| WorkerCommand::TokenizeCount {
            request,
            reply,
        })
    }

    pub(crate) fn detokenize(&self, request: DetokenizeRequest) -> Result<DetokenizeResponse> {
        request_reply(&self.tx, |reply| WorkerCommand::Detokenize {
            request,
            reply,
        })
    }

    pub(crate) fn shutdown(&self) {
        let _ = self.tx.send(WorkerCommand::Shutdown);
    }
}

fn request_reply<T>(
    tx: &mpsc::Sender<WorkerCommand>,
    command: impl FnOnce(mpsc::Sender<Result<T>>) -> WorkerCommand,
) -> Result<T> {
    let (reply, rx) = mpsc::channel();
    tx.send(command(reply))
        .map_err(|error| PluginError::worker(error.to_string()))?;
    rx.recv()
        .map_err(|error| PluginError::worker(error.to_string()))?
}

fn run_chat(llama: &mut Llama, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
    if request.has_media() {
        return Err(PluginError::invalid_request(
            "multimodal chat requires a plugin build with mtmd support",
        ));
    }

    let id = new_id("chatcmpl");
    let created = crate::models::unix_timestamp();
    let template = request.template()?;
    let messages = request.llama_messages()?;
    let tools = request.tool_definitions();
    let prompt = render_builtin(template, &messages, &tools, true);
    let prompt_tokens = token_count(llama, &prompt, true, true)?;
    let tool_format = ToolFormat::from_chat_format(template.as_str());
    let mut choices = Vec::new();
    let mut completion_tokens = 0_u32;

    for index in 0..request.choice_count() {
        let completion = create_completion_with_constraints(
            llama,
            &prompt,
            request.completion_options(),
            &request.structured,
        )?;
        completion_tokens += completion.n_tokens as u32;
        choices.push(chat_choice(index as u32, completion, tool_format));
    }

    Ok(ChatCompletionResponse {
        id,
        object: "chat.completion",
        created,
        model: request.model,
        choices,
        usage: Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        },
    })
}

fn chat_choice(index: u32, completion: Completion, tool_format: ToolFormat) -> ChatChoice {
    let parsed_calls = extract_tool_calls(tool_format, &completion.text);
    let tool_calls: Vec<_> = parsed_calls
        .into_iter()
        .filter_map(std::result::Result::ok)
        .map(|call| ChatResponseToolCall {
            id: call.id,
            kind: "function",
            function: ChatResponseToolCallFunction {
                name: call.name,
                arguments: call.arguments.to_string(),
            },
        })
        .collect();
    let finish = if tool_calls.is_empty() {
        stop_reason(completion.stop_reason).to_string()
    } else {
        "tool_calls".to_string()
    };

    ChatChoice {
        index,
        message: ChatResponseMessage {
            role: "assistant",
            content: tool_calls.is_empty().then_some(completion.text),
            tool_calls,
        },
        finish_reason: Some(finish),
        logprobs: completion.logprobs.map(CompletionLogprobsResponse::from),
    }
}

fn run_chat_stream(
    llama: &mut Llama,
    request_id: String,
    request: ChatCompletionRequest,
    cancel: Arc<AtomicBool>,
    on_chunk: Channel<ChatCompletionChunk>,
) -> Result<()> {
    if request.has_media() {
        return Err(PluginError::invalid_request(
            "multimodal chat requires a plugin build with mtmd support",
        ));
    }

    let id = new_id("chatcmpl");
    let created = crate::models::unix_timestamp();
    let template = request.template()?;
    let messages = request.llama_messages()?;
    let tools = request.tool_definitions();
    let tool_format = ToolFormat::from_chat_format(template.as_str());
    let mut tool_stream = ToolCallStream::new(tool_format);

    send_chat_chunk(
        &on_chunk,
        ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk",
            created,
            model: request.model.clone(),
            choices: vec![ChatChunkChoice {
                index: 0,
                delta: ChatChunkDelta {
                    role: Some("assistant"),
                    content: None,
                    tool_calls: Vec::new(),
                },
                finish_reason: None,
            }],
            usage: None,
            request_id: Some(request_id.clone()),
        },
    )?;

    let prompt = render_builtin(template, &messages, &tools, true);
    let result = create_completion_stream_with_constraints(
        llama,
        &prompt,
        request.completion_options(),
        &request.structured,
        |chunk| {
            let tool_deltas = tool_stream.feed(&chunk.text);
            let tool_calls = tool_deltas
                .into_iter()
                .map(|delta| ChatStreamToolCall {
                    index: delta.index,
                    id: delta.id,
                    kind: Some("function").filter(|_| delta.name.is_some()),
                    function: ChatStreamToolCallFunction {
                        name: delta.name,
                        arguments: delta.arguments,
                    },
                })
                .collect::<Vec<_>>();

            if (!chunk.text.is_empty() && !tool_stream.in_call()) || !tool_calls.is_empty() {
                let _ = on_chunk.send(ChatCompletionChunk {
                    id: id.clone(),
                    object: "chat.completion.chunk",
                    created,
                    model: request.model.clone(),
                    choices: vec![ChatChunkChoice {
                        index: 0,
                        delta: ChatChunkDelta {
                            role: None,
                            content: (!tool_stream.in_call() && !chunk.text.is_empty())
                                .then_some(chunk.text.clone()),
                            tool_calls,
                        },
                        finish_reason: None,
                    }],
                    usage: None,
                    request_id: Some(request_id.clone()),
                });
            }

            if cancel.load(Ordering::Relaxed) {
                StreamControl::Stop
            } else {
                StreamControl::Continue
            }
        },
    );

    let finish_reason = result
        .as_ref()
        .ok()
        .map(|message| {
            if message.text.is_empty() {
                "stop"
            } else if tool_stream.completed_count() > 0 {
                "tool_calls"
            } else {
                "stop"
            }
        })
        .unwrap_or("stop")
        .to_string();

    for delta in tool_stream.finish() {
        let _ = on_chunk.send(ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk",
            created,
            model: request.model.clone(),
            choices: vec![ChatChunkChoice {
                index: 0,
                delta: ChatChunkDelta {
                    role: None,
                    content: None,
                    tool_calls: vec![ChatStreamToolCall {
                        index: delta.index,
                        id: delta.id,
                        kind: Some("function"),
                        function: ChatStreamToolCallFunction {
                            name: delta.name,
                            arguments: delta.arguments,
                        },
                    }],
                },
                finish_reason: None,
            }],
            usage: None,
            request_id: Some(request_id.clone()),
        });
    }

    send_chat_chunk(
        &on_chunk,
        ChatCompletionChunk {
            id,
            object: "chat.completion.chunk",
            created,
            model: request.model,
            choices: vec![ChatChunkChoice {
                index: 0,
                delta: ChatChunkDelta::default(),
                finish_reason: Some(finish_reason),
            }],
            usage: None,
            request_id: Some(request_id),
        },
    )?;

    result.map(|_| ()).map_err(PluginError::from)
}

fn run_completion(llama: &mut Llama, request: CompletionRequest) -> Result<CompletionResponse> {
    let id = new_id("cmpl");
    let created = crate::models::unix_timestamp();
    let mut choices = Vec::new();
    let mut prompt_tokens = 0_u32;
    let mut completion_tokens = 0_u32;
    let mut choice_index = 0_u32;

    for prompt in request.prompts() {
        prompt_tokens += token_count(llama, &prompt, true, true)?;
        for _ in 0..request.choice_count() {
            let completion = create_completion_with_constraints(
                llama,
                &prompt,
                request.completion_options(),
                &request.structured,
            )?;
            completion_tokens += completion.n_tokens as u32;
            choices.push(CompletionChoice {
                text: completion.text,
                index: choice_index,
                finish_reason: Some(stop_reason(completion.stop_reason).into()),
                logprobs: completion.logprobs.map(CompletionLogprobsResponse::from),
            });
            choice_index += 1;
        }
    }

    Ok(CompletionResponse {
        id,
        object: "text_completion",
        created,
        model: request.model,
        choices,
        usage: Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        },
    })
}

fn run_completion_stream(
    llama: &mut Llama,
    request_id: String,
    request: CompletionRequest,
    cancel: Arc<AtomicBool>,
    on_chunk: Channel<CompletionChunkFrame>,
) -> Result<()> {
    let prompts = request.prompts();
    if prompts.len() != 1 || request.choice_count() != 1 {
        return Err(PluginError::invalid_request(
            "streaming completions currently require one prompt and n=1",
        ));
    }
    let prompt = prompts.into_iter().next().unwrap_or_default();
    let id = new_id("cmpl");
    let created = crate::models::unix_timestamp();
    let model = request.model.clone();

    let result = create_completion_stream_with_constraints(
        llama,
        &prompt,
        request.completion_options(),
        &request.structured,
        |chunk| {
            if !chunk.text.is_empty() || chunk.stop_reason.is_some() {
                let _ = on_chunk.send(completion_chunk_frame(
                    &id,
                    created,
                    &model,
                    &request_id,
                    chunk,
                ));
            }
            if cancel.load(Ordering::Relaxed) {
                StreamControl::Stop
            } else {
                StreamControl::Continue
            }
        },
    );

    result.map(|_| ()).map_err(PluginError::from)
}

fn completion_chunk_frame(
    id: &str,
    created: u64,
    model: &str,
    request_id: &str,
    chunk: CompletionChunk,
) -> CompletionChunkFrame {
    CompletionChunkFrame {
        id: id.into(),
        object: "text_completion.chunk",
        created,
        model: model.into(),
        choices: vec![CompletionChunkChoice {
            text: chunk.text,
            index: 0,
            finish_reason: chunk.stop_reason.map(|reason| stop_reason(reason).into()),
            logprobs: chunk.logprobs.map(CompletionLogprobsResponse::from),
        }],
        usage: None,
        request_id: Some(request_id.into()),
    }
}

fn create_completion_with_constraints(
    llama: &mut Llama,
    prompt: &str,
    options: CompletionOptions,
    structured: &StructuredRequest,
) -> Result<Completion> {
    let Some((grammar, root)) = structured.grammar_text()? else {
        return llama
            .create_completion_with_options(prompt, options)
            .map_err(PluginError::from);
    };
    let mut sampler = constrained_sampler(llama, &options, &grammar, &root)?;
    llama
        .create_completion_with_sampler(prompt, options, &mut sampler)
        .map_err(PluginError::from)
}

fn create_completion_stream_with_constraints<F>(
    llama: &mut Llama,
    prompt: &str,
    options: CompletionOptions,
    structured: &StructuredRequest,
    on_chunk: F,
) -> Result<Completion>
where
    F: FnMut(CompletionChunk) -> StreamControl,
{
    let Some((grammar, root)) = structured.grammar_text()? else {
        return llama
            .create_completion_stream(prompt, options, on_chunk)
            .map_err(PluginError::from);
    };
    let mut sampler = constrained_sampler(llama, &options, &grammar, &root)?;
    llama
        .create_completion_stream_with_sampler(prompt, options, &mut sampler, on_chunk)
        .map_err(PluginError::from)
}

fn constrained_sampler(
    llama: &Llama,
    options: &CompletionOptions,
    grammar: &str,
    root: &str,
) -> Result<LlamaSampler> {
    let grammar_sampler = unsafe { LlamaSampler::grammar(llama.model(), grammar, root) }
        .map_err(|error| PluginError::invalid_request(error.to_string()))?;
    let base_sampler = options.build_sampler(llama)?;
    LlamaSampler::chain(vec![grammar_sampler, base_sampler], false)
        .ok_or_else(|| PluginError::worker("sampler_chain_init returned null"))
}

fn run_embedding(llama: &mut Llama, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
    let format = request.encoding_format.as_deref().unwrap_or("float");
    if !matches!(format, "float" | "base64") {
        return Err(PluginError::invalid_request(
            "encoding_format must be `float` or `base64`",
        ));
    }

    let mut prompt_tokens = 0_u32;
    let mut data = Vec::new();
    for (index, text) in request.input.texts().into_iter().enumerate() {
        prompt_tokens += token_count(llama, &text, true, false)?;
        let embedding = llama.embed(&text, request.normalize)?;
        data.push(EmbeddingItem {
            object: "embedding",
            embedding: if format == "base64" {
                EmbeddingValue::Base64(base64_f32(&embedding))
            } else {
                EmbeddingValue::Float(embedding)
            },
            index: index as u32,
            encoding_format: (format == "base64").then_some("base64".into()),
        });
    }

    Ok(EmbeddingResponse {
        object: "list",
        data,
        model: request.model,
        usage: EmbeddingUsage {
            prompt_tokens,
            total_tokens: prompt_tokens,
        },
    })
}

fn run_rerank(llama: &mut Llama, request: RerankRequest) -> Result<RerankResponse> {
    let docs: Vec<&str> = request.documents.iter().map(String::as_str).collect();
    let scores = llama.rerank(&request.query, &docs)?;
    let mut results = request
        .documents
        .iter()
        .cloned()
        .zip(scores)
        .enumerate()
        .map(|(index, (document, relevance_score))| RerankResult {
            index: index as u32,
            document,
            relevance_score,
        })
        .collect::<Vec<_>>();
    results.sort_by(|left, right| right.relevance_score.total_cmp(&left.relevance_score));
    if let Some(top_n) = request.top_n {
        results.truncate(top_n);
    }
    Ok(RerankResponse {
        model: request.model,
        results,
    })
}

fn run_tokenize(llama: &mut Llama, request: TokenizeRequest) -> Result<TokenizeResponse> {
    Ok(TokenizeResponse {
        tokens: llama
            .model()
            .tokenize(&request.input, true, false)?
            .into_iter()
            .map(LlamaToken::raw)
            .collect(),
    })
}

fn run_tokenize_count(
    llama: &mut Llama,
    request: TokenizeRequest,
) -> Result<TokenizeCountResponse> {
    Ok(TokenizeCountResponse {
        count: llama.model().tokenize(&request.input, true, false)?.len(),
    })
}

fn run_detokenize(llama: &mut Llama, request: DetokenizeRequest) -> Result<DetokenizeResponse> {
    let tokens = request
        .tokens
        .into_iter()
        .map(LlamaToken::from)
        .collect::<Vec<_>>();
    Ok(DetokenizeResponse {
        text: llama.model().detokenize(&tokens, false)?,
    })
}

fn send_chat_chunk(
    on_chunk: &Channel<ChatCompletionChunk>,
    chunk: ChatCompletionChunk,
) -> Result<()> {
    on_chunk
        .send(chunk)
        .map_err(|error| PluginError::worker(error.to_string()))
}

fn token_count(llama: &Llama, text: &str, add_bos: bool, special: bool) -> Result<u32> {
    Ok(llama.model().tokenize(text, add_bos, special)?.len() as u32)
}

fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::new_v4())
}

fn base64_f32(values: &[f32]) -> String {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    base64_encode(&bytes)
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
