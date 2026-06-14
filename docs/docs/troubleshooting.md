# Troubleshooting

This page covers common issues for the server, Tauri plugin, and TypeScript packages.

## Server does not start

Check that the model path exists and that the selected backend can load it:

```bash
llama-crab-server --model /absolute/path/to/model.gguf
```

If startup exits after logging a model-load failure, reduce memory pressure with a smaller model, lower `--n-ctx`, lower `--n-gpu-layers`, or try a mobile preset such as `--mobile-preset low-ram`.

## `/v1/rerank` returns an error

The reranking endpoint is disabled unless the process starts with `--reranking`.

```bash
llama-crab-server \
  --model /models/reranker.gguf \
  --reranking \
  --pooling rank
```

## `/v1/embeddings` fails or returns unexpected vectors

Start the server with `--embeddings` for embedding workloads:

```bash
llama-crab-server \
  --model /models/embed.gguf \
  --embeddings \
  --pooling mean
```

Use `encoding_format: "float"` for numeric arrays or `encoding_format: "base64"` for base64-encoded little-endian `f32` bytes. Any other value is rejected.

## Streaming returns `[DONE]` after an error event

This is expected for the current SSE implementation. Streaming route errors are emitted as:

```text
event: error
data: <message>
```

The stream then emits the final:

```text
data: [DONE]
```

Treat the error event as request failure even though `[DONE]` follows it.

## Streaming completions reject multiple prompts

`/v1/completions` supports multiple prompts only in non-streaming mode. Streaming completions require exactly one prompt.

## Chat role is rejected

The server accepts `system`, `user`, `assistant`, `tool`, and `function` roles. The TypeScript Tauri client also accepts `developer` and maps it to `system`, but the Rust server path does not accept `developer` directly.

## Multimodal chat content is rejected

Multimodal server requests require both:

- A server binary built with the `mtmd` feature.
- Startup with `--mmproj /path/to/mmproj.gguf`.

Only local image paths or `file://` image URLs are accepted by the current multimodal server path. Data URLs and remote URLs are rejected.

## Tauri IPC says a model is not loaded

Load the model before generation, and pass the same model id to chat or generation calls:

```ts
await client.models.load({
  model: "local",
  path: "/models/model.gguf",
})

await client.chat.completions.create({
  model: "local",
  messages: [{ role: "user", content: "Hello" }],
})
```

The Rust plugin reports missing models with `kind: "modelNotFound"`.

## Tauri TypeScript client rejects a request before IPC

The current `@llama-crab/tauri` client rejects unsupported OpenAI features such as multiple choices, tools, `tool_choice`, `response_format`, `logprobs`, image content, and audio content. Remove those fields or use the HTTP server path when you need a feature that exists there.

## Docusaurus build reports missing docs

`docs/sidebars.ts` references several documentation sections outside the server, Tauri, TypeScript, and troubleshooting pages. If those pages have not been created yet, Docusaurus can fail with missing document ids. Add the remaining sidebar pages or adjust the sidebar once ownership allows it.
