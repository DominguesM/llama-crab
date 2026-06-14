import { describe, expect, it } from "vitest"
import {
  toChatCompletionChunk,
  toInternalChatCompletionRequest,
  toInternalCompletionRequest,
  toInternalEmbeddingRequest,
  toInternalRerankRequest,
} from "../src/index"

describe("OpenAI-like core contracts", () => {
  it("maps chat params with tools, response_format, logprobs, sampling, and image parts", () => {
    const request = toInternalChatCompletionRequest({
      model: "local",
      messages: [
        { role: "developer", content: "Answer in JSON." },
        {
          role: "user",
          content: [
            { type: "text", text: "Describe this: " },
            { type: "image_url", image_url: { url: "file:///tmp/image.png" } },
          ],
        },
      ],
      tools: [
        {
          type: "function",
          function: {
            name: "lookup",
            description: "Lookup data",
            parameters: { type: "object" },
          },
        },
      ],
      tool_choice: "auto",
      response_format: { type: "json_object" },
      logprobs: true,
      top_logprobs: 2,
      n: 2,
      frequency_penalty: 0.1,
      presence_penalty: 0.2,
      max_tokens: 64,
      stop: "END",
      llama_crab: {
        template: "chatml",
        top_k: 20,
        grammar: "root ::= object",
        grammar_root: "root",
      },
    })

    expect(request).toMatchObject({
      model: "local",
      messages: [
        { role: "system", content: "Answer in JSON." },
        {
          role: "user",
          content: [
            { type: "text", text: "Describe this: " },
            { type: "image_url", image_url: { url: "file:///tmp/image.png" } },
          ],
        },
      ],
      template: "chatml",
      maxTokens: 64,
      topK: 20,
      stop: ["END"],
      logprobs: true,
      topLogprobs: 2,
      n: 2,
      responseFormat: { type: "json_object" },
      grammar: "root ::= object",
      grammarRoot: "root",
    })
  })

  it("maps text completion params", () => {
    expect(
      toInternalCompletionRequest({
        model: "local",
        prompt: ["A", "B"],
        max_tokens: 8,
        stop: ["END"],
        echo: true,
        suffix: "!",
        logprobs: 3,
        n: 2,
        llama_crab: { top_k: 10 },
      }),
    ).toEqual({
      model: "local",
      prompt: ["A", "B"],
      maxTokens: 8,
      stop: ["END"],
      echo: true,
      suffix: "!",
      logprobs: 3,
      n: 2,
      topK: 10,
    })
  })

  it("maps embedding, rerank, and streaming chunk contracts", () => {
    expect(
      toInternalEmbeddingRequest({
        model: "embedding-model",
        input: ["one", "two"],
        encoding_format: "base64",
        llama_crab: { normalize: false },
      }),
    ).toEqual({
      model: "embedding-model",
      input: ["one", "two"],
      encodingFormat: "base64",
      normalize: false,
    })

    expect(
      toInternalRerankRequest({
        model: "reranker",
        query: "q",
        documents: ["b", "a"],
        top_n: 1,
      }),
    ).toEqual({
      model: "reranker",
      query: "q",
      documents: ["b", "a"],
      topN: 1,
    })

    expect(
      toChatCompletionChunk({
        id: "chatcmpl-1",
        model: "local",
        created: 123,
        choices: [
          {
            index: 0,
            delta: { tool_calls: [{ index: 0, id: "call_1", type: "function", function: { name: "lookup" } }] },
            finish_reason: null,
          },
        ],
      }),
    ).toEqual({
      id: "chatcmpl-1",
      object: "chat.completion.chunk",
      created: 123,
      model: "local",
      choices: [
        {
          index: 0,
          delta: {
            tool_calls: [{ index: 0, id: "call_1", type: "function", function: { name: "lookup" } }],
          },
          finish_reason: null,
        },
      ],
      usage: null,
    })
  })
})
