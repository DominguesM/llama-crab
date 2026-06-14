# Contribuindo

Obrigado por considerar uma contribuição ao `llama-crab`! Esta
página percorre as maneiras mais comuns de se envolver.

## Código de Conduta

O projeto segue o
[Contributor Covenant](https://www.contributor-covenant.org/). Ao
participar, você concorda em defender seus termos. O texto completo
está em [`CODE_OF_CONDUCT.md`](https://github.com/DominguesM/llama-crab/blob/main/CODE_OF_CONDUCT.md).

## Reportando um bug

Pesquise nas [GitHub issues] primeiro para evitar duplicatas.
Quando você abrir uma nova issue, inclua:

- Um título curto e descritivo.
- O comando exato que você rodou e a saída exata que obteve.
- O identificador do modelo (caminho do Hugging Face) e o tamanho
  do GGUF.
- A plataforma (`aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`,
  `x86_64-pc-windows-msvc`, …) e a versão do Rust.
- As sondas de capacidade `llama_crab::LlamaBackend`, se
  relevante.

## Propondo uma feature

Abra uma issue com a label `enhancement`. Descreva:

- O caso de uso que a feature desbloqueia.
- A forma da API que você esperaria.
- Se você estaria disposto a enviar um PR.

Para features maiores, a aba [Discussions] é um lugar melhor para
coletar feedback antes de abrir uma issue.

## Enviando um pull request

### 1. Fork e clone

```bash
git clone --recursive https://github.com/<você>/llama-crab.git
cd llama-crab
git checkout -b minha-feature
```

O `--recursive` é importante: o submódulo
`llama-crab-sys/llama.cpp` é parte do build.

### 2. Faça suas mudanças

O repositório segue o estilo Rust padrão. O `Makefile` expõe as
verificações comuns:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy -p llama-crab --all-features --all-targets
cargo doc -p llama-crab --no-deps --all-features
```

Se você adicionar uma API pública, escreva um comentário de
rustdoc que inclua um exemplo executável. A CI falha o build se o
rustdoc tem links quebrados ou warnings.

### 3. Adicione um teste

A API segura tem testes unitários no mesmo módulo do código, e
testes de integração em `llama-crab/tests/`. Os testes de
integração pulam de forma limpa quando o modelo não está no disco.

### 4. Atualize a documentação

Se você adicionar uma feature pública, atualize o guia do usuário
em `docs/`. O fonte da documentação é Markdown; o build é
mkdocs-material.

### 5. Abra o PR

Faça push do branch para seu fork e abra um PR contra `main`. A CI
roda:

- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy -p llama-crab --all-features --all-targets`
- `cargo doc -p llama-crab --no-deps --all-features`
- A build de matriz nos backends suportados.

Um mantenedor vai revisar e ou fazer merge ou pedir mudanças.

## Adicionando uma feature do Cargo

Adicionar uma nova feature do Cargo é uma mudança de API pública. A
convenção é:

1. Discuta a feature em uma issue primeiro.
2. Adicione a feature em `llama-crab-sys/build.rs` e no
   `Cargo.toml` de `llama-crab`.
3. Atualize a [referência de features do Cargo](../reference/cargo-features.md).
4. Adicione uma linha à matriz de CI que exercita a nova feature.
5. Documente a feature no guia relevante.

## Adicionando um novo exemplo

Veja o [índice de exemplos](../examples/index.md) para o
boilerplate e as regras.

## Adicionando um template de chat embutido

1. Adicione a nova variante `BuiltinTemplate` em
   `llama_crab::chat::template`.
2. Adicione a lógica de renderização em `render_builtin`.
3. Adicione um teste unitário que renderiza uma mensagem
   conhecida e afirma a saída esperada.
4. Atualize a [referência de templates de chat embutidos](../reference/chat-templates.md).

## Processo de release

Releases são cortados pelos mantenedores. O fluxo:

1. Suba a versão em `Cargo.toml`.
2. Atualize o `CHANGELOG.md`.
3. Marque o commit com tag.
4. O workflow `release.yml` compila cada alvo suportado,
   publica no crates.io e cria um GitHub release.

## Por onde ir a partir daqui

- [GitHub issues] — reporte um bug ou uma requisição de feature.
- [Discussions] — questões e ideias de design.
- [Código de conduta] — as regras da estrada.

[GitHub issues]: https://github.com/DominguesM/llama-crab/issues
[Discussions]: https://github.com/DominguesM/llama-crab/discussions
[Código de conduta]: https://github.com/DominguesM/llama-crab/blob/main/CODE_OF_CONDUCT.md
