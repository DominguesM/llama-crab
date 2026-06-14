import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

# TypeScript

llama-crab publishes TypeScript packages for applications that need shared
request/response contracts or a Tauri frontend client.

| Package | Purpose |
| --- | --- |
| `@llama-crab/core` | Shared OpenAI-like types, errors, and pure mapping helpers. |
| `@llama-crab/tauri` | Tauri IPC client built on top of `@llama-crab/core`. |

Use `@llama-crab/core` when you are building an adapter and need stable request and response contracts. Use `@llama-crab/tauri` when your frontend runs inside a Tauri app with `tauri-plugin-llama-crab` installed.

## Install

Install the package that matches your integration. Use `@llama-crab/core` for
plain TypeScript adapters and `@llama-crab/tauri` for Tauri frontends.

<Tabs groupId="package-manager" queryString>
  <TabItem value="npm" label="npm">

```bash title="Install with npm"
npm install @llama-crab/core
npm install @llama-crab/tauri
```

  </TabItem>
  <TabItem value="yarn" label="Yarn">

```bash title="Install with Yarn"
yarn add @llama-crab/core
yarn add @llama-crab/tauri
```

  </TabItem>
  <TabItem value="pnpm" label="pnpm" default>

```bash title="Install with pnpm"
pnpm add @llama-crab/core
pnpm add @llama-crab/tauri
```

  </TabItem>
  <TabItem value="bun" label="Bun">

```bash title="Install with Bun"
bun add @llama-crab/core
bun add @llama-crab/tauri
```

  </TabItem>
</Tabs>

`@llama-crab/tauri` depends on `@llama-crab/core`, so Tauri apps usually install
only `@llama-crab/tauri` directly.

## Import paths

```ts title="TypeScript imports"
import type { ChatCompletionCreateParams } from "@llama-crab/core"
import { toInternalChatRequest } from "@llama-crab/core"
import { LlamaCrabTauri } from "@llama-crab/tauri"
```

Both packages publish ESM, CommonJS, and TypeScript declaration outputs from `dist`.

Package build and test commands for contributors are covered in
[Development](../contributing/development.md).
