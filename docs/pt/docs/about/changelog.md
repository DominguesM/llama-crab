# Changelog

O histórico completo de releases vive no arquivo
[`CHANGELOG.md`](https://github.com/DominguesM/llama-crab/blob/main/CHANGELOG.md)
na raiz do repositório. Esta página é um resumo dos releases
mais recentes, com as mudanças breaking destacadas.

## Releases recentes

### `0.1.300` (mais recente)

- Adicionado `llama_crab::embed::EmbedOptions` para controle
  refinado sobre a extração de embeddings.
- Melhoria nas sondas de capacidade `LlamaBackend`.
- Subiu o pin do `llama.cpp` para o último stable.
- Novo `MobilePreset::GpuMax` para GPUs mobile de alta gama.
- Corrigido um bug onde `chunks.eval` podia falhar silenciosamente
  em certas versões de projetor.

### `0.1.200`

- Adicionada a feature `llguidance` para o sampler
  [`llguidance`].
- Novos `BuiltinTemplate::DeepSeek2` e `BuiltinTemplate::CommandR`.
- Melhoria no renderizador de subconjunto Jinja2 para suportar
  `for` loops aninhados.
- O módulo `chat` agora exporta um método builder
  `ToolDefinition::with_strict`.
- Subiu o MSRV para `1.88.0`.

### `0.1.100`

- A feature `mtmd` agora suporta bitmaps de áudio além de imagens.
- Novo helper de alto nível `Llama::rerank` para cross-encoder
  rankers.
- O binário `server` agora expõe uma flag `--mobile-preset`.
- Corrigido um memory leak em `LlamaSampler::chain` quando a
  cadeia era derrubada no meio da geração.

### `0.1.0`

- Release público inicial da série `0.1.x`.
- A API segura de alto nível sobre `llama-crab-sys`.
- 14 crates de exemplo.
- O binário `llama-crab-server`.

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

[`llguidance`]: https://github.com/microsoft/llguidance
