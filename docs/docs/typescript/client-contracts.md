# Client Contracts

The TypeScript packages separate public OpenAI-like contracts from the smaller internal contract used by adapters.

## Chat request mapping

`toInternalChatRequest` validates and maps:

| Public field | Internal field |
| --- | --- |
| `model` | `model` |
| `messages` | `messages` |
| `max_tokens` | `maxTokens` |
| `temperature` | `temperature` |
| `top_p` | `topP` |
| `seed` | `seed` |
| `stop` | `stopSequences` |
| `llama_crab.template` | `template` |
| `llama_crab.top_k` | `topK` |

`stop` may be a string or an array. A string becomes a one-item array.

The public `developer` role maps to internal `system`. Other supported internal roles are `user`, `assistant`, and `tool`.

## Content handling

String content is passed through unchanged. Text content parts are concatenated in order:

```ts
{
  role: "user",
  content: [
    { type: "text", text: "Hello " },
    { type: "text", text: "there" },
  ],
}
```

Image and audio content parts are currently rejected by the TypeScript mapping layer for the Tauri client.

## Unsupported feature errors

The shared helper throws `UnsupportedFeatureError` for features that the current Tauri adapter does not implement:

| Request feature | Condition |
| --- | --- |
| Multiple choices | `n` is present and not `1`. |
| Tools | `tools` is present and non-empty. |
| Tool choice | `tool_choice` is present. |
| Structured response | `response_format` is present. |
| Logprobs | `logprobs` is present. |
| Image input | Any `image_url` content part. |
| Audio input | Any `input_audio` content part. |

Missing `model` or an empty `messages` array throws `InvalidRequestError`.

## Response mapping

`toChatCompletion` builds:

```ts
{
  object: "chat.completion",
  choices: [
    {
      index: 0,
      message: { role: "assistant", content: text },
      finish_reason: "stop"
    }
  ],
  usage: null
}
```

`toChatCompletionChunk` builds `chat.completion.chunk` frames from internal token events. A done event yields an empty `delta` and a normalized `finish_reason`; token events yield `delta.content`.

Finish reasons are normalized as follows:

| Internal reason | Public reason |
| --- | --- |
| `length` | `length` |
| `stop` | `stop` |
| `eos` | `stop` |
| `tool_calls` | `tool_calls` |
| `content_filter` | `content_filter` |
| Other or missing | `null` |
