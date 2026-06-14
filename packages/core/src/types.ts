export type ChatCompletionRole = "developer" | "system" | "user" | "assistant" | "tool"

export type ChatCompletionContentPartText = {
  type: "text"
  text: string
}

export type ChatCompletionContentPartImage = {
  type: "image_url"
  image_url: {
    url: string
    detail?: "auto" | "low" | "high"
  }
}

export type ChatCompletionContentPartInputAudio = {
  type: "input_audio"
  input_audio: {
    data: string
    format: "wav" | "mp3"
  }
}

export type ChatCompletionContentPart =
  | ChatCompletionContentPartText
  | ChatCompletionContentPartImage
  | ChatCompletionContentPartInputAudio

export type ChatCompletionToolCall = {
  id: string
  type: "function"
  function: {
    name: string
    arguments: string
  }
}

export type ChatCompletionToolCallParam = {
  id: string
  type: "function"
  function: {
    name: string
    arguments: string | Record<string, unknown>
  }
}

export type ChatCompletionMessageParam = {
  role: ChatCompletionRole
  content?: string | ChatCompletionContentPart[] | null
  name?: string
  tool_call_id?: string
  tool_calls?: ChatCompletionToolCallParam[]
}

export type ChatCompletionTool = {
  type: "function"
  function: {
    name: string
    description?: string
    parameters?: Record<string, unknown>
  }
}

export type ChatCompletionResponseFormat =
  | { type: "text" }
  | { type: "json_object" }
  | { type: "json_schema"; json_schema: { name?: string; schema: Record<string, unknown>; strict?: boolean } }

export type ChatCompletionFinishReason = "stop" | "length" | "tool_calls" | "content_filter" | null

export type LlamaCrabSpecificOptions = {
  template?: string
  top_k?: number
  grammar?: string
  grammar_root?: string
  json_schema?: Record<string, unknown>
  pooling?: string
  normalize?: boolean
  mmproj_path?: string
}

export type ChatCompletionCreateParams = {
  model: string
  messages: ChatCompletionMessageParam[]
  stream?: boolean
  max_tokens?: number
  min_tokens?: number
  temperature?: number
  top_p?: number
  stop?: string | string[]
  seed?: number
  n?: number
  tools?: ChatCompletionTool[]
  tool_choice?: "none" | "auto" | "required" | { type: "function"; function: { name: string } }
  response_format?: ChatCompletionResponseFormat
  logprobs?: boolean
  top_logprobs?: number
  frequency_penalty?: number
  presence_penalty?: number
  user?: string
  llama_crab?: LlamaCrabSpecificOptions
}

export type ChatCompletionChoice = {
  index: number
  message: {
    role: "assistant"
    content: string | null
    tool_calls?: ChatCompletionToolCall[]
  }
  finish_reason: ChatCompletionFinishReason
  logprobs?: CompletionLogprobs | null
}

export type Usage = {
  prompt_tokens: number
  completion_tokens?: number
  total_tokens: number
}

export type ChatCompletion = {
  id: string
  object: "chat.completion"
  created: number
  model: string
  choices: ChatCompletionChoice[]
  usage: Usage | null
}

export type ChatCompletionChunkChoice = {
  index: number
  delta: {
    role?: "assistant"
    content?: string | null
    tool_calls?: ChatCompletionToolCallChunk[]
  }
  finish_reason: ChatCompletionFinishReason
  logprobs?: CompletionLogprobs | null
}

export type ChatCompletionToolCallChunk = {
  index: number
  id?: string
  type?: "function"
  function?: {
    name?: string
    arguments?: string
  }
}

export type ChatCompletionChunk = {
  id: string
  object: "chat.completion.chunk"
  created: number
  model: string
  choices: ChatCompletionChunkChoice[]
  usage: Usage | null
  requestId?: string
}

export type CompletionCreateParams = {
  model: string
  prompt: string | string[]
  stream?: boolean
  max_tokens?: number
  min_tokens?: number
  temperature?: number
  top_p?: number
  stop?: string | string[]
  seed?: number
  n?: number
  echo?: boolean
  suffix?: string
  logprobs?: number
  frequency_penalty?: number
  presence_penalty?: number
  user?: string
  llama_crab?: LlamaCrabSpecificOptions
}

export type CompletionLogprobs = {
  tokens: string[]
  text_offset: number[]
  token_logprobs: Array<number | null>
  top_logprobs: Array<Array<{ token: string; logprob: number }> | null>
}

export type CompletionChoice = {
  text: string
  index: number
  finish_reason: ChatCompletionFinishReason
  logprobs?: CompletionLogprobs | null
}

export type Completion = {
  id: string
  object: "text_completion"
  created: number
  model: string
  choices: CompletionChoice[]
  usage: Usage
}

export type CompletionChunk = {
  id: string
  object: "text_completion.chunk"
  created: number
  model: string
  choices: CompletionChoice[]
  usage: Usage | null
  requestId?: string
}

export type EmbeddingCreateParams = {
  model: string
  input: string | string[]
  encoding_format?: "float" | "base64"
  user?: string
  llama_crab?: Pick<LlamaCrabSpecificOptions, "normalize">
}

export type Embedding = {
  object: "embedding"
  embedding: number[] | string
  index: number
}

export type EmbeddingCreateResponse = {
  object: "list"
  data: Embedding[]
  model: string
  usage: {
    prompt_tokens: number
    total_tokens: number
  }
}

export type RerankCreateParams = {
  model: string
  query: string
  documents: string[]
  top_n?: number
}

export type RerankResponse = {
  model: string
  results: Array<{
    index: number
    document: string
    relevance_score: number
  }>
}

export type TokenizeParams = {
  model: string
  input: string
}

export type TokenizeResponse = {
  tokens: number[]
}

export type TokenizeCountResponse = {
  count: number
}

export type DetokenizeParams = {
  model: string
  tokens: number[]
}

export type DetokenizeResponse = {
  text: string
}

export type ModelLoadParams = {
  model: string
  path: string
  kind?: "chat" | "completion" | "embedding" | "rerank" | "multimodal"
  mobile_preset?: "low-ram" | "balanced" | "gpu-max"
  pooling?: "unspecified" | "none" | "mean" | "cls" | "last" | "rank"
  embeddings?: boolean
  mmproj_path?: string
  n_ctx?: number
  n_batch?: number
  n_ubatch?: number
  n_gpu_layers?: number
  n_threads?: number
  n_threads_batch?: number
  use_mmap?: boolean
  flash_attn?: boolean
  offload_kqv?: boolean
}

export type ModelObject = {
  id: string
  object: "model"
  created: number
  owned_by: "llama-crab"
  path?: string
  kind?: string
  mobile_preset?: string
  pooling?: string
  mmproj_path?: string
}

export type ModelListResponse = {
  object: "list"
  data: ModelObject[]
}

export type InternalChatRequest = {
  model: string
  messages: ChatCompletionMessageParam[]
  user?: string
  maxTokens?: number
  minTokens?: number
  template?: string
  stop?: string[]
  tools?: ChatCompletionTool[]
  toolChoice?: ChatCompletionCreateParams["tool_choice"]
  responseFormat?: ChatCompletionCreateParams["response_format"]
  logprobs?: boolean
  topLogprobs?: number
  n?: number
  temperature?: number
  topP?: number
  topK?: number
  frequencyPenalty?: number
  presencePenalty?: number
  seed?: number
  grammar?: string
  grammarRoot?: string
  jsonSchema?: Record<string, unknown>
}

export type InternalCompletionRequest = {
  model: string
  prompt: string | string[]
  user?: string
  maxTokens?: number
  minTokens?: number
  stop?: string[]
  echo?: boolean
  suffix?: string
  logprobs?: number
  n?: number
  temperature?: number
  topP?: number
  topK?: number
  frequencyPenalty?: number
  presencePenalty?: number
  seed?: number
  grammar?: string
  grammarRoot?: string
  jsonSchema?: Record<string, unknown>
}

export type InternalEmbeddingRequest = {
  model: string
  input: string | string[]
  encodingFormat?: "float" | "base64"
  normalize?: boolean
  user?: string
}

export type InternalRerankRequest = {
  model: string
  query: string
  documents: string[]
  topN?: number
}
