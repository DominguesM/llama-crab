import { InvalidRequestError } from "./errors"
import type { CompletionCreateParams, InternalCompletionRequest } from "./types"

export function toInternalCompletionRequest(params: CompletionCreateParams): InternalCompletionRequest {
  if (!params.model) {
    throw new InvalidRequestError("model is required")
  }
  if (Array.isArray(params.prompt) && params.prompt.length === 0) {
    throw new InvalidRequestError("prompt must not be empty")
  }

  return dropUndefined({
    model: params.model,
    prompt: params.prompt,
    user: params.user,
    maxTokens: params.max_tokens,
    minTokens: params.min_tokens,
    temperature: params.temperature,
    topP: params.top_p,
    topK: params.llama_crab?.top_k,
    stop: normalizeStop(params.stop),
    seed: params.seed,
    n: params.n,
    echo: params.echo,
    suffix: params.suffix,
    logprobs: params.logprobs,
    frequencyPenalty: params.frequency_penalty,
    presencePenalty: params.presence_penalty,
    grammar: params.llama_crab?.grammar,
    grammarRoot: params.llama_crab?.grammar_root,
    jsonSchema: params.llama_crab?.json_schema,
  })
}

function normalizeStop(stop: CompletionCreateParams["stop"]): string[] | undefined {
  if (typeof stop === "string") {
    return [stop]
  }
  return stop
}

function dropUndefined<T extends Record<string, unknown>>(value: T): T {
  return Object.fromEntries(Object.entries(value).filter((entry) => entry[1] !== undefined)) as T
}
