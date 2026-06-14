# Agradecimentos

`llama-crab` não existiria sem o trabalho das pessoas e projetos
listados abaixo. Obrigado.

## A fundação

- **[llama.cpp](https://github.com/ggml-org/llama.cpp)** —
  Georgi Gerganov e a equipe `ggml-org`. O engine de inferência
  C/C++ que `llama-crab` encapsula. `llama.cpp` é a razão pela
  qual podemos rodar grandes modelos de linguagem em hardware
  de consumidor.
- **[GGML](https://github.com/ggml-org/ggml)** — a biblioteca de
  tensores que alimenta cada backend.

## O ecossistema Rust

`llama-crab` se apoia nos ombros de uma longa lista de projetos
Rust. Destaques:

- **`bindgen`** — geração automática dos bindings FFI em
  `llama-crab-sys`.
- **`cmake`** e **`cc`** — a cola de build C/C++.
- **`serde` e `serde_json`** — tipos de requisição/resposta,
  definições de tools, o conversor JSON-Schema.
- **`anyhow` e `thiserror`** — tratamento de erros.
- **`tokio` e `axum`** — o servidor HTTP.
- **`tracing` e `tracing-subscriber`** — logging estruturado.
- **`sled`** — o cache de prompt em disco.

Uma lista completa vive no `Cargo.lock` do workspace.

## Os modelos

Os exemplos neste repositório são testados contra modelos
open-weights do Hugging Face Hub. Obrigado a:

- **Alibaba (equipe Qwen)** — Qwen 2 / 2.5.
- **Meta (equipe Llama)** — Llama 3 / 3.1 / 3.2 / 3.3.
- **Google (equipe Gemma)** — Gemma 2 / 3 / 4.
- **Mistral AI** — Mistral e Mixtral.
- **Microsoft (equipe Phi)** — Phi-3.
- **DeepSeek AI** — DeepSeek-V2 / V2.5.
- **Liquid AI** — LFM2.5-VL.
- **Beijing Academy of Artificial Intelligence (equipe BGE)** —
  embeddings e rerankers BGE.
- **Cohere** — Command R / R+.

## As ferramentas

- **[Material for MkDocs](https://squidfunk.github.io/mkdocs-material/)** —
  o tema do site de documentação.
- **[Pymdown Extensions](https://facelessuser.github.io/pymdown-extensions/)** —
  as extensões Markdown usadas pelos docs.
- **[mdBook](https://rust-lang.github.io/mdBook/)** — a ferramenta
  de documentação anterior. Obrigado pelos anos de serviço.

## A comunidade

Obrigado a cada contribuidor que reportou uma issue, enviou um PR
ou ajudou alguém nas discussions. A lista completa vive no
[grafo de contribuidores](https://github.com/DominguesM/llama-crab/graphs/contributors).

## Por onde ir a partir daqui

- [Licença](license.md) — o texto completo.
- [Contribuindo](contributing.md) — como enviar uma correção
  para um bug que você encontrou.
- [Início](../index.md) — de volta à home da documentação.
