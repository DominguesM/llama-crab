# Features do Cargo

`llama-crab` expõe um rico conjunto de features do Cargo para que
você compile apenas o que realmente precisa. O conjunto padrão
cobre os casos mais comuns — CPU via OpenMP, mais Metal em Apple
Silicon — mas binários de produção devem sempre fixar a combinação
exata de features que corresponde ao ambiente alvo.

## Features padrão

```toml
[dependencies]
llama-crab = "0.1"
```

Expande para:

```toml
features = ["openmp"]
# Em `aarch64-apple-darwin`, também habilita "metal".
```

Isso é suficiente para rodar todos os exemplos e a maioria dos
chatbots em um laptop. Para produção, você quase sempre quer ser
explícito:

```toml
[dependencies]
llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
```

## Matriz de features

### Backends de computação

| Feature | O que ela adiciona | Notas |
| --- | --- | --- |
| `openmp` | Backend CPU com OpenMP. | Padrão. |
| `metal` | Backend Apple GPU. | Padrão em `aarch64-apple-darwin`. |
| `cuda` | Backend NVIDIA CUDA. | Mutuamente exclusivo com `cuda-no-vmm`. |
| `cuda-no-vmm` | CUDA sem gerenciamento de memória virtual. | Use em sistemas onde CUDA VMM é restrito. |
| `vulkan` | Backend Vulkan / SPIR-V. | Funciona na maioria das GPUs (NVIDIA, AMD, Intel, Apple). |
| `rocm` | Backend AMD ROCm/HIP. | Requer uma toolchain ROCm recente. |
| `opencl` | Backend OpenCL, primariamente para Android Adreno e dispositivos Arm64. | Requer cabeçalhos OpenCL e um ICD loader. |
| `kleidiai` | Kernels KleidiAI CPU para alvos Arm mobile. | Combina com `openmp` ou `opencl`. |
| `dynamic-link` | Linka o llama.cpp como objeto compartilhado em vez de estático. | Reduz tempo de build; requer `libllama.so/.dylib/.dll` pré-construído. |
| `dynamic-backends` | Carrega backends GGML dinamicamente. | Útil para arquiteturas de plugins. |
| `system-ggml` | Usa uma instalação GGML do sistema em vez da cópia empacotada. | Pula a etapa de build do GGML. |

### Subsistemas opcionais

| Feature | O que ela adiciona |
| --- | --- |
| `mtmd` | Suporte multimodal através de `mtmd.h`; ativa helpers de imagem e áudio. Necessário para visão. |
| `common` | Compila os utilitários `common` do llama.cpp usados pelos helpers de chat e gramática. Necessário para JSON-Schema → GBNF e o sampler `grammar`. |
| `llguidance` | Habilita a integração do sampler [`llguidance`](https://github.com/microsoft/llguidance). Mais rápido e flexível que o sampler GBNF para gramáticas complexas. |
| `hf-tokenizer` | Habilita a integração com o crate `tokenizers` do Hugging Face. Use quando carregar um modelo a partir de um `tokenizer.json` em vez do tokenizador embutido no GGUF. |
| `disk-cache` | Habilita o cache de prompt persistente baseado em `sled`. |

### Apenas mobile / Android

| Feature | O que ela adiciona |
| --- | --- |
| `shared-stdcxx` | Usa `c++_shared` para builds Android. |
| `static-stdcxx` | Usa `c++_static` para builds Android (o padrão histórico). |

Estas duas são **mutuamente exclusivas**. Se nenhuma for definida,
o Android mantém o comportamento legado de `c++_static`.

## Combinações recomendadas

=== "Laptop macOS (Apple Silicon)"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
    ```

=== "Servidor Linux com NVIDIA H100"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
    ```

=== "Servidor Linux com AMD MI300X"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["rocm", "openmp"] }
    ```

=== "App iOS"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["metal"] }
    ```

    Compile com o perfil dedicado:

    ```bash
    cargo build --profile release-perf --target aarch64-apple-ios \
        --no-default-features --features metal
    ```

=== "Celular Android (Snapdragon / Adreno)"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["openmp", "kleidiai", "shared-stdcxx"] }
    ```

    Compile com o perfil otimizado para tamanho:

    ```bash
    cargo build --profile release-size --target aarch64-linux-android \
        --no-default-features --features openmp,kleidiai,shared-stdcxx
    ```

=== "Carga de trabalho visão-linguagem"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp", "mtmd"] }
    ```

## Detectando quais features estão ativas

O `LlamaBackend` compilado expõe algumas sondas de capacidade que
você pode chamar em tempo de execução:

```rust
use llama_crab::LlamaBackend;

let backend = LlamaBackend::init()?;
println!("Offload GPU: {}", backend.supports_gpu_offload());
println!("mmap       : {}", backend.supports_mmap());
println!("mlock      : {}", backend.supports_mlock());
println!("RPC        : {}", backend.supports_rpc());
```

São particularmente úteis para diagnóstico em binários que
distribuem para múltiplos alvos.

## E quanto às features padrão na CI?

A CI fixa as combinações de features que ela de fato exercita:

| Linha da matriz CI | Features |
| --- | --- |
| `linux-cpu`     | `openmp` |
| `linux-cuda`    | `cuda`, `openmp` |
| `linux-vulkan`  | `vulkan`, `openmp` |
| `linux-rocm`    | `rocm`, `openmp` |
| `macos-metal`   | `metal`, `openmp` |
| `macos-cpu`     | `openmp` |
| `windows-cpu`   | `openmp` |

Isso garante que os caminhos de código que cada backend expõe
continuam funcionando release após release. Se você quer que um
backend seja oficialmente suportado na CI, abra uma issue e
proponha a adição à matriz.

## Por onde ir a partir daqui

- [Distribuição mobile](../guides/mobile.md) — as receitas para
  iOS e Android e os padrões `MobilePreset`.
- [Backends & offload de GPU](../guides/backends.md) — como
  escolher um backend e como `n_gpu_layers` funciona.
- [Referência de features do Cargo](../reference/cargo-features.md) —
  a mesma tabela, com a descrição longa de cada feature.
