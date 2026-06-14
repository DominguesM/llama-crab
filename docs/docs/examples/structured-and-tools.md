---
title: Structured Output and Tools
---

# Structured output and tools

These examples show two related patterns: constraining the model to JSON and
turning model output into a Rust-side function call.

## Structured JSON

```bash
./run.sh structured
```

The example:

1. Defines a JSON Schema with `name` and `age`.
2. Converts it with `json_schema_grammar`.
3. Creates a grammar sampler with `LlamaSampler::grammar`.
4. Chains the grammar sampler with greedy decoding.
5. Parses the final text with `serde_json`.

This is the right pattern when the response itself must conform to a schema.

## Tool call demo

```bash
./run.sh tools
./run.sh tool_calls_qwen
```

The current tool example keeps the runtime simple:

1. The system message tells the model to return one JSON call.
2. Rust extracts and validates the JSON.
3. Rust executes a local `get_weather` fixture.
4. The example prints both the tool call and tool result.

Use this pattern when the model should select or populate an action but your
application owns execution, authorization, and error handling.

## Server support

The HTTP server accepts structured response hints and tool definitions on
`/v1/chat/completions`. For production APIs, validate model output server-side
even when grammar or tool hints are present.
