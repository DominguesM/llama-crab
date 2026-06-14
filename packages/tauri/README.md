# `@llama-crab/tauri`

OpenAI-like TypeScript client for the `tauri-plugin-llama-crab` IPC runtime.

```ts
import { LlamaCrabTauri } from "@llama-crab/tauri"

const client = new LlamaCrabTauri()
```

## Models

```ts
await client.models.load({
  model: "local",
  path: "/models/model.gguf",
  kind: "chat",
  mobile_preset: "balanced",
})

const models = await client.models.list()
const model = await client.models.retrieve("local")
await client.models.unload(model.id)
```

Use explicit runtime options for embedding, rerank, and multimodal models:

```ts
await client.models.load({
  model: "embedder",
  path: "/models/embed.gguf",
  kind: "embedding",
  embeddings: true,
  pooling: "mean",
})
```

## Chat Completions

```ts
const completion = await client.chat.completions.create({
  model: "local",
  messages: [
    { role: "developer", content: "Answer briefly." },
    { role: "user", content: "Explain Tauri in one sentence." },
  ],
  tools: [{ type: "function", function: { name: "lookup" } }],
  response_format: { type: "json_object" },
  max_tokens: 128,
})

console.log(completion.choices[0]?.message.content)
```

Streaming returns OpenAI-like chunks:

```ts
const abort = new AbortController()
const stream = await client.chat.completions.create(
  {
    model: "local",
    messages: [{ role: "user", content: "Count to three." }],
    stream: true,
  },
  { signal: abort.signal },
)

for await (const chunk of stream) {
  process.stdout.write(chunk.choices[0]?.delta.content ?? "")
}
```

## Text Completions

```ts
await client.completions.create({
  model: "local",
  prompt: "Rust is",
  max_tokens: 32,
  stop: ["\n"],
})
```

## Embeddings

```ts
await client.embeddings.create({
  model: "embedder",
  input: ["texto A", "texto B"],
  encoding_format: "float",
})
```

## Rerank

```ts
await client.rerank.create({
  model: "reranker",
  query: "pergunta",
  documents: ["doc 1", "doc 2"],
  top_n: 2,
})
```

## Extras

```ts
const { tokens } = await client.extras.tokenize({ model: "local", input: "hello" })
const { count } = await client.extras.tokenize.count({ model: "local", input: "hello" })
const { text } = await client.extras.detokenize({ model: "local", tokens })
```

## llama-crab Options

llama-crab-specific options stay under `llama_crab`:

```ts
await client.chat.completions.create({
  model: "local",
  messages: [{ role: "user", content: "Hello" }],
  llama_crab: {
    template: "chatml",
    top_k: 40,
    grammar: "root ::= object",
    grammar_root: "root",
  },
})
```

## Support Matrix

| Area | Status |
| --- | --- |
| Models | OpenAI-like list/retrieve/load/unload |
| Chat completions | OpenAI-like non-streaming and streaming, tools and structured output fields |
| Text completions | OpenAI legacy completions shape |
| Embeddings | OpenAI-like float/base64 responses |
| Rerank | llama-crab server-compatible namespace |
| Tokenize/detokenize | llama-crab extras namespace |
| Multimodal | Accepts OpenAI-like image parts; Rust runtime currently rejects media unless an mtmd path is integrated |
