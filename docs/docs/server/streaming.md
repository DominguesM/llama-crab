# Streaming

Streaming is enabled by setting `stream: true` on `/v1/completions` or `/v1/chat/completions`. The server responds with server-sent events and always ends the stream with:

```text
data: [DONE]
```

If generation fails after the SSE response has started, the stream emits an error event:

```text
event: error
data: <message>
```

## Chat streaming

```bash
curl -N http://127.0.0.1:8080/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "local-chat",
    "stream": true,
    "messages": [{ "role": "user", "content": "Count to three." }],
    "max_tokens": 32
  }'
```

Chat streams use `chat.completion.chunk` frames. The first valid frame announces the assistant role:

```json
{
  "object": "chat.completion.chunk",
  "choices": [
    {
      "index": 0,
      "delta": { "role": "assistant" },
      "finish_reason": null
    }
  ]
}
```

Token frames place generated text in `choices[0].delta.content`. The final chunk uses an empty delta and a `finish_reason` such as `stop`, `length`, or `tool_calls`.

Tool-call deltas are emitted in `choices[0].delta.tool_calls` when the selected template and generated text produce parseable function-call output.

## Completion streaming

```bash
curl -N http://127.0.0.1:8080/v1/completions \
  -H 'content-type: application/json' \
  -d '{
    "model": "local",
    "stream": true,
    "prompt": "One short sentence:",
    "max_tokens": 32
  }'
```

Completion streams use `text_completion` frames and place generated text in `choices[0].text`.

Streaming completions require exactly one prompt. If `prompt` is an array with more than one item, the server emits a streaming error.

## Client handling

A minimal browser or Node-compatible SSE reader should:

1. Read each `data:` frame.
2. Stop when the data is `[DONE]`.
3. Treat `event: error` as a failed request.
4. For chat, append `choices[0].delta.content` when present.
5. For text completions, append `choices[0].text` when present.

The server sends default SSE keep-alives through Axum's `KeepAlive::default()`.
