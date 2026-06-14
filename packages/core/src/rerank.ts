import { InvalidRequestError } from "./errors"
import type { InternalRerankRequest, RerankCreateParams } from "./types"

export function toInternalRerankRequest(params: RerankCreateParams): InternalRerankRequest {
  if (!params.model) {
    throw new InvalidRequestError("model is required")
  }
  if (!params.documents.length) {
    throw new InvalidRequestError("documents must contain at least one document")
  }

  return dropUndefined({
    model: params.model,
    query: params.query,
    documents: params.documents,
    topN: params.top_n,
  })
}

function dropUndefined<T extends Record<string, unknown>>(value: T): T {
  return Object.fromEntries(Object.entries(value).filter((entry) => entry[1] !== undefined)) as T
}
