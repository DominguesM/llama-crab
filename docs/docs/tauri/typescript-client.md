import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

# TypeScript Client for Tauri

`@llama-crab/tauri` wraps the Tauri IPC commands with an OpenAI-like client shape.

## Install

<Tabs groupId="package-manager" queryString>
  <TabItem value="npm" label="npm">

```bash title="Install with npm"
npm install @llama-crab/tauri
```

  </TabItem>
  <TabItem value="yarn" label="Yarn">

```bash title="Install with Yarn"
yarn add @llama-crab/tauri
```

  </TabItem>
  <TabItem value="pnpm" label="pnpm" default>

```bash title="Install with pnpm"
pnpm add @llama-crab/tauri
```

  </TabItem>
  <TabItem value="bun" label="Bun">

```bash title="Install with Bun"
bun add @llama-crab/tauri
```

  </TabItem>
</Tabs>

## Create a client

```ts title="Create a Tauri client"
import { LlamaCrabTauri } from "@llama-crab/tauri"

const client = new LlamaCrabTauri()

await client.models.load({
  model: "local",
  path: "/models/model.gguf",
  mobile_preset: "balanced",
})
```

## Model management

```ts title="Load, inspect, and unload a model"
const model = await client.models.load({
  model: "local",
  path: "/models/model.gguf",
  n_ctx: 4096,
  n_gpu_layers: 32,
})

const list = await client.models.list()
const sameModel = await client.models.retrieve(model.id)

await client.models.unload(model.id)
```

The TypeScript client maps snake_case public fields to the plugin's camelCase IPC payload. For example, `n_ctx` becomes `nCtx`, and `mobile_preset` becomes `mobilePreset`.

## Chat completions

```ts title="Create a chat completion"
const completion = await client.chat.completions.create({
  model: "local",
  messages: [
    { role: "developer", content: "Answer in one sentence." },
    { role: "user", content: "What is local inference?" },
  ],
  max_tokens: 128,
  temperature: 0.7,
})

console.log(completion.choices[0]?.message.content)
```

`developer` messages are converted to `system` before they reach the internal request.

## Streaming

```ts title="Stream chat tokens"
const stream = await client.chat.completions.create({
  model: "local",
  messages: [{ role: "user", content: "Count to three." }],
  stream: true,
})

for await (const chunk of stream) {
  const token = chunk.choices[0]?.delta.content
  if (token) {
    console.log(token)
  }
}
```

The client collects `TokenEvent` values from the Tauri channel and converts them into OpenAI-like `chat.completion.chunk` objects.

## Supported parameters

The Tauri TypeScript client currently accepts the shared chat parameter type from `@llama-crab/core`, but rejects several OpenAI features before IPC:

| Feature | Behavior |
| --- | --- |
| `n` other than `1` | Throws `UnsupportedFeatureError`. |
| `tools` with entries | Throws `UnsupportedFeatureError`. |
| `tool_choice` | Throws `UnsupportedFeatureError`. |
| `response_format` | Throws `UnsupportedFeatureError`. |
| `logprobs` | Throws `UnsupportedFeatureError`. |
| Image content parts | Throws `UnsupportedFeatureError`. |
| Audio content parts | Throws `UnsupportedFeatureError`. |

llama-crab-specific options live under `llama_crab`:

```ts title="Pass llama-crab-specific options"
await client.chat.completions.create({
  model: "local",
  messages: [{ role: "user", content: "Hello" }],
  llama_crab: {
    template: "chatml",
    top_k: 40,
  },
})
```
