# Distribuição mobile

`llama-crab` não envia artefatos mobile pré-construídos — você
constrói o crate Rust para seu alvo e agrupa a biblioteca ou
binário produzido com seu app. Esta página cobre os perfis de
release, as receitas para iOS e Android, a integração OpenCL + NDK
e os presets de runtime.

## Perfis de release

O workspace define dois perfis de release para empacotamento mobile:

| Perfil | Caso de uso | Trade-offs |
| --- | --- | --- |
| `release-perf` | Performance máxima de runtime. | Binário maior, link mais longo, LTO thin. |
| `release-size` | Artefato menor com LTO fat, stripping de símbolos, `panic = "abort"`. | Melhor para apps distribuídos em lojas. |

```bash
cargo build --profile release-perf
cargo build --profile release-size
```

## iOS

Use a feature `metal` para offload da GPU Apple:

```bash
cargo build --profile release-perf \
    --target aarch64-apple-ios \
    --no-default-features --features metal
```

Para artefatos apenas CPU menores, desabilite as features padrão
e use `release-size`:

```bash
cargo build --profile release-size \
    --target aarch64-apple-ios-sim \
    --no-default-features --features openmp
```

Mantenha os arquivos de modelo fora do binário e carregue-os do
armazenamento do app — embarcar um GGUF de 4 GB dentro do `.ipa`
raramente é uma boa ideia.

## Android CPU

Para builds Android focados em CPU, comece com OpenMP e KleidiAI:

```bash
cargo build --profile release-size \
    --target aarch64-linux-android \
    --no-default-features --features openmp,kleidiai
```

`static-stdcxx` e `shared-stdcxx` selecionam o runtime C++ do
Android. São **mutuamente exclusivos**. Se nenhuma feature for
definida, o build mantém o padrão histórico `c++_static`.

## Android OpenCL

Para builds OpenCL orientados a Adreno, instale cabeçalhos OpenCL
e um ICD loader para o NDK alvo. Depois compile com OpenCL
habilitado:

```bash
cargo build --profile release-perf \
    --target aarch64-linux-android \
    --no-default-features --features opencl,shared-stdcxx
```

O build encaminha estas variáveis de ambiente para o CMake quando
definidas:

| Variável | Propósito |
| --- | --- |
| `OpenCL_LIBRARY` | Caminho para a biblioteca OpenCL. |
| `OPENCL_HEADERS_DIR` | Caminho para os cabeçalhos OpenCL. |
| `OPENCL_ICD_LOADER_HEADERS_DIR` | Caminho usado ao construir um ICD loader. |

OpenCL e KleidiAI requerem SDKs alvo e validação de dispositivo. A
CI padrão apenas verifica a fiação de features do Cargo com
`dynamic-link`; ela **não** prova que um SDK Android, driver ou
dispositivo específico consegue rodar o backend.

## Presets de runtime

A API de alto nível expõe [`MobilePreset`], um conjunto compacto de
padrões ajustados para os cenários mobile mais comuns:

| Preset | Quando usar |
| --- | --- |
| `LowRam` | Celulares antigos, Android Go, smartwatches. 1–2 GB de RAM livre. |
| `Balanced` | Celulares modernos, 4 GB+ de RAM livre. |
| `GpuMax` | Dispositivos com uma GPU rápida (Adreno 7xx+, Apple A-series). |

```rust
use llama_crab::{Llama, LlamaParams, MobilePreset};

let mut llama = Llama::load(
    LlamaParams::new("modelo.gguf")
        .with_mobile_preset(MobilePreset::Balanced)
        .with_n_ctx(2048),
)?;
```

Chame setters explícitos após `with_mobile_preset` quando precisar
sobrescrever um valor individual:

```rust
use llama_crab::{Llama, LlamaParams, MobilePreset};

let mut llama = Llama::load(
    LlamaParams::new("modelo.gguf")
        .with_mobile_preset(MobilePreset::LowRam)
        .with_n_ctx(1024)        // sobrescreve o n_ctx do preset
        .with_n_threads(2),      // sobrescreve a contagem de threads
)?;
```

O servidor expõe os mesmos presets através de
`--mobile-preset low-ram`, `--mobile-preset balanced` e
`--mobile-preset gpu-max`.

## Checklist de empacotamento

Uma checklist curta para enviar um binário `llama-crab` dentro de
um app mobile:

- [ ] Escolha o triple alvo certo (`aarch64-apple-ios`,
      `aarch64-linux-android`, …).
- [ ] Escolha um perfil de release (`release-perf` para usuários
      avançados, `release-size` para a App Store).
- [ ] Agrupe o GGUF como asset baixável, não como recurso
      embutido.
- [ ] Adicione uma UI de "baixar modelo" em runtime — modelos são
      grandes.
- [ ] Pré-aqueça o modelo em uma thread de background para que o
      primeiro prompt do usuário não pague o custo de carga.
- [ ] Monitore a memória; no Android, dispare um GC após carregar
      o modelo.
- [ ] No iOS, declare o Privacy manifest para qualquer dado que o
      app envia para o modelo (a inferência em si fica no
      dispositivo).

## Armadilhas comuns

| Armadilha | Correção |
| --- | --- |
| Erro de linker: `cannot find -lomp` | Habilite a feature `openmp` e link contra o runtime OpenMP disponível no alvo. |
| Erro de linker: `cannot find -lOpenCL` | Instale cabeçalhos OpenCL e um ICD loader no sysroot do NDK, ou use as variáveis CMake `OPENCL_*`. |
| App rejeitado pela App Store por "tamanho de binário excessivo" | Use `release-size` e apenas as features `openmp` / `opencl`. |
| Crash durante carga do modelo no Android Go | Diminua o tamanho do modelo ou use `MobilePreset::LowRam` com um `n_ctx` menor. |
| Primeiro token leva 5+ segundos | Pré-aqueça o modelo em uma thread de background no início do app. |

## Por onde ir a partir daqui

- [Backends & offload de GPU](backends.md) — escolha o backend
  certo para o dispositivo.
- [Features do Cargo](../getting-started/cargo-features.md) — o
  conjunto completo de flags de feature.
- [Servidor](../server/index.md) — quando você quer um processo
  separado para hospedar o modelo.

[`MobilePreset`]: https://docs.rs/llama-crab/latest/llama_crab/enum.MobilePreset.html
