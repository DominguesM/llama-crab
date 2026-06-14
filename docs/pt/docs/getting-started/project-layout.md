# Estrutura do projeto

Um projeto típico com `llama-crab` tem três partes móveis: um
binário (ou biblioteca) Rust, um arquivo de modelo GGUF e —
opcionalmente — um projetor `mmproj` para modelos visão-linguagem.
Esta página mostra uma estrutura que mantém as partes móveis fáceis
de encontrar e fáceis de trocar.

## Estrutura recomendada

```
meu-app/
├── Cargo.toml
├── src/
│   └── main.rs
├── models/                  # Arquivos de modelo GGUF (não versionados)
│   ├── qwen2.5-7b-instruct-q4_k_m.gguf
│   └── mmproj-qwen2.5-vl-q8_0.gguf
├── prompts/                 # opcional: templates de chat, prompts de sistema
│   └── system.txt
└── tests/
    └── integration.rs
```

O diretório `models/` é intencionalmente **não** versionado: modelos
são binários grandes que você baixa fora do repositório. Adicione
ao `.gitignore`:

```gitignore title=".gitignore"
/models/
```

## Baixando modelos

O repositório vem com um helper `scripts/download_models.sh` que
busca fixtures que sabemos que funcionam do Hugging Face. Copie
para o seu próprio projeto, ou chame de um script de setup:

=== "Baixar um pequeno modelo de chat"

    ```bash
    ./scripts/download_models.sh smol
    ```

=== "Baixar um modelo de embedding"

    ```bash
    ./scripts/download_models.sh bge
    ```

=== "Baixar um modelo de visão + projetor"

    ```bash
    ./scripts/download_models.sh gemma4
    ```

Se preferir usar a CLI do `huggingface_hub` diretamente:

=== "Python (huggingface_hub)"

    ```bash
    pip install -U "huggingface_hub[cli]"
    hf download Qwen/Qwen2.5-0.5B-Instruct-GGUF \
        qwen2.5-0.5b-instruct-q4_k_m.gguf \
        --local-dir models
    ```

=== "Fallback com curl"

    ```bash
    mkdir -p models
    curl -L "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf" \
        -o models/qwen2.5-0.5b-instruct-q4_k_m.gguf
    ```

## Boilerplate de Cargo.toml

Um `Cargo.toml` mínimo e amigável para produção:

```toml title="Cargo.toml"
[package]
name        = "meu-app"
version     = "0.1.0"
edition     = "2021"
rust-version = "1.88"

[dependencies]
llama-crab = { version = "0.1", default-features = false, features = ["metal", "openmp"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow     = "1"

[profile.release]
opt-level     = 3
lto           = "thin"
codegen-units = 1
strip         = "debuginfo"
```

!!! tip "Fixe o caminho do modelo"

    Prefira carregar o caminho do modelo de uma variável de ambiente
    ou de uma flag CLI em vez de hardcodar no binário. Isso torna o
    mesmo binário executável contra múltiplos arquivos GGUF na CI.

## Esqueleto de um único binário

```rust title="src/main.rs"
use std::env;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_path = env::args()
        .nth(1)
        .or_else(|| env::var("LLAMA_CRAB_MODEL").ok())
        .unwrap_or_else(|| "models/qwen2.5-0.5b-instruct-q4_k_m.gguf".into());

    let mut llama = Llama::load(
        LlamaParams::new(&model_path)
            .with_n_ctx(2048)
            .with_n_threads(4),
    )?;

    let resp = llama.create_completion("Hello, world!", 64)?;
    print!("{}", resp.text);
    Ok(())
}
```

## Projetos com múltiplos binários

Se você quer vários binários compartilhando o mesmo carregador
de modelo, separe o código em uma biblioteca e um binário fino:

```
meu-app/
├── Cargo.toml
├── src/
│   ├── lib.rs            # pub fn load_model() -> Result<Llama, _>
│   ├── chat.rs
│   └── server.rs
├── src/bin/
│   ├── chat.rs           # usa meu_app::chat
│   └── server.rs         # usa meu_app::server
└── models/
```

`Cargo.toml`:

```toml title="Cargo.toml"
[package]
name        = "meu-app"
version     = "0.1.0"
edition     = "2021"

[lib]
name = "meu_app"
path = "src/lib.rs"

[[bin]]
name = "chat"
path = "src/bin/chat.rs"

[[bin]]
name = "server"
path = "src/bin/server.rs"

[dependencies]
llama-crab = { version = "0.1", features = ["metal", "openmp"] }
```

## Trabalhando com o runner de exemplos

O repositório vem com um wrapper `examples/run.sh` que baixa o
modelo certo, compila o binário certo e o executa. Você pode ler
o script para aprender como montar o seu próprio:

```bash
# O que ele faz:
# 1. Resolve o nome do exemplo → "alvo-de-download|nome-do-binário".
# 2. Chama ./scripts/download_models.sh <alvo>.
# 3. Chama cargo run --release --bin <nome>.
```

As duas variáveis de ambiente que vale a pena conhecer são:

- `LLAMA_CRAB_SKIP_DOWNLOAD=1` — pula a etapa de download do modelo.
- `LLAMA_CRAB_DRY_RUN=1` — imprime o comando sem executá-lo.

## Por onde ir a partir daqui

- [Features do Cargo](cargo-features.md) — ajuste o build para seu
  alvo.
- [Índice de exemplos](../examples/index.md) — copie um programa
  inicial que já roda.
- [Servidor](../server/index.md) — se seu projeto precisa de um
  endpoint HTTP, use `llama-crab-server` diretamente.
