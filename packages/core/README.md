# `@llama-crab/core`

<p align="center">
  <img
    src="https://gist.githubusercontent.com/DominguesM/127b9e5614e0e2da6b896fb3da3c8f2d/raw/d5dec07e795979f0a1b43d246a730f4031452113/canarim-crab.png"
    alt="llama-crab logo"
    width="180"
  />
</p>

OpenAI-like TypeScript contracts and pure mapping helpers shared by llama-crab adapters.

This package does not create a runtime client. Use it for types and request normalization when building a backend adapter.

Documentation:

- [User guide](https://dominguesm.github.io/llama-crab-docs/)
- [TypeScript packages](https://dominguesm.github.io/llama-crab-docs/tauri/)
- [Repository](https://github.com/DominguesM/llama-crab)

```ts
import type { ChatCompletionCreateParams } from "@llama-crab/core"
import { toInternalChatCompletionRequest } from "@llama-crab/core"

const params: ChatCompletionCreateParams = {
  model: "local",
  messages: [{ role: "user", content: "Hello" }],
  tools: [{ type: "function", function: { name: "lookup" } }],
  response_format: { type: "json_object" },
  llama_crab: { template: "chatml", top_k: 40 },
}

const request = toInternalChatCompletionRequest(params)
```

## Helpers

- `toInternalChatCompletionRequest`
- `toInternalCompletionRequest`
- `toInternalEmbeddingRequest`
- `toInternalRerankRequest`
- `toInternalTokenizeRequest`
- `toInternalDetokenizeRequest`
- `toChatCompletion`
- `toChatCompletionChunk`

## Support Matrix

| Area | Contract |
| --- | --- |
| Models | OpenAI-like `object: "list"` and `object: "model"` |
| Chat completions | OpenAI-like messages, tools, response format, usage, chunks |
| Text completions | OpenAI legacy completions shape |
| Embeddings | OpenAI-like `input`, `model`, `encoding_format`, `object: "list"` |
| Rerank | Explicit llama-crab namespace compatible with llama-crab server `/v1/rerank` |
| Tokenize/detokenize | Explicit llama-crab extras namespace |
| llama-crab options | Isolated under `llama_crab` before mapping |
