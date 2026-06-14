import { InvalidRequestError } from "./errors"
import type { DetokenizeParams, TokenizeParams } from "./types"

export function toInternalTokenizeRequest(params: TokenizeParams): TokenizeParams {
  if (!params.model) {
    throw new InvalidRequestError("model is required")
  }
  return params
}

export function toInternalDetokenizeRequest(params: DetokenizeParams): DetokenizeParams {
  if (!params.model) {
    throw new InvalidRequestError("model is required")
  }
  return params
}
