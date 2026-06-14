# Backends & offload de GPU

`llama-crab` é construído sobre `llama.cpp`, que delega o trabalho
pesado de álgebra linear a um **backend**. O backend ativo é
escolhido em *tempo de build* através de features do Cargo; você
pode misturar trabalho de CPU e GPU em *tempo de execução* ao
descarregar um número escolhido de camadas do transformer para a
GPU.

## Escolhendo um backend

| Backend | Feature do Cargo | Padrão? | Quando escolher |
| --- | --- | --- | --- |
| CPU (OpenMP) | `openmp` | sim | Sempre ativo. Eleva inferência na CPU a múltiplos cores. |
| Apple Metal | `metal` | sim em `aarch64-apple-darwin` | Apple Silicon. Melhor perf-por-watt. |
| NVIDIA CUDA | `cuda` | – | Linux + NVIDIA. Melhor throughput bruto em GPUs grandes. |
| NVIDIA CUDA (sem VMM) | `cuda-no-vmm` | – | CUDA sem gerenciamento de memória virtual. |
| Vulkan / SPIR-V | `vulkan` | – | Compute de GPU cross-vendor. Cai de volta para CPU graciosamente. |
| AMD ROCm / HIP | `rocm` | – | Linux + AMD. |
| OpenCL | `opencl` | – | Android Adreno e Arm64. |
| KleidiAI CPU kernels | `kleidiai` | – | Alvos Arm mobile. |
| Linkagem dinâmica | `dynamic-link` | – | Linka llama.cpp como biblioteca compartilhada. |
| Backends dinâmicos | `dynamic-backends` | – | Carrega backends GGML dinamicamente. |
| GGML do sistema | `system-ggml` | – | Pula o build GGML empacotado, usa um do sistema. |

Veja a [referência de features do Cargo](../reference/cargo-features.md)
para a lista canônica.

### Um Cargo.toml recomendado

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

## Inicializando o backend

`LlamaBackend::init()` é chamado automaticamente quando você
carrega um modelo através do orquestrador de alto nível [`Llama`].
Se você dirige a API de baixo nível diretamente, segure um guard
[`LlamaBackend`] por todo o tempo de vida do modelo — derrubá-lo
derruba o backend.

```rust
use llama_crab::{LlamaBackend, NumaStrategy};

// Backend padrão.
let _backend = LlamaBackend::init()?;

// Inicialização ciente de NUMA. Distribute, Isolate ou Numactl.
let _backend = LlamaBackend::init_numa(NumaStrategy::Distribute)?;
```

### Sondas de capacidade

O backend expõe um punhado de sondas de capacidade que você pode
chamar em tempo de execução para detectar o que está disponível:

| Método | O que ela diz |
| --- | --- |
| `supports_gpu_offload()` | Algum backend de GPU (Metal, CUDA, Vulkan, ROCm) está disponível. |
| `supports_mmap()` | Carregamento de modelo memory-mapped está disponível. |
| `supports_mlock()` | `mlock` (fixar modelo na RAM) está disponível. |
| `supports_rpc()` | Inferência RPC distribuída está disponível. |

```rust
let backend = LlamaBackend::init()?;
if backend.supports_gpu_offload() {
    println!("Offload de GPU está disponível");
} else {
    println!("Apenas CPU");
}
```

## Offload de camadas

`LlamaParams::with_n_gpu_layers(n)` controla quantas camadas do
transformer são empurradas para a GPU. Passe um número grande
(`99`) para descarregar o modelo inteiro; passe `0` para rodar
inteiramente na CPU.

```rust
use llama_crab::{Llama, LlamaParams};

// Descarregar totalmente um modelo pequeno na GPU.
let llama = Llama::load(
    LlamaParams::new("modelo.gguf")
        .with_n_ctx(2048)
        .with_n_gpu_layers(99),
)?;
```

O knob "offload" é um contador por camada que caminha pelo modelo
do embedding de entrada em direção à saída. Definir como `N`
significa "as primeiras N camadas rodam na GPU, as `total - N`
restantes rodam na CPU".

### Quando usar offload parcial

O offload de camadas é mais útil em três regimes:

1. **O modelo cabe na GPU** — defina `n_gpu_layers` para o número
   de camadas no modelo. Todas as camadas rodam na GPU; as threads
   de CPU ficam ociosas.
2. **O modelo é maior que a VRAM** — defina `n_gpu_layers` para a
   maior contagem que cabe na VRAM. A cauda do modelo roda na CPU
   e os dados cruzam o barramento PCIe entre as camadas. A queda
   de throughput é graciosa (tipicamente 2–4× por camada cruzada).
3. **Máquinas apenas CPU** — defina `n_gpu_layers = 0`. O modelo
   roda inteiramente na CPU usando threads OpenMP.

### Uma regra de ouro rápida

| Tamanho do quant | GPU 8 GB | GPU 16 GB | GPU 24 GB |
| --- | --- | --- | --- |
| 7B Q4_K_M (~4 GB) | 99 camadas | 99 camadas | 99 camadas |
| 13B Q4_K_M (~7,5 GB) | 99 camadas | 99 camadas | 99 camadas |
| 70B Q4_K_M (~40 GB) | 10–15 camadas | 20–25 camadas | 35–40 camadas |

Os números dependem muito do vocabulário do modelo, do tamanho da
cabeça e do comprimento do contexto. Use-os como ponto de partida,
depois meça com seu próprio prompt.

## Threads de CPU

Para execuções apenas CPU ou híbridas, controle a contagem de
threads com:

```rust
use llama_crab::{Llama, LlamaParams};

let llama = Llama::load(
    LlamaParams::new("modelo.gguf")
        .with_n_threads(8)         // threads para ingestão do prompt
        // .with_n_threads_batch(8) // contagem separada para batches
)?;
```

Um ponto de partida razoável é o número de cores **físicos**. Em
Apple Silicon, o número de cores de *performance* é um alvo melhor
que a contagem total de cores.

## Flash attention

Flash attention é opt-in via `LlamaContextParams::with_flash_attn`:

```rust
use llama_crab::{Llama, LlamaParams};

let llama = Llama::load(
    LlamaParams::new("modelo.gguf")
        .with_n_ctx(4096)
        .with_n_gpu_layers(99)
        .with_flash_attn(true),
)?;
```

Reduz memória e acelera inferência de contexto longo na maioria
das arquiteturas modernas (Gemma, Llama 3, Qwen2.5, …).

## Multi-GPU

`llama-crab` expõe multi-GPU através do modelo de layer-split do
`llama.cpp`. A API `LlamaParams` expõe um único knob
`n_gpu_layers`; para splits granulares entre múltiplos dispositivos,
dirija a API `llama-crab-sys` diretamente. A [documentação do
`llama.cpp`](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)
cobre os mecanismos subjacentes em detalhe.

## Quando o backend não pode ser inicializado

| Sintoma | Causa provável | Correção |
| --- | --- | --- |
| `BackendNotInitialised` no startup | API de baixo nível chamada sem `LlamaBackend::init()`. | Segure um guard `LlamaBackend` pelo tempo de vida do modelo. |
| Erro de linker no Metal | Feature `metal` não habilitada. | Adicione `features = ["metal"]` à dependência. |
| Erro de linker no CUDA | CUDA toolkit não está no `PATH`. | Instale o CUDA toolkit e garanta que `nvcc` está acessível. |
| Loader OpenCL não encontrado | `OPENCL_HEADERS_DIR` / `OPENCL_ICD_LOADER_HEADERS_DIR` não definidos. | Veja o [guia de Distribuição mobile](mobile.md). |

## Por onde ir a partir daqui

- [Distribuição mobile](mobile.md) — as receitas para iOS e Android.
- [Estratégias de amostragem](sampling.md) — o que fazer com os
  logits uma vez que o modelo os produz.
- [Cache & estado de sessão](caching.md) — persista e restaure
  manualmente o estado KV.

[`Llama`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html
[`LlamaBackend`]: https://docs.rs/llama-crab/latest/llama_crab/struct.LlamaBackend.html
