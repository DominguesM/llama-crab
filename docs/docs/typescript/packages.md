# Packages

## `@llama-crab/core`

`@llama-crab/core` contains type definitions and pure conversion helpers. It does not talk to HTTP, Tauri, or Rust directly.

Main exports include:

| Export | Purpose |
| --- | --- |
| `ChatCompletionCreateParams` | OpenAI-like chat completion request type. |
| `ChatCompletion` | Non-streaming chat response type. |
| `ChatCompletionChunk` | Streaming chat chunk type. |
| `ModelLoadParams` | Model load request used by adapters. |
| `ModelObject`, `ModelListResponse` | Model-listing contracts. |
| `toInternalChatRequest` | Converts public chat params to the internal adapter request. |
| `toChatCompletion` | Builds a non-streaming chat completion response. |
| `toChatCompletionChunk` | Converts an internal token event to a streaming chunk. |
| `InvalidRequestError` | Thrown for invalid public requests. |
| `UnsupportedFeatureError` | Thrown when an adapter does not support a requested feature. |

Example:

```ts title="Convert a public chat request"
import { toInternalChatRequest } from "@llama-crab/core"

const request = toInternalChatRequest({
  model: "local",
  messages: [{ role: "user", content: "Hello" }],
  max_tokens: 64,
  llama_crab: {
    template: "chatml",
    top_k: 40,
  },
})
```

## `@llama-crab/tauri`

`@llama-crab/tauri` depends on `@llama-crab/core` and `@tauri-apps/api`. It exposes `LlamaCrabTauri`, which has two resource groups:

| Resource | Methods |
| --- | --- |
| `client.models` | `load`, `unload`, `list`, `retrieve` |
| `client.chat.completions` | `create` |

Example:

```ts title="Load a model and create a chat completion"
import { LlamaCrabTauri } from "@llama-crab/tauri"

const client = new LlamaCrabTauri()

await client.models.load({
  model: "local",
  path: "/models/model.gguf",
})

const response = await client.chat.completions.create({
  model: "local",
  messages: [{ role: "user", content: "Say hello." }],
})
```

For streaming, `create` returns an `AsyncIterable<ChatCompletionChunk>` when `stream: true`.
