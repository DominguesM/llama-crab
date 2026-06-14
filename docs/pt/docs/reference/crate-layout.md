# Layout dos crates

O workspace `llama-crab` contém dois crates de biblioteca e um
crate de binário. Esta página é o mapa de que você precisa para
navegar pela árvore de código-fonte.

```
llama-crab/
├── crates/
│   ├── llama-crab-sys/      # FFI bruta (bindgen + CMake)
│   ├── llama-crab/          # API 100% segura em Rust
│   │   ├── backend          # LlamaBackend + NumaStrategy
│   │   ├── model            # LlamaModel + LlamaModelParams
│   │   ├── context          # LlamaContext + params + embeddings + session
│   │   ├── batch            # LlamaBatch
│   │   ├── sampling         # LlamaSampler + SamplerChain (17 estratégias)
│   │   ├── chat             # ChatMessage + templates + tool calling
│   │   ├── speculative      # PromptLookupDecoding + speculative_decode
│   │   ├── multimodal       # MtmdContext + MtmdBitmap (feature mtmd)
│   │   ├── cache            # RamCache + DiskCache
│   │   ├── json_schema      # JSON-Schema -> GBNF
│   │   ├── high_level       # Orquestrador Llama + create_completion
│   │   ├── error            # Enum LlamaError
│   │   └── log              # Integração com tracing
│   └── llama-crab-server/   # Binário HTTP construído sobre llama-crab
├── packages/
│   ├── core/                # Reservado para @llama-crab/core
│   └── tauri/               # Reservado para @llama-crab/tauri
├── examples/                # Crates de exemplo executáveis
└── docs/                    # Guia do usuário e fonte do site
```

## `llama-crab-sys`

O pacote de FFI de baixo nível. Contém:

- O cabeçalho `wrapper.h` que seleciona a API C do llama.cpp a
  expor.
- O `build.rs` que roda `bindgen` contra o wrapper e `cmake`
  contra a árvore de código-fonte `llama.cpp/` empacotada.
- O `bindings.rs` gerado (não edite — é regenerado em cada build).
- Um punhado de wrappers seguros para as chamadas FFI mais usadas,
  para que consumidores do crate seguro possam ficar em Rust
  seguro.

A maioria das aplicações deve depender de `llama-crab` em vez
disso. Use `llama-crab-sys` apenas quando precisar de acesso
direto a um símbolo do llama.cpp que o crate seguro ainda não
encapsula.

## `llama-crab`

A API segura de alto nível. Cada módulo é documentado em
[docs.rs/llama-crab](https://docs.rs/llama-crab); esta página dá
as responsabilidades de alto nível.

| Módulo | Responsabilidade |
| --- | --- |
| `backend` | `LlamaBackend`, o handle do backend GGML, posicionamento NUMA. |
| `model` | `LlamaModel` — os pesos carregados + tokenizador + metadados. |
| `context` | `LlamaContext` — o cache KV + o driver de forward pass. |
| `batch` | `LlamaBatch` — o builder de batch tipado para `decode`. |
| `sampling` | `LlamaSampler` + `SamplerChain` — 17 estratégias de amostragem. |
| `chat` | `ChatMessage`, `Role`, `BuiltinTemplate`, `render_builtin`, o parser de tool-call. |
| `speculative` | `PromptLookupDecoding`, trait `DraftModel`, `speculative_decode`. |
| `multimodal` (feature `mtmd`) | `MtmdContext`, `MtmdBitmap`, `MtmdInputText`, `chunks.eval`. |
| `cache` | `RamCache`, `DiskCache` (feature `disk-cache`), o trait `Cache`. |
| `json_schema` | O conversor JSON-Schema → GBNF. |
| `high_level` | `Llama`, o orquestrador que possui o modelo + contexto + estado do sampler. |
| `error` | `LlamaError` — o tipo único de erro para a API segura. |
| `log` | Integração com `tracing` — logs info/warn ao redor do carregamento do modelo. |

### O orquestrador `Llama`

O tipo mais usado na API segura. Ele possui:

- Um guard `LlamaBackend` (inicializado em `Llama::load`).
- Um `LlamaModel` (os pesos + tokenizador).
- Um `LlamaContext` (o cache KV).
- Uma `SamplerChain` padrão (greedy por padrão, configurável
  através de `Llama::create_*_with_sampler`).

Os métodos de alto nível (`create_completion`,
`create_chat_completion`, `embed`, `rerank`, `complete_infill`)
escondem o loop ilustrado no
[guia de arquitetura](../core-concepts/architecture.md) atrás de
uma única chamada de função.

## `llama-crab-server`

Um binário HTTP fino construído sobre a API segura. Ele mantém a
inferência dentro do binding Rust e usa uma thread worker que
possui o modelo e o contexto. Veja o [guia do servidor](../server/index.md)
para a forma do runtime e a superfície da API.

## Exemplos

O diretório [`examples/`](https://github.com/DominguesM/llama-crab/tree/main/examples)
contém 14 crates Cargo autocontidos que exercitam cada
funcionalidade pública. Cada um é um `[[bin]]` de seu próprio
crate e pode ser copiado para outro projeto sem modificação. Veja
o [índice de exemplos](../examples/index.md) para a tabela.

## Testes de integração

O diretório [`crates/llama-crab/tests/`](https://github.com/DominguesM/llama-crab/tree/main/crates/llama-crab/tests)
contém os mesmos exemplos em forma de teste. Eles pulam de forma
limpa quando o modelo não está no disco, então um clone fresco
pode compilar o binário de teste sem possuir o modelo.

| Teste | Modelo | O que verifica |
| --- | --- | --- |
| `gemma4_text.rs` | Gemma 4 (apenas texto) | Geração de texto, sem visão. |
| `gemma4_vision.rs` | Gemma 4 + mmproj + imagem de teste | A API de alto nível `MtmdContext`. |
| `lfm_vl_vision.rs` | LFM2.5-VL + mmproj + imagem de teste | Multimodal em um modelo menor. |

## Por onde ir a partir daqui

- [Arquitetura](../core-concepts/architecture.md) — o fluxo de
  dados dentro de uma única passada forward.
- [Features do Cargo](cargo-features.md) — o que cada feature
  ativa.
- [API no docs.rs](https://docs.rs/llama-crab) — o rustdoc
  auto-gerado.
