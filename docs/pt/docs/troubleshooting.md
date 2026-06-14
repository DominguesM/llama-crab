# Solução de problemas

Respostas para os problemas que mais provavelmente vão afetar
novos usuários. Se seu problema não está listado aqui, pesquise
nas [GitHub issues] antes de abrir uma nova.

## Build & compilação

### Erros de CMake / clang ao compilar `llama-crab-sys`

`llama-crab-sys` constrói o `llama.cpp` a partir do código-fonte
via CMake. Você precisa:

- **CMake** ≥ 3.18
- **Um compilador C/C++** que suporte C11 / C++17 (clang 14+,
  GCC 11+, MSVC 2022)
- No macOS: Xcode Command Line Tools (`xcode-select --install`)
- No Linux: `build-essential` (Debian/Ubuntu) ou equivalente

Se o build morre em `llama-crab-sys`, re-execute com
`cargo build -vv` para ver o erro CMake subjacente.

### O primeiro build é lento

Compilar todos os backends do llama.cpp leva ~3 minutos em uma
máquina de 16 cores. Builds subsequentes ficam em cache. Para
acelerar o primeiro build, desabilite os backends que você não
precisa:

```toml
llama-crab = { version = "0.1", default-features = false, features = ["openmp"] }
```

### `avx2 not detected` em CPUs mais antigos

Defina `LLAMA_NO_AVX2=1` (ou qualquer uma das flags `LLAMA_NO_*`
documentadas em `llama.cpp`) antes de `cargo build`:

```bash
LLAMA_NO_AVX2=1 cargo build --release
```

### Erros de linker no macOS

```text
ld: library 'omp' not found
```

Instale o OpenMP via Homebrew:

```bash
brew install libomp
```

Depois re-execute `cargo build`. O `build.rs` do crate deve pegar
a localização do Homebrew automaticamente.

### Erros de linker no Linux + CUDA

```text
/usr/bin/ld: cannot find -lcudart
```

O CUDA toolkit não está no caminho de busca de bibliotecas.
Instale o toolkit e garanta que `/usr/local/cuda/lib64` está em
`LD_LIBRARY_PATH`, ou use a feature `cuda-no-vmm` que linka
contra um subconjunto menor.

## Carregamento de modelo

### `model not found` / `failed to open file`

O caminho GGUF que você passou para `LlamaParams::new(...)` não
existe ou não é legível. Use o script de conveniência para baixar
fixtures que sabemos que funcionam:

```bash
./scripts/download_models.sh smol    # ~400 MB modelo de texto
./scripts/download_models.sh bge     # ~30 MB modelo de embedding
./scripts/download_models.sh gemma4  # ~5 GB modelo de visão + projetor
```

### `failed to allocate context` / out-of-memory

O modelo precisa de mais memória do que está disponível.
Mitigações, em ordem de impacto:

1. Escolha um quant menor (`Q4_K_M` → `Q3_K_M` → `Q2_K`).
2. Diminua `n_ctx` (ex. `4096 → 2048`).
3. Reduza `n_gpu_layers` para manter mais camadas na CPU.
4. Troque de backend (Metal → CPU quando VRAM é o gargalo).

### GPU não detectada / `supports_gpu_offload()` retorna false

Você compilou sem a feature de GPU para sua plataforma. No
Linux/CUDA:

```toml
llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
```

No macOS, a feature `metal` está ativa por padrão para `aarch64`.
Em macOS Intel, você deve compilar apenas para CPU.

## Multimodal (mtmd)

### `mtmd.h` não encontrado / `MtmdContext` não está no escopo

A API multimodal é protegida pela feature do Cargo `mtmd`:

```toml
llama-crab = { version = "0.1", features = ["mtmd"] }
```

### `this projector does not support vision`

O arquivo `mmproj-*.gguf` que você carregou foi treinado para
áudio (ou alguma outra modalidade). Verifique se o projetor
corresponde ao modelo de texto e à modalidade que você quer —
tanto Gemma 4 quanto LFM2.5-VL vêm com projetores de visão
separados.

## Performance

### Geração está lenta

Causas comuns:

- **Offload de camadas insuficiente** — aumente `n_gpu_layers`.
- **`n_ctx` grande** — contextos longos custam memória e tempo
  por passo.
- **Build apenas CPU em um Mac** — instale a feature `metal`.
- **Build de debug** — certifique-se de usar `cargo run --release`.

Veja a [receita de ajuste de performance](recipes/performance.md)
para um guia passo a passo.

### Embeddings são zero / similaridade é `NaN`

Você provavelmente esqueceu `with_embeddings(true)` nos params,
ou escolheu o tipo de pooling errado para o modelo. BGE / GTE /
E5 esperam `PoolingType::Cls`; modelos estilo sentence-
transformers preferem `Mean`.

### Decodificação especulativa não dá speedup

O modelo de rascunho e o modelo principal têm baixa concordância.
Escrita criativa aberta raramente se beneficia. Tente:

- Um modelo de rascunho diferente (menor, mais rápido).
- Pular a decodificação especulativa completamente.

## Servidor

### `address already in use` no startup

A porta que você escolheu está em uso. Escolha outra:

```bash
cargo run -p llama-crab-server -- --port 8081
```

Ou encontre o processo que segura a porta e pare-o:

```bash
lsof -i :8080
```

### Servidor retorna 422 em requisições de chat

A requisição é JSON bem-formado mas o servidor a rejeitou. A
causa mais comum é `tool_choice` nomeando uma função que não está
na lista `tools`. Verifique os logs do servidor para a razão
exata.

### Streaming corta no meio da resposta

O cliente HTTP fechou a conexão cedo, ou o worker entrou em
pânico. Habilite `RUST_LOG=debug` para um log mais detalhado.

## Mensagens de erro comuns

| Erro | Causa provável | Correção |
| --- | --- | --- |
| `LlamaError::Io(...)` | Arquivo não encontrado, permissão negada. | Verifique o caminho, o CWD e o modo do arquivo. |
| `LlamaError::ModelLoad("unknown architecture")` | O GGUF é para uma família de modelos que o `llama.cpp` empacotado não reconhece. | Atualize o `llama-crab` ou use um GGUF diferente. |
| `LlamaError::ContextCreate("n_ctx too large")` | O cache KV é maior que a VRAM. | Diminua `n_ctx` ou escolha um quant menor. |
| `LlamaError::Tokenize("invalid utf-8")` | O prompt contém bytes não-UTF-8. | Sanitize o prompt antes de tokenizar. |
| `LlamaError::BackendNotInitialised` | Você chamou uma API de baixo nível sem um `LlamaBackend` ativo. | Segure um guard `LlamaBackend` pelo tempo de vida do modelo. |

## Ainda travado?

- [Abra uma issue][GitHub issues] com a saída de
  `cargo build -vv` e a saída das sondas de capacidade do
  `llama_crab::LlamaBackend`.
- Para questões de design, a aba [Discussions] é melhor que
  issues.
- O [Discord](https://discord.gg/llama-crab) (se existir) é o
  caminho mais rápido para uma resposta em tempo real.

[GitHub issues]: https://github.com/DominguesM/llama-crab/issues
[Discussions]: https://github.com/DominguesM/llama-crab/discussions
