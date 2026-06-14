export class LlamaCrabError extends Error {
  constructor(
    message: string,
    readonly code: string,
  ) {
    super(message)
    this.name = "LlamaCrabError"
  }
}

export class InvalidRequestError extends LlamaCrabError {
  constructor(message: string) {
    super(message, "invalid_request")
    this.name = "InvalidRequestError"
  }
}

export class UnsupportedFeatureError extends LlamaCrabError {
  constructor(feature: string) {
    super(`${feature} is not supported by @llama-crab/tauri`, "unsupported_feature")
    this.name = "UnsupportedFeatureError"
  }
}
