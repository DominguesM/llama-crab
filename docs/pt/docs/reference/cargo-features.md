# Features do Cargo

A lista canônica de features do Cargo, com a descrição longa de
cada uma. Veja o [guia de Primeiros Passos](../getting-started/cargo-features.md)
para uma visão mais curta e orientada a tarefas.

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

## Backends de computação

| Feature | Descrição |
| --- | --- |
| `openmp` | Backend CPU com OpenMP. Habilitado por padrão. |
| `metal` | Backend Apple Metal. Habilitado por padrão em `aarch64-apple-darwin`. |
| `cuda` | Backend NVIDIA CUDA. |
| `cuda-no-vmm` | Backend NVIDIA CUDA sem gerenciamento de memória virtual. |
| `vulkan` | Backend Vulkan / SPIR-V. |
| `rocm` | Backend AMD ROCm / HIP. |
| `opencl` | Backend OpenCL, primariamente para Android Adreno e dispositivos Arm64. |
| `kleidiai` | Kernels KleidiAI CPU para alvos Arm mobile. |
| `dynamic-link` | Linka llama.cpp como objeto compartilhado em vez de estático. |
| `dynamic-backends` | Carrega backends GGML dinamicamente. |
| `system-ggml` | Usa uma instalação GGML do sistema em vez da cópia empacotada. |

## Subsistemas opcionais

| Feature | Descrição |
| --- | --- |
| `mtmd` | Suporte multimodal através de `mtmd.h`; ativa helpers de imagem e áudio. Necessário para visão. |
| `common` | Compila os utilitários `common` do llama.cpp usados pelos helpers de chat e gramática. Necessário para JSON-Schema → GBNF e o sampler `grammar`. |
| `llguidance` | Habilita a integração do sampler [`llguidance`](https://github.com/microsoft/llguidance). Mais rápido e flexível que o sampler GBNF para gramáticas complexas. |
| `hf-tokenizer` | Habilita a integração com o crate `tokenizers` do Hugging Face. Use quando carregar um modelo a partir de um `tokenizer.json` em vez do tokenizador embutido no GGUF. |
| `disk-cache` | Habilita o cache de prompt persistente baseado em `sled`. |

## Apenas mobile / Android

| Feature | Descrição |
| --- | --- |
| `shared-stdcxx` | Usa `c++_shared` para builds Android. |
| `static-stdcxx` | Usa `c++_static` para builds Android. O padrão histórico. |

Estas duas são **mutuamente exclusivas**. Se nenhuma for definida,
o Android mantém o comportamento legado de `c++_static`.

## Grupos mutuamente exclusivos

| Grupo | Escolha no máximo um |
| --- | --- |
| Variante CUDA | `cuda`, `cuda-no-vmm` |
| Runtime C++ Android | `shared-stdcxx`, `static-stdcxx` |

O `build.rs` do crate falhará o build com um erro claro se duas
features mutuamente exclusivas forem habilitadas juntas.

## Combinações recomendadas

=== "Apple Silicon (macOS)"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
    ```

=== "Linux + NVIDIA"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
    ```

=== "Linux + AMD"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["rocm", "openmp"] }
    ```

=== "Cross-vendor (Vulkan)"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["vulkan", "openmp"] }
    ```

=== "Apenas CPU"

    ```toml
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["openmp"] }
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

=== "Android (Snapdragon / Adreno)"

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

Se você quer que um backend seja oficialmente suportado na CI,
abra uma issue e proponha a adição à matriz.

## Por onde ir a partir daqui

- [Features do Cargo (Primeiros Passos)](../getting-started/cargo-features.md) —
  a visão orientada a tarefas.
- [Backends & offload de GPU](../guides/backends.md) — a
  configuração em tempo de execução.
- [Distribuição mobile](../guides/mobile.md) — as receitas para
  iOS e Android.
