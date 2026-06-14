import {
  toInternalChatCompletionRequest,
  toInternalCompletionRequest,
  toInternalDetokenizeRequest,
  toInternalEmbeddingRequest,
  toInternalRerankRequest,
  toInternalTokenizeRequest,
  type ChatCompletion,
  type ChatCompletionChunk,
  type ChatCompletionCreateParams,
  type Completion,
  type CompletionChunk,
  type CompletionCreateParams,
  type DetokenizeParams,
  type DetokenizeResponse,
  type EmbeddingCreateParams,
  type EmbeddingCreateResponse,
  type ModelListResponse,
  type ModelLoadParams,
  type ModelObject,
  type RerankCreateParams,
  type RerankResponse,
  type TokenizeCountResponse,
  type TokenizeParams,
  type TokenizeResponse,
} from "@llama-crab/core"
import { LlamaCrabTauriIpc, type CallOptions } from "./ipc"

type LlamaCrabTauriOptions = {
  ipc?: LlamaCrabTauriIpc
}

type NonStreamingChatCompletionParams = ChatCompletionCreateParams & {
  stream?: false
}

type StreamingChatCompletionParams = ChatCompletionCreateParams & {
  stream: true
}

type NonStreamingCompletionParams = CompletionCreateParams & {
  stream?: false
}

type StreamingCompletionParams = CompletionCreateParams & {
  stream: true
}

type ChatCompletionsResource = {
  create(params: StreamingChatCompletionParams, options?: CallOptions): Promise<AsyncIterable<ChatCompletionChunk>>
  create(params: NonStreamingChatCompletionParams, options?: CallOptions): Promise<ChatCompletion>
}

type CompletionsResource = {
  create(params: StreamingCompletionParams, options?: CallOptions): Promise<AsyncIterable<CompletionChunk>>
  create(params: NonStreamingCompletionParams, options?: CallOptions): Promise<Completion>
}

type ModelsResource = {
  load(params: ModelLoadParams): Promise<ModelObject>
  unload(id: string): Promise<void>
  list(): Promise<ModelListResponse>
  retrieve(id: string): Promise<ModelObject>
}

type ExtrasResource = {
  tokenize: ((params: TokenizeParams) => Promise<TokenizeResponse>) & {
    count(params: TokenizeParams): Promise<TokenizeCountResponse>
  }
  detokenize(params: DetokenizeParams): Promise<DetokenizeResponse>
}

export class LlamaCrabTauri {
  readonly models: ModelsResource
  readonly chat: {
    completions: ChatCompletionsResource
  }
  readonly completions: CompletionsResource
  readonly embeddings: {
    create(params: EmbeddingCreateParams): Promise<EmbeddingCreateResponse>
  }
  readonly rerank: {
    create(params: RerankCreateParams): Promise<RerankResponse>
  }
  readonly extras: ExtrasResource

  private readonly ipc: LlamaCrabTauriIpc

  constructor(options: LlamaCrabTauriOptions = {}) {
    this.ipc = options.ipc ?? new LlamaCrabTauriIpc()
    this.models = {
      load: this.#load.bind(this),
      unload: this.#unload.bind(this),
      list: this.#list.bind(this),
      retrieve: this.#retrieve.bind(this),
    }
    this.chat = {
      completions: {
        create: this.#createChatCompletion.bind(this) as ChatCompletionsResource["create"],
      },
    }
    this.completions = {
      create: this.#createCompletion.bind(this) as CompletionsResource["create"],
    }
    this.embeddings = {
      create: this.#createEmbedding.bind(this),
    }
    this.rerank = {
      create: this.#createRerank.bind(this),
    }
    const tokenize = this.#tokenize.bind(this) as ExtrasResource["tokenize"]
    tokenize.count = this.#tokenizeCount.bind(this)
    this.extras = {
      tokenize,
      detokenize: this.#detokenize.bind(this),
    }
  }

  #load(params: ModelLoadParams): Promise<ModelObject> {
    return this.ipc.load(params)
  }

  #unload(id: string): Promise<void> {
    return this.ipc.unload(id)
  }

  #list(): Promise<ModelListResponse> {
    return this.ipc.list()
  }

  #retrieve(id: string): Promise<ModelObject> {
    return this.ipc.retrieve(id)
  }

  #createChatCompletion(
    params: StreamingChatCompletionParams,
    options?: CallOptions,
  ): Promise<AsyncIterable<ChatCompletionChunk>>
  #createChatCompletion(params: NonStreamingChatCompletionParams, options?: CallOptions): Promise<ChatCompletion>
  async #createChatCompletion(
    params: ChatCompletionCreateParams,
    options: CallOptions = {},
  ): Promise<ChatCompletion | AsyncIterable<ChatCompletionChunk>> {
    const request = toInternalChatCompletionRequest(params)
    if (params.stream) {
      return this.ipc.streamChatCompletion(request, options)
    }
    return this.ipc.createChatCompletion<ChatCompletion>(request)
  }

  #createCompletion(params: StreamingCompletionParams, options?: CallOptions): Promise<AsyncIterable<CompletionChunk>>
  #createCompletion(params: NonStreamingCompletionParams, options?: CallOptions): Promise<Completion>
  async #createCompletion(
    params: CompletionCreateParams,
    options: CallOptions = {},
  ): Promise<Completion | AsyncIterable<CompletionChunk>> {
    const request = toInternalCompletionRequest(params)
    if (params.stream) {
      return this.ipc.streamCompletion(request, options)
    }
    return this.ipc.createCompletion<Completion>(request)
  }

  #createEmbedding(params: EmbeddingCreateParams): Promise<EmbeddingCreateResponse> {
    return this.ipc.createEmbedding(toInternalEmbeddingRequest(params))
  }

  #createRerank(params: RerankCreateParams): Promise<RerankResponse> {
    return this.ipc.createRerank(toInternalRerankRequest(params))
  }

  #tokenize(params: TokenizeParams): Promise<TokenizeResponse> {
    return this.ipc.tokenize(toInternalTokenizeRequest(params))
  }

  #tokenizeCount(params: TokenizeParams): Promise<TokenizeCountResponse> {
    return this.ipc.tokenizeCount(toInternalTokenizeRequest(params))
  }

  #detokenize(params: DetokenizeParams): Promise<DetokenizeResponse> {
    return this.ipc.detokenize(toInternalDetokenizeRequest(params))
  }
}
