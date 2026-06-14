import { Channel, invoke } from "@tauri-apps/api/core"
import type {
  ChatCompletionChunk,
  CompletionChunk,
  DetokenizeParams,
  EmbeddingCreateResponse,
  InternalChatRequest,
  InternalCompletionRequest,
  InternalEmbeddingRequest,
  InternalRerankRequest,
  ModelListResponse,
  ModelLoadParams,
  ModelObject,
  RerankResponse,
  TokenizeCountResponse,
  TokenizeParams,
  TokenizeResponse,
} from "@llama-crab/core"

const COMMAND_PREFIX = "plugin:llama-crab"

export type CallOptions = {
  signal?: AbortSignal
}

export type IpcLoadedModelInfo = ModelObject

export class LlamaCrabTauriIpc {
  load(params: ModelLoadParams): Promise<ModelObject> {
    return invoke(`${COMMAND_PREFIX}|load_model`, {
      payload: dropUndefined({
        id: params.model,
        path: params.path,
        kind: params.kind,
        mobilePreset: params.mobile_preset,
        pooling: params.pooling,
        embeddings: params.embeddings,
        mmprojPath: params.mmproj_path,
        nCtx: params.n_ctx,
        nBatch: params.n_batch,
        nUbatch: params.n_ubatch,
        nGpuLayers: params.n_gpu_layers,
        nThreads: params.n_threads,
        nThreadsBatch: params.n_threads_batch,
        useMmap: params.use_mmap,
        flashAttn: params.flash_attn,
        offloadKqv: params.offload_kqv,
      }),
    })
  }

  unload(id: string): Promise<void> {
    return invoke(`${COMMAND_PREFIX}|unload_model`, { id })
  }

  list(): Promise<ModelListResponse> {
    return invoke(`${COMMAND_PREFIX}|list_models`)
  }

  retrieve(id: string): Promise<IpcLoadedModelInfo> {
    return invoke(`${COMMAND_PREFIX}|retrieve_model`, { id })
  }

  createChatCompletion<T>(request: InternalChatRequest): Promise<T> {
    return invoke(`${COMMAND_PREFIX}|create_chat_completion`, { payload: request })
  }

  streamChatCompletion(request: InternalChatRequest, options: CallOptions = {}): AsyncIterable<ChatCompletionChunk> {
    return streamCommand<ChatCompletionChunk>("stream_chat_completion", request, options)
  }

  createCompletion<T>(request: InternalCompletionRequest): Promise<T> {
    return invoke(`${COMMAND_PREFIX}|create_completion`, { payload: request })
  }

  streamCompletion(request: InternalCompletionRequest, options: CallOptions = {}): AsyncIterable<CompletionChunk> {
    return streamCommand<CompletionChunk>("stream_completion", request, options)
  }

  createEmbedding(request: InternalEmbeddingRequest): Promise<EmbeddingCreateResponse> {
    return invoke(`${COMMAND_PREFIX}|create_embedding`, { payload: request })
  }

  createRerank(request: InternalRerankRequest): Promise<RerankResponse> {
    return invoke(`${COMMAND_PREFIX}|create_rerank`, { payload: request })
  }

  tokenize(request: TokenizeParams): Promise<TokenizeResponse> {
    return invoke(`${COMMAND_PREFIX}|tokenize`, { payload: request })
  }

  tokenizeCount(request: TokenizeParams): Promise<TokenizeCountResponse> {
    return invoke(`${COMMAND_PREFIX}|tokenize_count`, { payload: request })
  }

  detokenize(request: DetokenizeParams): Promise<{ text: string }> {
    return invoke(`${COMMAND_PREFIX}|detokenize`, { payload: request })
  }

  cancel(requestId: string): Promise<void> {
    return invoke(`${COMMAND_PREFIX}|cancel`, { requestId })
  }
}

function streamCommand<T extends { requestId?: string }>(
  command: "stream_chat_completion" | "stream_completion",
  payload: InternalChatRequest | InternalCompletionRequest,
  options: CallOptions,
): AsyncIterable<T> {
  return {
    async *[Symbol.asyncIterator](): AsyncIterator<T> {
      const queue: T[] = []
      let done = false
      let error: unknown
      let requestId: string | undefined
      let notify: (() => void) | undefined

      const wake = () => {
        notify?.()
        notify = undefined
      }

      const onChunk = new Channel<T>()
      onChunk.onmessage = (chunk) => {
        requestId ??= chunk.requestId
        queue.push(chunk)
        wake()
      }

      const abort = () => {
        if (requestId) {
          void invoke(`${COMMAND_PREFIX}|cancel`, { requestId })
        }
      }
      options.signal?.addEventListener("abort", abort, { once: true })

      invoke(`${COMMAND_PREFIX}|${command}`, { payload, onChunk })
        .then(() => {
          done = true
          wake()
        })
        .catch((err: unknown) => {
          error = err
          done = true
          wake()
        })

      try {
        while (!done || queue.length > 0) {
          const chunk = queue.shift()
          if (chunk) {
            requestId ??= chunk.requestId
            yield chunk
            continue
          }

          await new Promise<void>((resolve) => {
            notify = resolve
          })
        }
      } finally {
        options.signal?.removeEventListener("abort", abort)
      }

      if (error) {
        throw error
      }
    },
  }
}

function dropUndefined<T extends Record<string, unknown>>(value: T): T {
  return Object.fromEntries(Object.entries(value).filter((entry) => entry[1] !== undefined)) as T
}
