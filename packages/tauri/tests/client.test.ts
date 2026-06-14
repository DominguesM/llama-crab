import { beforeEach, describe, expect, it, vi } from "vitest"

vi.mock("@tauri-apps/api/core", () => {
  class Channel<T> {
    onmessage?: (message: T) => void
  }

  return {
    Channel,
    invoke: vi.fn(),
  }
})

import { invoke } from "@tauri-apps/api/core"
import { LlamaCrabTauri } from "../src/index"

describe("LlamaCrabTauri public API", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset()
  })

  it("loads models with expanded runtime options and uses list/retrieve/unload commands", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "embedder",
      object: "model",
      created: 0,
      owned_by: "llama-crab",
      path: "/models/e.gguf",
      kind: "embedding",
    })
    vi.mocked(invoke).mockResolvedValueOnce({
      object: "list",
      data: [{ id: "embedder", object: "model", created: 0, owned_by: "llama-crab", kind: "embedding" }],
    })
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "embedder",
      object: "model",
      created: 0,
      owned_by: "llama-crab",
      path: "/models/e.gguf",
      kind: "embedding",
    })
    vi.mocked(invoke).mockResolvedValueOnce(undefined)

    const client = new LlamaCrabTauri()

    await client.models.load({
      model: "embedder",
      path: "/models/e.gguf",
      kind: "embedding",
      pooling: "mean",
      embeddings: true,
      n_batch: 256,
      n_ubatch: 128,
      flash_attn: true,
      offload_kqv: false,
    })
    await client.models.list()
    await client.models.retrieve("embedder")
    await client.models.unload("embedder")

    expect(invoke).toHaveBeenNthCalledWith(1, "plugin:llama-crab|load_model", {
      payload: {
        id: "embedder",
        path: "/models/e.gguf",
        kind: "embedding",
        pooling: "mean",
        embeddings: true,
        nBatch: 256,
        nUbatch: 128,
        flashAttn: true,
        offloadKqv: false,
      },
    })
    expect(invoke).toHaveBeenNthCalledWith(2, "plugin:llama-crab|list_models")
    expect(invoke).toHaveBeenNthCalledWith(3, "plugin:llama-crab|retrieve_model", { id: "embedder" })
    expect(invoke).toHaveBeenNthCalledWith(4, "plugin:llama-crab|unload_model", { id: "embedder" })
  })

  it("routes chat completions, completions, embeddings, rerank, and extras", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "chatcmpl-1",
      object: "chat.completion",
      created: 1,
      model: "local",
      choices: [{ index: 0, message: { role: "assistant", content: "ok" }, finish_reason: "stop" }],
      usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 },
    })
    vi.mocked(invoke).mockResolvedValueOnce({
      id: "cmpl-1",
      object: "text_completion",
      created: 1,
      model: "local",
      choices: [{ index: 0, text: "ok", finish_reason: "stop" }],
      usage: { prompt_tokens: 1, completion_tokens: 1, total_tokens: 2 },
    })
    vi.mocked(invoke).mockResolvedValueOnce({
      object: "list",
      data: [{ object: "embedding", embedding: [0.1], index: 0 }],
      model: "embedder",
      usage: { prompt_tokens: 1, total_tokens: 1 },
    })
    vi.mocked(invoke).mockResolvedValueOnce({
      model: "reranker",
      results: [{ index: 1, document: "b", relevance_score: 0.9 }],
    })
    vi.mocked(invoke).mockResolvedValueOnce({ tokens: [1, 2] })
    vi.mocked(invoke).mockResolvedValueOnce({ count: 2 })
    vi.mocked(invoke).mockResolvedValueOnce({ text: "hi" })

    const client = new LlamaCrabTauri()

    await client.chat.completions.create({
      model: "local",
      messages: [{ role: "user", content: "hi" }],
      tools: [{ type: "function", function: { name: "lookup" } }],
      response_format: { type: "json_object" },
      logprobs: true,
      top_logprobs: 2,
    })
    await client.completions.create({ model: "local", prompt: "hi", max_tokens: 4 })
    await client.embeddings.create({ model: "embedder", input: "hi", encoding_format: "float" })
    await client.rerank.create({ model: "reranker", query: "q", documents: ["a", "b"], top_n: 1 })
    await client.extras.tokenize({ model: "local", input: "hi" })
    await client.extras.tokenize.count({ model: "local", input: "hi" })
    await client.extras.detokenize({ model: "local", tokens: [1, 2] })

    expect(invoke).toHaveBeenNthCalledWith(1, "plugin:llama-crab|create_chat_completion", {
      payload: expect.objectContaining({ model: "local", tools: expect.any(Array), responseFormat: { type: "json_object" } }),
    })
    expect(invoke).toHaveBeenNthCalledWith(2, "plugin:llama-crab|create_completion", {
      payload: expect.objectContaining({ model: "local", prompt: "hi" }),
    })
    expect(invoke).toHaveBeenNthCalledWith(3, "plugin:llama-crab|create_embedding", {
      payload: { model: "embedder", input: "hi", encodingFormat: "float" },
    })
    expect(invoke).toHaveBeenNthCalledWith(4, "plugin:llama-crab|create_rerank", {
      payload: { model: "reranker", query: "q", documents: ["a", "b"], topN: 1 },
    })
    expect(invoke).toHaveBeenNthCalledWith(5, "plugin:llama-crab|tokenize", {
      payload: { model: "local", input: "hi" },
    })
    expect(invoke).toHaveBeenNthCalledWith(6, "plugin:llama-crab|tokenize_count", {
      payload: { model: "local", input: "hi" },
    })
    expect(invoke).toHaveBeenNthCalledWith(7, "plugin:llama-crab|detokenize", {
      payload: { model: "local", tokens: [1, 2] },
    })
  })

  it("streams chunks through channels and cancels via AbortSignal", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === "plugin:llama-crab|stream_chat_completion") {
        const channel = (args as { onChunk?: { onmessage?: (message: unknown) => void } }).onChunk!
        channel.onmessage?.({
          id: "chatcmpl-1",
          object: "chat.completion.chunk",
          created: 1,
          model: "local",
          choices: [{ index: 0, delta: { role: "assistant" }, finish_reason: null }],
          usage: null,
          requestId: "req-1",
        })
        channel.onmessage?.({
          id: "chatcmpl-1",
          object: "chat.completion.chunk",
          created: 1,
          model: "local",
          choices: [{ index: 0, delta: {}, finish_reason: "stop" }],
          usage: null,
          requestId: "req-1",
        })
      }
      return undefined
    })

    const abort = new AbortController()
    const client = new LlamaCrabTauri()
    const stream = await client.chat.completions.create(
      { model: "local", messages: [{ role: "user", content: "hi" }], stream: true },
      { signal: abort.signal },
    )
    const chunks = []

    for await (const chunk of stream) {
      chunks.push(chunk)
      abort.abort()
    }

    expect(chunks).toHaveLength(2)
    expect(invoke).toHaveBeenCalledWith("plugin:llama-crab|cancel", { requestId: "req-1" })
  })
})
