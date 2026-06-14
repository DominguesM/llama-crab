# Templates de chat embutidos

`llama-crab` vem com **14 templates de chat embutidos** que cobrem
os modelos open-weights mais populares. Cada template é exposto
como uma variante do enum [`BuiltinTemplate`] e renderiza para uma
string de prompt estável através da função [`render_builtin`].

## A lista completa

| Template | Arquitetura | Notas |
| --- | --- | --- |
| `Plain` | _qualquer_ | Um separador `### ` simples. Fallback para arquiteturas desconhecidas. |
| `ChatMl` | Qwen 2 / 2.5, Hermes, Yi | Marcadores `<|im_start|>` / `<|im_end|>`. |
| `Llama2` | Llama 2 chat | Marcadores `[INST] ... [/INST]`. |
| `Llama3` | Llama 3 / 3.1 / 3.2 / 3.3 | Marcadores `<|start_header_id|>` / `<|end_header_id|>`, suporta tools. |
| `Mistral` | Mistral / Mixtral instruct | Marcadores `[INST] ... [/INST]`, com `[TOOL_CALLS]` para tools. |
| `Qwen2` | Qwen 2 | Marcadores `<|im_start|>` / `<|im_end|>`. |
| `Qwen2_5` | Qwen 2.5 | Marcadores `<|im_start|>` / `<|im_end|>`, suporta tools. |
| `Phi3` | Phi-3 / Phi-3.5 | Marcadores `<|user|>` / `<|assistant|>` / `<|end|>`. |
| `Gemma` | Gemma 2 / 3 | Marcadores `<start_of_turn>` / `<end_of_turn>`. |
| `CommandR` | Cohere Command R / R+ | Marcadores `<|START_OF_TURN_TOKEN|>` / `<|END_OF_TURN_TOKEN|>`. |
| `DeepSeek2` | DeepSeek-V2 / V2.5 | Marcadores customizados, suporta tools. |
| `CodeFim` | Modelos de código | Formato de prompt FIM (fill-in-the-middle). |
| `FunctionaryV2` | Functionary v2 | Protocolo de tool multi-turno. |
| `OpenChat` | OpenChat | Marcadores `<|start_of_role|>` / `<|end_of_role|>`. |

Se seu modelo não está na lista, o renderizador de subconjunto
Jinja2 em `llama_crab::chat::jinja` pode fazer parse de uma string
de template diretamente. Abra uma issue se você precisa de um
template específico que ainda não está embutido.

## Auto-detecção

A maioria dos arquivos GGUF modernos declara seu template de chat
nos metadados. Use [`detect_chat_format`] para lê-lo e escolher
um `BuiltinTemplate` que combine:

```rust
use llama_crab::chat::detect_chat_format;

let metadata = llama.model().metadata();
let template = detect_chat_format(&metadata);
```

`detect_chat_format` olha para a chave `general.architecture` nos
metadados e retorna a variante `BuiltinTemplate` mais próxima. Se
a arquitetura não é reconhecida, retorna `BuiltinTemplate::Plain`.

## Renderizando manualmente

```rust
use llama_crab::chat::{BuiltinTemplate, render_builtin, ChatMessage, Role};

let prompt = render_builtin(
    BuiltinTemplate::Llama3,
    &[ChatMessage::new(Role::User, "Oi")],
    &[],      // sem tools
    true,     // adiciona o turn-prefix do assistant
);
```

O último argumento controla se deve ou não anexar o "turn prefix"
do assistant (ex. `<|start|>assistant\n` para Llama 3). Defina
como `true` quando o modelo deve continuar, `false` quando você
está inspecionando o prompt renderizado.

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

## Renderização de tools

Templates que suportam tool calling renderizam um JSON Schema para
os parâmetros da tool de uma maneira específica do modelo. A forma
do prompt renderizado depende do template; o modelo foi treinado
para reconhecê-lo.

| Template | Sintaxe de tool |
| --- | --- |
| `ChatMl` | `<tool_call>{"name": ..., "arguments": ...}</tool_call>` |
| `Llama3` | `<\|python_tag\|>{"name": ..., "arguments": ...}` |
| `Mistral` | `[TOOL_CALLS][{"name": ..., "arguments": ...}]` |
| `FunctionaryV2` | `<\|start\|>function<\|message\|>...<\|call\|>` |
| `Plain` | `{...}` (qualquer objeto JSON) |

Veja o [guia de chat & tool calling](../features/chat.md) para a
matriz de parsers.

## Adicionando um novo template

Se você precisa de um template que não está embutido, pode
renderizá-lo manualmente ou estender o renderizador de subconjunto
Jinja2. Abra uma issue com o template e um modelo de exemplo, e
nós o adicionaremos ao enum.

## Por onde ir a partir daqui

- [Guia de chat & tool calling](../features/chat.md) — a matriz
  de parsers e o protocolo de tool multi-turno.
- [Referência de template embutido](https://docs.rs/llama-crab/latest/llama_crab/chat/enum.BuiltinTemplate.html) —
  o rustdoc auto-gerado.
- [Exemplo de tool calling](../examples/tools.md) — um programa
  executável.

[`BuiltinTemplate`]: https://docs.rs/llama-crab/latest/llama_crab/chat/enum.BuiltinTemplate.html
[`render_builtin`]: https://docs.rs/llama-crab/latest/llama_crab/chat/fn.render_builtin.html
[`detect_chat_format`]: https://docs.rs/llama-crab/latest/llama_crab/chat/fn.detect_chat_format.html
