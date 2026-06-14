# MSRV & versionamento

Esta página documenta a versão mínima suportada de Rust (MSRV), a
política de `SemVer` e o pin do commit do llama.cpp.

## MSRV

**`1.88.0`** — fixado via [`rust-toolchain.toml`](https://github.com/DominguesM/llama-crab/blob/main/rust-toolchain.toml).

Subir o MSRV é uma **mudança breaking** e vai disparar um bump de
versão major.

O MSRV é exercitado pela matriz de CI em cada push para `main`. Uma
falha de build na versão mais baixa suportada é tratada como um
bug e corrigida antes da mudança ser mergeada.

## Política de `SemVer`

`llama-crab` segue [Semantic Versioning 2.0.0](https://semver.org/).
Para uma API pública em um release `0.x.y`, as regras são:

- **Patch (`0.0.y` → `0.0.y+1`)** — bug fixes retrocompatíveis.
  Refatorações internas, documentação, melhorias de performance.
- **Minor (`0.x.y` → `0.x+1.0`)** — nova superfície de API
  retrocompatível. Novos módulos, novos métodos, novas features do
  Cargo. Código existente continua funcionando.
- **Major (`0.x.y` → `1.0.0`)** — mudanças incompatíveis.

O crate está atualmente na série `0.1.x`, o que significa que a
API é *esperada* evoluir. Mudanças breaking dentro de `0.1.x`
são documentadas no [CHANGELOG](https://github.com/DominguesM/llama-crab/blob/main/CHANGELOG.md)
e no guia de migração abaixo.

## Pin do llama.cpp

`llama-crab` fixa o `llama.cpp` em um commit específico através de
um submódulo e uma feature do Cargo. O commit exato é visível em:

- O badge do README (`llama.cpp: <commit>`).
- O ponteiro do submódulo em
  [`crates/llama-crab-sys/llama.cpp`](https://github.com/DominguesM/llama-crab/tree/main/crates/llama-crab-sys/llama.cpp).
- O `Cargo.lock` (procure pela dependência `llama-cpp-sys-2`).

Dois builds da mesma versão de `llama-crab` sempre produzem a
mesma biblioteca nativa, então o binário é reproduzível.

### Subindo o pin

Subir o commit do `llama.cpp` é tratado como um bump de versão
**minor** em `0.1.x`. A matriz de CI re-roda cada backend e os
testes de integração; o bump só é mergeado quando tudo está verde.

## Cargo.lock

O `Cargo.lock` é versionado. Para bibliotecas, isso é incomum;
para `llama-crab` é intencional, porque o build linka contra uma
biblioteca nativa fixada e queremos que consumidores downstream
vejam exatamente os mesmos artefatos.

## Cadência de release

Releases são cortadas do `main` sempre que uma mudança
significativa aterrissou. Os critérios são:

- Uma nova API pública ou um refinamento significativo de uma
  existente.
- Uma nova feature do Cargo, um backend, ou uma família de
  modelos.
- Um conjunto significativo de bug fixes.

O processo de release é automatizado através do workflow do
GitHub Actions [`release.yml`](https://github.com/DominguesM/llama-crab/blob/main/.github/workflows/release.yml),
que compila cada alvo suportado, publica no crates.io e cria um
GitHub release.

## Guia de migração

Esta seção é atualizada sempre que uma mudança breaking aterrissa
em `0.1.x`. O histórico completo vive no
[CHANGELOG](https://github.com/DominguesM/llama-crab/blob/main/CHANGELOG.md).

### De `0.1.0` para `0.1.100`

- `LlamaParams::new` agora requer um `&str` ou `String` (era
  `Into<PathBuf>`). Converta com `.to_string()` ou `.into()`.
- `Llama::create_completion` não aceita mais um argumento
  `temperature: f32`. Passe através de
  `CompletionOptions::with_temperature`.
- O campo `n_threads_batch` em `LlamaParams` foi renomeado para
  `with_n_threads_decode`.

### De `0.1.100` para `0.1.200`

- `chat::render_builtin` retorna `Result<String, _>` em vez de
  `String`. Erro é `ChatRenderError::UnknownArchitecture`.
- A feature `mtmd` agora requer um ambiente de build com a edição
  Rust 2024.

## Por onde ir a partir daqui

- [CHANGELOG](https://github.com/DominguesM/llama-crab/blob/main/CHANGELOG.md) —
  o histórico completo de releases.
- [Layout dos crates](crate-layout.md) — a árvore de código-fonte.
- [Features do Cargo](cargo-features.md) — a referência em forma
  longa.
