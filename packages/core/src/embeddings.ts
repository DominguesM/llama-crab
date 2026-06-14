import { InvalidRequestError } from "./errors"
import type { EmbeddingCreateParams, InternalEmbeddingRequest } from "./types"

export function toInternalEmbeddingRequest(params: EmbeddingCreateParams): InternalEmbeddingRequest {
  if (!params.model) {
    throw new InvalidRequestError("model is required")
  }
  if (Array.isArray(params.input) && params.input.length === 0) {
    throw new InvalidRequestError("input must not be empty")
  }

  return dropUndefined({
    model: params.model,
    input: params.input,
    encodingFormat: params.encoding_format,
    normalize: params.llama_crab?.normalize,
    user: params.user,
  })
}

function dropUndefined<T extends Record<string, unknown>>(value: T): T {
  return Object.fromEntries(Object.entries(value).filter((entry) => entry[1] !== undefined)) as T
}
