# Instalação

`llama-crab` é distribuído no [crates.io](https://crates.io/crates/llama-crab)
e também pode ser compilado a partir de um checkout do Git. As
features padrão compilam o backend CPU (OpenMP) e — em Apple Silicon —
Metal, então a maioria dos usuários pode adicionar a dependência
e começar a compilar.

## 1. Adicione a dependência

=== "Estável (crates.io)"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = "0.1"
    ```

=== "Branch main do Git"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { git = "https://github.com/DominguesM/llama-crab", branch = "main" }
    ```

=== "Checkout local"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { path = "../llama-crab" }
    ```

!!! tip "Fixe a versão do llama.cpp"

    O crate fixa o `llama.cpp` em um commit específico, então dois
    builds da mesma versão de `llama-crab` sempre produzem a mesma
    biblioteca nativa. Você pode ver o commit fixado no badge do
    README ou através de `cargo tree -p llama-crab-sys`.

## 2. Escolha um backend

As features padrão lhe dão um binário funcional nas plataformas
mais comuns, mas em produção você quase sempre quer ser explícito:

=== "Apple Silicon (macOS)"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
    ```

=== "Linux + GPU NVIDIA"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["cuda", "openmp"] }
    ```

=== "Linux + GPU AMD"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["rocm", "openmp"] }
    ```

=== "Vulkan (qualquer fornecedor)"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["vulkan", "openmp"] }
    ```

=== "Apenas CPU"

    ```toml title="Cargo.toml"
    [dependencies]
    llama-crab = { version = "0.1", default-features = false, features = ["openmp"] }
    ```

=== "Android / mobile"

    Veja o guia dedicado [Distribuição mobile](../guides/mobile.md).

Veja a [referência de features do Cargo](../reference/cargo-features.md)
para a lista completa de features e o que cada uma ativa.

## 3. Requisitos do sistema

O script de build compila o `llama.cpp` a partir do código-fonte.
Certifique-se de que o seguinte está disponível **antes** de rodar
`cargo build`:

=== "macOS"

    ```bash
    # Xcode Command Line Tools
    xcode-select --install

    # CMake (Homebrew, ou use o do CLT se estiver presente)
    brew install cmake
    ```

=== "Debian / Ubuntu"

    ```bash
    sudo apt update
    sudo apt install -y build-essential cmake
    ```

=== "Fedora / RHEL"

    ```bash
    sudo dnf install -y gcc gcc-c++ cmake make
    ```

=== "Windows (MSVC)"

    ```powershell
    # Instale o Visual Studio 2022 com a carga de trabalho
    # "Desenvolvimento para desktop com C++", depois:
    winget install Kitware.CMake
    ```

!!! warning "O primeiro build é lento"

    Compilar todos os backends do llama.cpp leva ~3 minutos em uma
    máquina de 16 cores na primeira vez. Builds subsequentes ficam
    em cache. Para reduzir o build a frio, desabilite os backends
    que você não precisa — veja o passo 2.

## 4. Verifique a toolchain

Após a instalação, rode um `cargo build` rápido para garantir que
o CMake, o compilador e a biblioteca padrão C++ estão todos
acessíveis:

```bash
cargo new hello-crab --bin
cd hello-crab
# Adicione a dependência mostrada no passo 1, depois:
cargo build --release
```

Um build bem-sucedido imprime algo como:

```
   Compiling llama-crab-sys v0.1.300 (...)
   Compiling llama-crab v0.1.300 (...)
    Finished `release` profile [optimized] [..]
```

Você está pronto para escrever seu [primeiro programa](first-program.md).

## Opcional: baixe um modelo

O resto do guia assume que você tem um arquivo GGUF no disco. A
maneira mais fácil de obter um que sabemos que funciona é o script
helper:

=== "Menor modelo de texto (Qwen2.5 0.5B)"

    ```bash
    ./scripts/download_models.sh smol
    # → models/qwen2.5-0.5b-instruct-q4_k_m.gguf
    ```

=== "Modelo de embedding (BGE-small)"

    ```bash
    ./scripts/download_models.sh bge
    # → models/bge-small-en-v1.5-q4_k_m.gguf
    ```

=== "Modelo de visão (Gemma 4 + mmproj)"

    ```bash
    ./scripts/download_models.sh gemma4
    # → models/gemma-4-E4B-it-Q4_K_M.gguf
    # → models/mmproj-gemma-4-E4B-it-BF16.gguf
    ```

Veja [`scripts/download_models.sh`](https://github.com/DominguesM/llama-crab/blob/main/scripts/download_models.sh)
para a lista completa de alvos suportados.

## Próximos passos

- Siga o [Seu primeiro programa](first-program.md) — um `main.rs`
  de 50 linhas que exercita os caminhos mais comuns.
- Dê uma olhada na [referência de features do Cargo](../reference/cargo-features.md)
  para saber o que vem habilitado por padrão e o que ativar para
  seu alvo.
- Vá direto para um [guia de funcionalidade](../features/index.md)
  que combine com o que você quer construir.
