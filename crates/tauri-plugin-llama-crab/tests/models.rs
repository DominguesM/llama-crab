//! DTO contract tests for the Tauri plugin.

use tauri_plugin_llama_crab::{
    ChatCompletionRequest, ChatMessage, CompletionRequest, EmbeddingInput, EmbeddingRequest,
    LoadModelRequest, MobilePresetName, ModelKind, PoolingName, RerankRequest, TokenizeRequest,
};

#[test]
fn dto_deserialization_accepts_camel_case_contract() {
    let payload = serde_json::json!({
        "id": "local",
        "path": "/models/tiny.gguf",
        "mobilePreset": "balanced",
        "nCtx": 2048,
        "nGpuLayers": 32,
        "nThreads": 4,
        "useMmap": true
    });

    let request: LoadModelRequest = serde_json::from_value(payload).unwrap();

    assert_eq!(request.id.as_deref(), Some("local"));
    assert_eq!(request.mobile_preset, Some(MobilePresetName::Balanced));
    assert_eq!(request.n_ctx, Some(2048));
    assert_eq!(request.n_gpu_layers, Some(32));
    assert_eq!(request.n_threads, Some(4));
    assert_eq!(request.use_mmap, Some(true));
}

#[test]
fn load_model_request_accepts_runtime_feature_options() {
    let request: LoadModelRequest = serde_json::from_value(serde_json::json!({
        "id": "embedder",
        "path": "/models/e.gguf",
        "kind": "embedding",
        "pooling": "mean",
        "embeddings": true,
        "mmprojPath": "/models/mmproj.gguf",
        "nBatch": 256,
        "nUbatch": 128,
        "flashAttn": true,
        "offloadKqv": false
    }))
    .unwrap();

    assert_eq!(request.kind, Some(ModelKind::Embedding));
    assert_eq!(request.pooling, Some(PoolingName::Mean));
    assert_eq!(request.embeddings, Some(true));
    assert_eq!(request.mmproj_path.as_deref(), Some("/models/mmproj.gguf"));
}

#[test]
fn chat_completion_request_builds_rich_completion_options() {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "local",
        "messages": [{"role": "user", "content": "hello"}],
        "maxTokens": 64,
        "temperature": 0.4,
        "topK": 20,
        "topP": 0.8,
        "frequencyPenalty": 0.1,
        "presencePenalty": 0.2,
        "stop": ["END"],
        "logprobs": true,
        "topLogprobs": 2,
        "n": 2,
        "grammar": "root ::= object",
        "grammarRoot": "root"
    }))
    .unwrap();

    let options = request.completion_options();
    assert_eq!(request.n, Some(2));
    assert_eq!(options.max_tokens, 64);
    assert_eq!(options.sampling.temperature, 0.4);
    assert_eq!(options.sampling.top_k, 20);
    assert_eq!(options.sampling.top_p, 0.8);
    assert_eq!(options.sampling.frequency_penalty, 0.1);
    assert_eq!(options.sampling.presence_penalty, 0.2);
    assert_eq!(options.stop_sequences, vec!["END"]);
    assert_eq!(options.logprobs, Some(2));
}

#[test]
fn completion_embedding_rerank_and_extras_deserialize() {
    let completion: CompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "local",
        "prompt": ["A", "B"],
        "maxTokens": 8,
        "echo": true,
        "suffix": "!",
        "logprobs": 2
    }))
    .unwrap();
    assert_eq!(completion.prompts(), vec!["A", "B"]);
    assert!(completion.completion_options().echo_prompt);

    let embedding: EmbeddingRequest = serde_json::from_value(serde_json::json!({
        "model": "embedder",
        "input": ["one", "two"],
        "encodingFormat": "base64",
        "normalize": false
    }))
    .unwrap();
    assert_eq!(
        embedding.input,
        EmbeddingInput::Many(vec!["one".into(), "two".into()])
    );
    assert_eq!(embedding.encoding_format.as_deref(), Some("base64"));

    let rerank: RerankRequest = serde_json::from_value(serde_json::json!({
        "model": "reranker",
        "query": "q",
        "documents": ["a", "b"],
        "topN": 1
    }))
    .unwrap();
    assert_eq!(rerank.top_n, Some(1));

    let tokenize: TokenizeRequest = serde_json::from_value(serde_json::json!({
        "model": "local",
        "input": "hello"
    }))
    .unwrap();
    assert_eq!(tokenize.input, "hello");
}

#[test]
fn generate_request_builds_completion_options() {
    let request: CompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "local",
        "prompt": "hello",
        "maxTokens": 12,
        "temperature": 0.1,
        "topP": 0.9,
        "topK": 20,
        "stop": ["</s>"],
        "seed": 42
    }))
    .unwrap();

    let options = request.completion_options();

    assert_eq!(options.max_tokens, 12);
    assert_eq!(options.sampling.temperature, 0.1);
    assert_eq!(options.sampling.top_p, 0.9);
    assert_eq!(options.sampling.top_k, 20);
    assert_eq!(options.stop_sequences, vec!["</s>"]);
    assert_eq!(options.sampling.seed, Some(42));
}

#[test]
fn chat_request_parses_roles_and_template() {
    let payload = serde_json::json!({
        "model": "local",
        "messages": [{ "role": "user", "content": "hello" }],
        "template": "chatml",
        "maxTokens": 8
    });

    let request: ChatCompletionRequest = serde_json::from_value(payload).unwrap();
    let messages = request.llama_messages().unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].role.as_str(), "user");
    assert_eq!(request.template().unwrap().as_str(), "chatml");
    assert_eq!(request.completion_options().max_tokens, 8);
}

#[test]
fn chat_message_rejects_unknown_roles() {
    let message: ChatMessage = serde_json::from_value(serde_json::json!({
        "role": "invalid",
        "content": "hidden"
    }))
    .unwrap();

    assert!(message.to_llama_message().is_err());
}
