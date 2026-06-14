---
hide:
  - navigation
  - toc
---

<div align="center" markdown>

# :material-crab: **llama-crab**

**Bindings Rust seguros, ergonômicos e completos para [`llama.cpp`](https://github.com/ggml-org/llama.cpp).**

[![Crates.io](https://img.shields.io/crates/v/llama-crab.svg)](https://crates.io/crates/llama-crab)
[![docs.rs](https://docs.rs/llama-crab/badge.svg)](https://docs.rs/llama-crab)
[![MSRV: 1.88](https://img.shields.io/badge/MSRV-1.88-blue.svg)](https://github.com/DominguesM/llama-crab/blob/main/rust-toolchain.toml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/DominguesM/llama-crab/blob/main/LICENSE-MIT)
[![llama.cpp pinned](https://img.shields.io/badge/llama.cpp-pinned-5c5c5c?logo=github)](https://github.com/ggml-org/llama.cpp)

</div>

---

## O que é o llama-crab?

`llama-crab` é um crate Rust (na verdade um workspace de dois crates) que
oferece uma **API 100% segura em Rust** sobre o [`llama.cpp`](https://github.com/ggml-org/llama.cpp).
Você pode carregar qualquer modelo GGUF, executar completions de texto
e chat, calcular embeddings, restringir a geração com uma gramática GBNF,
acionar modelos visão-linguagem através do `mtmd`, ou expor tudo via HTTP
— tudo isso sem tocar em um único bloco `unsafe` no nível da aplicação.

<div class="grid cards" markdown>

-   :material-rocket-launch: __Comece em 5 minutos__

    Carregue um modelo e gere uma completion com poucas linhas.

    [:octicons-arrow-right-24: Instalação](getting-started/installation.md)
    [:octicons-arrow-right-24: Seu primeiro programa](getting-started/first-program.md)

-   :material-cog-outline: __Execute em qualquer hardware__

    CPU, Metal, CUDA, Vulkan, ROCm, OpenCL e KleidiAI — escolha seu
    backend em tempo de compilação e descarregue quantas camadas couberem
    na VRAM.

    [:octicons-arrow-right-24: Backends & offload de GPU](guides/backends.md)

-   :material-cellphone: __Distribua em celulares e tablets__

    Perfis `release-size` e `release-perf`, OpenCL + KleidiAI para
    Android, Metal para iOS, e `MobilePreset` para padrões sensatos.

    [:octicons-arrow-right-24: Distribuição mobile](guides/mobile.md)

-   :material-eye-outline: __Visão e áudio__

    Combine um GGUF de texto com um projetor `mmproj` e alimente
    imagens ou áudio no mesmo contexto.

    [:octicons-arrow-right-24: Multimodal](features/multimodal.md)

-   :material-graph-outline: __Embeddings e reranking__

    Extraia vetores com pooling configurável, faça busca semântica
    ou use um cross-encoder para ranqueamento de alta qualidade.

    [:octicons-arrow-right-24: Embeddings](features/embeddings.md)

-   :material-server: __Servidor HTTP pronto__

    `llama-crab-server` expõe a API de alto nível através de uma
    interface HTTP compatível com OpenAI, com streaming SSE.

    [:octicons-arrow-right-24: Servidor](server/index.md)

</div>

## Um gostinho da API

=== "Texto simples"

    ```rust
    use llama_crab::{Llama, LlamaParams};

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let mut llama = Llama::load(
            LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
                .with_n_ctx(2048)
                .with_n_gpu_layers(99),
        )?;

        let response = llama.create_completion("The capital of France is", 32)?;
        println!("{}", response.text);
        Ok(())
    }
    ```

=== "Chat"

    ```rust
    use llama_crab::chat::BuiltinTemplate;
    use llama_crab::high_level::chat_completion::{create_chat_completion_with, ChatMessage};
    use llama_crab::{Llama, LlamaParams, Role};

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let mut llama = Llama::load(
            LlamaParams::new("models/instruct.gguf").with_n_ctx(4096),
        )?;

        let messages = vec![
            ChatMessage::new(Role::System, "You are a concise assistant."),
            ChatMessage::new(Role::User, "Explain Rust ownership in one paragraph."),
        ];

        let response = create_chat_completion_with(
            &mut llama,
            &messages,
            BuiltinTemplate::ChatMl,
            &[],
            128,
        )?;

        println!("{}", response.content);
        Ok(())
    }
    ```

=== "Embeddings"

    ```rust
    use llama_crab::{Llama, LlamaParams};

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let mut llama = Llama::load(
            LlamaParams::new("models/bge-small-en-v1.5-q4_k_m.gguf")
                .with_n_ctx(512)
                .with_embeddings(true),
        )?;

        let embedding = llama.embed("Rust is memory-safe.", true)?;
        println!("dim = {}", embedding.len());
        Ok(())
    }
    ```

## Por que llama-crab?

<div class="grid" markdown>

:material-shield-check: __Seguro por padrão__

A API de alto nível não expõe nenhuma superfície `unsafe` — toda
fronteira FFI vive atrás de wrappers tipados em `llama-crab-sys`.
Você só opta por acesso bruto quando realmente precisa.

:material-puzzle-outline: __Superfície de recursos completa__

Cada estratégia de amostragem, formato de chat, pipeline de visão,
conversor JSON-Schema, decodificador especulativo por prompt-lookup
e cache KV persistente que o `llama.cpp` oferece está acessível
a partir de Rust seguro.

:material-package-variant: __Builds reproduzíveis__

`llama.cpp` está fixado em um commit específico, o build é hermético
e a CI roda em uma matriz de combinações CPU / CUDA / Vulkan / Metal
/ ROCm para manter cada backend funcional.

:material-flash: __Performance em primeiro lugar__

Offload de camadas, flash attention, presets mobile, cadeias de
amostragem, decodificação especulativa e parsers de tool-call são
todos projetados para mantê-lo no caminho rápido sem escrever
kernels customizados.

</div>

## Crates neste workspace

| Crate | Propósito | Quando usar |
| --- | --- | --- |
| [`llama-crab`](https://crates.io/crates/llama-crab) | API 100% segura em Rust: carregamento de modelo, amostragem, chat, embeddings, cola do servidor. | **Maioria das aplicações.** Este é o crate do qual você depende. |
| [`llama-crab-sys`](https://crates.io/crates/llama-crab-sys) | FFI bruta gerada via `bindgen` sobre `wrapper.h` + CMake. | Quando você precisa de acesso direto a símbolos do llama.cpp que o crate seguro ainda não encapsula. |
| [`llama-crab-server`](https://github.com/DominguesM/llama-crab/tree/main/llama-crab-server) | Binário HTTP construído sobre `llama-crab`. | Quando você quer um endpoint compatível com OpenAI sem escrever um. |

## Licença

`llama-crab` é distribuído sob a **Licença MIT**. Veja
[`LICENSE-MIT`](https://github.com/DominguesM/llama-crab/blob/main/LICENSE-MIT)
para o texto completo.

---

!!! tip "Por onde começar?"

    - [Instale o crate](getting-started/installation.md) e verifique
      sua toolchain.
    - Leia a [visão geral da arquitetura](core-concepts/architecture.md)
      para entender os principais blocos de construção.
    - Dê uma olhada no [índice de exemplos](examples/index.md) e copie
      o que mais se aproxima do que você quer construir.
