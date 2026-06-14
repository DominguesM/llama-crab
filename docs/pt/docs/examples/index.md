# Exemplos

O repositório vem com **14 crates de exemplo autocontidos** em
[`examples/`], um por funcionalidade pública. Cada um é um crate
Cargo independente que você pode copiar.

<div class="grid cards" markdown>

-   :material-rocket-launch: __[Quickstart](quickstart.md)__

    O menor programa ponta a ponta: carregar, tokenizar, completar,
    chat, FIM. ~80 linhas, totalmente anotado.

-   :material-text-box: __[Completions simples](simple.md)__

    Completion de texto one-shot com uma cadeia de sampler
    customizada.

-   :material-broadcast: __[Streaming](streaming.md)__

    Saída token a token através da API de callback de alto nível.

-   :material-message-text: __[Chat multi-turno](chat.md)__

    Um chat de dois turnos usando um template embutido.

-   :material-console: __[REPL com estado](stateful-chat.md)__

    REPL multi-turno interativo com `/clear`, `/save`, tratamento
    de EOF.

-   :material-image-multiple: __[Visão (mtmd)](vision.md)__

    Imagem multimodal + texto com a API de alto nível
    `MtmdContext`.

-   :material-code-tags: __[API mtmd.h bruta](mtmd.md)__

    API mtmd.h de baixo nível: bitmap → chunks → eval.

-   :material-vector-arrange-above: __[Embeddings](embeddings.md)__

    Extração de embedding com normalização L2.

-   :material-magnify: __[Busca semântica](embedding-search.md)__

    BGE-small + ranqueamento cosseno sobre um corpus pequeno.

-   :material-order-alphabetical-ascending: __[Reranker](reranker.md)__

    Demo de reranker bi-encoder.

-   :material-tools: __[Tool calling](tools.md)__

    `ToolDefinition` + cinco formatos `ToolParser`.

-   :material-code-braces: __[Saída estruturada](structured.md)__

    JSON-Schema → GBNF → saída JSON restrita.

-   :material-fast-forward: __[Decodificação especulativa](speculative.md)__

    Decodificação por rascunho com `PromptLookupDecoding`.

</div>

## Runner de um comando

Cada exemplo é empacotado por `examples/run.sh`, que baixa o modelo
certo na primeira execução e é idempotente depois:

```bash
./examples/run.sh quickstart            # ~400 MB — apenas texto, menor demo
./examples/run.sh chat                  # mesmo modelo — REPL interativo
./examples/run.sh stateful_chat         # REPL multi-turno com /clear, /save
./examples/run.sh embeddings            # ~30 MB — embedding BGE-small
./examples/run.sh embedding_search      # BGE-small + ranqueamento cosseno
./examples/run.sh reranker              # pontuação bi-encoder
./examples/run.sh vision gemma4         # ~5 GB — chat de visão + texto
./examples/run.sh vision lfm-vl         # ~1 GB — modelo de visão menor
./examples/run.sh mtmd gemma4           # API mtmd.h bruta
./examples/run.sh tools                 # function calling
./examples/run.sh structured            # gramática JSON-schema
./examples/run.sh speculative           # decodificação por rascunho prompt-lookup
```

Sem argumentos, o script lista cada exemplo disponível.

## Tabela completa

| Exemplo | Modelo | Tamanho | O que mostra |
| --- | --- | --- | --- |
| [`quickstart`](quickstart.md) | `Qwen2.5-0.5B-Instruct-GGUF` | ~400 MB | Carrega → tokeniza → completa → chat → FIM |
| [`simple`](simple.md) | qualquer GGUF de texto | varia | Completion de texto simples |
| [`streaming`](streaming.md) | mesmo do `quickstart` | ~400 MB | Saída de alto nível token a token |
| [`chat`](chat.md) | GGUF instruct | varia | Chat one-shot com template embutido |
| [`stateful_chat`](stateful-chat.md) | mesmo do `quickstart` | ~400 MB | REPL com histórico crescente, `/clear`, `/save` |
| [`vision`](vision.md) | Gemma 4 ou LFM2.5-VL + mmproj | ~1–5 GB | Chat de visão de alto nível `MtmdContext` |
| [`mtmd`](mtmd.md) | Gemma 4 + mmproj | ~5 GB | API mtmd.h bruta: bitmap → chunks → eval |
| [`embeddings`](embeddings.md) | `bge-small-en-v1.5-gguf` | ~30 MB | Extração de embedding + L2 norm |
| [`embedding_search`](embedding-search.md) | `bge-small-en-v1.5-gguf` | ~30 MB | Busca semântica com ranqueamento cosseno |
| [`reranker`](reranker.md) | GGUF de embedding | varia | Ranqueamento bi-encoder por similaridade cosseno |
| [`tools`](tools.md) | GGUF instruct ciente de tools | varia | `ToolDefinition` + 5 formatos `ToolParser` |
| [`structured`](structured.md) | qualquer GGUF de texto | varia | `json_schema_grammar()` + parsing JSON |
| [`speculative`](speculative.md) | qualquer GGUF de texto | varia | Rascunho n-gram `prompt-lookup` |

## Passando um modelo diferente

Cada exemplo aceita o caminho do GGUF como **primeiro** argumento
posicional:

```bash
cargo run --release --bin run_quickstart -- models/llama-3.2-1b-instruct-q4_k_m.gguf
```

Exemplos de visão recebem `<text.gguf> <mmproj.gguf> <image>`.

## Adicionando um novo exemplo

O boilerplate para um novo crate de exemplo é ~15 linhas:

```toml title="examples/meu_exemplo/Cargo.toml"
[package]
name = "meu_exemplo"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish = false

[[bin]]
name = "run_meu_exemplo"
path = "src/main.rs"

[dependencies]
llama-crab = { path = "../../llama-crab", version = "0.1.0" }
anyhow = "1"
```

```rust title="examples/meu_exemplo/src/main.rs"
use anyhow::Result;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<()> {
    let mut llama = Llama::load(LlamaParams::new("models/seu.gguf"))?;
    let resp = llama.create_completion("Olá!", 32)?;
    print!("{}", resp.text);
    Ok(())
}
```

Depois adicione `examples/meu_exemplo` à lista `members = [...]`
no `Cargo.toml` raiz e uma linha à tabela desta página.

## Por onde ir a partir daqui

- [Quickstart](quickstart.md) — o menor programa ponta a ponta.
- [Streaming](streaming.md) — a requisição mais comum de
  desenvolvedores de apps.
- [Visão (mtmd)](vision.md) — se você quer alimentar imagens a
  um modelo.
- [Receita de chatbot](../recipes/chatbot.md) — quando um único
  exemplo não é suficiente e você precisa montar um agente
  completo.
