import { InvalidRequestError } from "./errors"
import type {
  ChatCompletion,
  ChatCompletionChunk,
  ChatCompletionCreateParams,
  ChatCompletionFinishReason,
  ChatCompletionMessageParam,
  InternalChatRequest,
} from "./types"

type ChatChunkInput = Omit<ChatCompletionChunk, "object" | "usage"> & {
  object?: "chat.completion.chunk"
  usage?: ChatCompletionChunk["usage"]
}

export function toInternalChatCompletionRequest(params: ChatCompletionCreateParams): InternalChatRequest {
  if (!params.model) {
    throw new InvalidRequestError("model is required")
  }
  if (!params.messages.length) {
    throw new InvalidRequestError("messages must contain at least one message")
  }

  return dropUndefined({
    model: params.model,
    messages: params.messages.map(toInternalMessage),
    user: params.user,
    template: params.llama_crab?.template,
    maxTokens: params.max_tokens,
    minTokens: params.min_tokens,
    temperature: params.temperature,
    topP: params.top_p,
    topK: params.llama_crab?.top_k,
    stop: normalizeStop(params.stop),
    seed: params.seed,
    tools: params.tools,
    toolChoice: params.tool_choice,
    responseFormat: params.response_format,
    logprobs: params.logprobs,
    topLogprobs: params.top_logprobs,
    n: params.n,
    frequencyPenalty: params.frequency_penalty,
    presencePenalty: params.presence_penalty,
    grammar: params.llama_crab?.grammar,
    grammarRoot: params.llama_crab?.grammar_root,
    jsonSchema: params.llama_crab?.json_schema,
  })
}

export const toInternalChatRequest = toInternalChatCompletionRequest

export function toChatCompletion(value: ChatCompletion): ChatCompletion
export function toChatCompletion(
  params: ChatCompletionCreateParams,
  text: string,
  finishReason?: string | null,
  metadata?: { id?: string; created?: number },
): ChatCompletion
export function toChatCompletion(
  paramsOrValue: ChatCompletionCreateParams | ChatCompletion,
  text = "",
  finishReason?: string | null,
  metadata: { id?: string; created?: number } = {},
): ChatCompletion {
  if ("object" in paramsOrValue) {
    return paramsOrValue
  }
  return {
    id: metadata.id ?? createId("chatcmpl"),
    object: "chat.completion",
    created: metadata.created ?? currentUnixTime(),
    model: paramsOrValue.model,
    choices: [
      {
        index: 0,
        message: {
          role: "assistant",
          content: text,
        },
        finish_reason: normalizeFinishReason(finishReason),
      },
    ],
    usage: null,
  }
}

export function toChatCompletionChunk(chunk: ChatChunkInput): ChatCompletionChunk
export function toChatCompletionChunk(
  event: { requestId: string; token: string; index: number; done?: boolean; stopReason?: string | null },
  model: string,
  metadata?: { created?: number },
): ChatCompletionChunk
export function toChatCompletionChunk(
  chunkOrEvent: ChatChunkInput | { requestId: string; token: string; index: number; done?: boolean; stopReason?: string | null },
  model?: string,
  metadata: { created?: number } = {},
): ChatCompletionChunk {
  if ("choices" in chunkOrEvent) {
    return {
      ...chunkOrEvent,
      object: "chat.completion.chunk",
      usage: chunkOrEvent.usage ?? null,
    }
  }

  return {
    id: `chatcmpl-${chunkOrEvent.requestId}`,
    object: "chat.completion.chunk",
    created: metadata.created ?? currentUnixTime(),
    model: model ?? "",
    choices: [
      {
        index: 0,
        delta: chunkOrEvent.done ? {} : { content: chunkOrEvent.token },
        finish_reason: chunkOrEvent.done ? normalizeFinishReason(chunkOrEvent.stopReason) : null,
      },
    ],
    usage: null,
  }
}

function toInternalMessage(message: ChatCompletionMessageParam): ChatCompletionMessageParam {
  return {
    ...message,
    role: message.role === "developer" ? "system" : message.role,
  }
}

function normalizeStop(stop: ChatCompletionCreateParams["stop"]): string[] | undefined {
  if (typeof stop === "string") {
    return [stop]
  }
  return stop
}

function normalizeFinishReason(reason: string | null | undefined): ChatCompletionFinishReason {
  if (reason === "length") {
    return "length"
  }
  if (reason === "stop" || reason === "eos") {
    return "stop"
  }
  if (reason === "tool_calls") {
    return "tool_calls"
  }
  if (reason === "content_filter") {
    return "content_filter"
  }
  return null
}

function createId(prefix: string): string {
  return `${prefix}-${Math.random().toString(36).slice(2, 12)}`
}

function currentUnixTime(): number {
  return Math.floor(Date.now() / 1000)
}

function dropUndefined<T extends Record<string, unknown>>(value: T): T {
  return Object.fromEntries(Object.entries(value).filter((entry) => entry[1] !== undefined)) as T
}
