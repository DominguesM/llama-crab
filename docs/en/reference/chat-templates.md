# Built-in chat templates

`llama-crab` ships with **14 built-in chat templates** that cover
the most popular open-weights models. Each template is exposed as a
variant of the [`BuiltinTemplate`] enum and renders to a stable
prompt string through the [`render_builtin`] function.

## The full list

| Template | Architecture | Notes |
| --- | --- | --- |
| `Plain` | _any_ | A simple `### ` separator. Fallback for unknown architectures. |
| `ChatMl` | Qwen 2 / 2.5, Hermes, Yi | `<|im_start|>` / `<|im_end|>` markers. |
| `Llama2` | Llama 2 chat | `[INST] ... [/INST]` markers. |
| `Llama3` | Llama 3 / 3.1 / 3.2 / 3.3 | `<|start_header_id|>` / `<|end_header_id|>` markers, supports tools. |
| `Mistral` | Mistral / Mixtral instruct | `[INST] ... [/INST]` markers, with `[TOOL_CALLS]` for tools. |
| `Qwen2` | Qwen 2 | `<|im_start|>` / `<|im_end|>` markers. |
| `Qwen2_5` | Qwen 2.5 | `<|im_start|>` / `<|im_end|>` markers, supports tools. |
| `Phi3` | Phi-3 / Phi-3.5 | `<|user|>` / `<|assistant|>` / `<|end|>` markers. |
| `Gemma` | Gemma 2 / 3 | `<start_of_turn>` / `<end_of_turn>` markers. |
| `CommandR` | Cohere Command R / R+ | `<|START_OF_TURN_TOKEN|>` / `<|END_OF_TURN_TOKEN|>` markers. |
| `DeepSeek2` | DeepSeek-V2 / V2.5 | Custom markers, supports tools. |
| `CodeFim` | Code models | FIM (fill-in-the-middle) prompt format. |
| `FunctionaryV2` | Functionary v2 | Multi-turn tool protocol. |
| `OpenChat` | OpenChat | `<|start_of_role|>` / `<|end_of_role|>` markers. |

If your model is not on the list, the Jinja2 subset renderer in
`llama_crab::chat::jinja` can parse a template string directly. Open
an issue if you need a specific template that is not yet built-in.

## Auto-detection

Most modern GGUF files declare their chat template in the metadata.
Use [`detect_chat_format`] to read it and pick a matching
`BuiltinTemplate`:

```rust
use llama_crab::chat::detect_chat_format;

let metadata = llama.model().metadata();
let template = detect_chat_format(&metadata);
```

`detect_chat_format` looks at the `general.architecture` key in the
metadata and returns the closest `BuiltinTemplate` variant. If the
architecture is not recognised, it returns `BuiltinTemplate::Plain`.

## Rendering manually

```rust
use llama_crab::chat::{BuiltinTemplate, render_builtin, ChatMessage, Role};

let prompt = render_builtin(
    BuiltinTemplate::Llama3,
    &[ChatMessage::new(Role::User, "Hi")],
    &[],      // no tools
    true,     // add the assistant turn-prefix
);
```

The last argument controls whether to append the assistant "turn
prefix" (e.g. `<|start|>assistant\n` for Llama 3). Set it to `true`
when the model is supposed to continue, `false` when you're
inspecting the rendered prompt.

## Snippets

=== "Plain"

    ```
    System: You are a helpful assistant.

    User: Hi
    ### Assistant:

    ```

=== "ChatMl"

    ```
    <|im_start|>system
    You are a helpful assistant.<|im_end|>
    <|im_start|>user
    Hi<|im_end|>
    <|im_start|>assistant
    ```

=== "Llama3"

    ```
    <|begin_of_text|><|start_header_id|>system<|end_header_id|>

    You are a helpful assistant.<|eot_id|><|start_header_id|>user<|end_header_id|>

    Hi<|eot_id|><|start_header_id|>assistant<|end_header_id|>


    ```

=== "Mistral"

    ```
    [INST] You are a helpful assistant.

    Hi [/INST]
    ```

=== "Gemma"

    ```
    <start_of_turn>user
    Hi<end_of_turn>
    <start_of_turn>model
    ```

## Tool rendering

Templates that support tool calling render a JSON Schema for the
tool parameters in a model-specific way. The shape of the rendered
prompt depends on the template; the model has been trained to
recognise it.

| Template | Tool syntax |
| --- | --- |
| `ChatMl` | `<tool_call>{"name": ..., "arguments": ...}</tool_call>` |
| `Llama3` | `<\|python_tag\|>{"name": ..., "arguments": ...}` |
| `Mistral` | `[TOOL_CALLS][{"name": ..., "arguments": ...}]` |
| `FunctionaryV2` | `<\|start\|>function<\|message\|>...<\|call\|>` |
| `Plain` | `{...}` (any JSON object) |

See the [chat & tool calling guide](../features/chat.md) for the
parser matrix.

## Adding a new template

If you need a template that is not built-in, you can render it
manually or extend the Jinja2 subset renderer. Open an issue with
the template and a sample model, and we'll add it to the enum.

## Where to next?

- [Chat & tool calling guide](../features/chat.md) — the parser
  matrix and the multi-turn tool protocol.
- [Built-in template reference](https://docs.rs/llama-crab/latest/llama_crab/chat/enum.BuiltinTemplate.html) —
  the auto-generated rustdoc.
- [Tool calling example](../examples/tools.md) — a runnable
  program.

[`BuiltinTemplate`]: https://docs.rs/llama-crab/latest/llama_crab/chat/enum.BuiltinTemplate.html
[`render_builtin`]: https://docs.rs/llama-crab/latest/llama_crab/chat/fn.render_builtin.html
[`detect_chat_format`]: https://docs.rs/llama-crab/latest/llama_crab/chat/fn.detect_chat_format.html
