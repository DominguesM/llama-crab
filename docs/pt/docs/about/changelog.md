# Changelog

O histórico completo de releases vive no arquivo
[`CHANGELOG.md`](https://github.com/DominguesM/llama-crab/blob/main/CHANGELOG.md)
na raiz do repositório. Esta página é um resumo dos releases
mais recentes, com as mudanças breaking destacadas.

## Releases recentes

### `0.1.300` (mais recente)

- Adicionado o binário HTTP `llama-crab-server` com endpoints de
  completions, chat, embeddings, reranking, tokenização e streaming SSE.
- Adicionadas APIs de streaming de completions em alto nível e o
  exemplo executável `streaming`.
- Adicionados presets mobile por meio de `MobilePreset` e
  `LlamaParams::with_mobile_preset`.
- Adicionado o exemplo wrapper `server_lfm` para iniciar o servidor
  com modelos de texto LFM.
- Migrado o guia de usuário de mdBook para Material for MkDocs com
  árvores de documentação em inglês e português.

### `0.1.201`

- Preparada a linha de release pós-`0.1.2`.
- Alinhadas as matrizes de features do CI com as features das crates.
- Escopo de coverage ajustado ao layout das crates publicadas.
- Instaladas dependências de shader Vulkan no CI.

### `0.1.2`

- Expandida a cobertura do guia mdBook e dos exemplos executáveis.
- Adicionados workflows de exemplo em um comando e helpers de download
  de modelos.
- Corrigidos comportamentos de completion, embeddings, gramáticas,
  multimodal e runner de exemplos contra a API atual do `llama.cpp`.

### `0.1.0`

- Release público inicial da série `0.1.x`.
- A API segura de alto nível sobre `llama-crab-sys`.
- 9 crates de exemplo e 3 testes de integração.

## Receitas de migração

Quando uma mudança breaking aterrissa em `0.1.x`, a receita para
migrar seu código é documentada na página [MSRV & versionamento](../reference/msrv.md).

## Por onde ir a partir daqui

- [MSRV & versionamento](../reference/msrv.md) — o guia de
  migração completo.
- [GitHub releases](https://github.com/DominguesM/llama-crab/releases) —
  os artefatos e notas por release.
- [Contribuindo](contributing.md) — como enviar uma correção para
  um bug que você encontrou em um release.
