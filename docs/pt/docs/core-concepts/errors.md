# Tratamento de erros

`llama-crab` reporta cada falha recuperável através do enum
[`LlamaError`]. Esta página explica as variantes, quando cada uma
é levantada, e os padrões que a API segura usa para expô-las.

## O enum `LlamaError`

```rust
pub enum LlamaError {
    /// Erro de I/O (arquivo de modelo não encontrado, falha de leitura, etc.).
    Io(std::io::Error),
    /// Erro de parse GGUF (arquivo presente, mas inválido).
    Gguf(String),
    /// O arquivo de modelo não pôde ser aberto ou carregado.
    ModelLoad(String),
    /// O contexto não pôde ser criado (sem memória, n_ctx muito grande).
    ContextCreate(String),
    /// Falha de tokenização.
    Tokenize(String),
    /// Falha de detokenização.
    Detokenize(String),
    /// Falha de criação ou amostragem do sampler.
    Sampling(String),
    /// Falha do stack multimodal.
    Multimodal(String),
    /// O backend não está inicializado (raro; usualmente pego no startup).
    BackendNotInitialised,
    /// Um erro customizado ou desconhecido retornado por uma função C++.
    Other(String),
}
```

O enum implementa `std::error::Error + Send + Sync`, então compõe
com `anyhow::Error` e `thiserror` sem cerimônia. Os métodos de alto
nível em `Llama` retornam `Result<T, LlamaError>`.

## Variantes comuns em detalhe

### `LlamaError::Io`

Levantada quando o arquivo de modelo não pode ser aberto, quando
um arquivo de tokenizador está faltando, ou quando o cache em disco
encontra um erro de I/O. O `std::io::Error` interno carrega os
detalhes a nível de SO:

```rust
match Llama::load(LlamaParams::new("ausente.gguf")) {
    Ok(llama) => { /* … */ }
    Err(LlamaError::Io(e)) => eprintln!("arquivo não encontrado: {e}"),
    Err(e) => eprintln!("outro erro: {e}"),
}
```

Mitigações:

- Certifique-se de que o caminho está correto.
- Baixe o modelo novamente com `scripts/download_models.sh <alvo>`.
- Para caminhos relativos, verifique o diretório de trabalho do
  binário em tempo de execução (pode diferir do `pwd` do seu shell).

### `LlamaError::ModelLoad`

Levantada quando o arquivo está aberto mas o modelo não pode ser
carregado — tipicamente por causa de uma arquitetura não suportada,
um arquivo GGUF corrompido, ou uma incompatibilidade de versão com
o `llama.cpp` empacotado.

Mitigações:

- Baixe o GGUF novamente.
- Confirme que o arquivo não está truncado (`ls -lh` vs tamanho
  esperado).
- Abra uma issue com a mensagem de erro exata e o identificador
  do modelo.

### `LlamaError::ContextCreate`

Levantada quando não há memória suficiente para alocar o cache KV.
As duas alavancas que você tem são `n_ctx` (tamanho do cache KV) e
`n_gpu_layers` (offload de GPU).

Mitigações, em ordem de impacto:

1. Diminua `n_ctx` (ex. `4096 → 1024`).
2. Diminua `n_gpu_layers` para manter mais camadas na CPU (menos
   VRAM, mais RAM).
3. Troque para um quant mais agressivo (`Q4_K_M → Q3_K_M → Q2_K`).
4. Troque de backend (Metal → CPU quando VRAM é o gargalo).

### `LlamaError::Tokenize` e `LlamaError::Detokenize`

Levantadas quando o texto de entrada contém bytes que não podem
ser tokenizados, ou quando o id do token está fora do range. Muito
raro na prática.

### `LlamaError::Multimodal`

Levantada pela feature `mtmd`. Causas típicas:

- O arquivo `mmproj` não corresponde ao modelo de texto.
- A imagem é grande demais para caber no contexto.
- A feature `mtmd` não está habilitada no build (o tipo nem existe
  nesse caso).

### `LlamaError::BackendNotInitialised`

Levantada apenas quando a API de baixo nível é usada sem um guard
`LlamaBackend` ativo. A API de alto nível `Llama::load` sempre
inicializa o backend, então usuários da API de alto nível nunca
verão esta variante.

## Padrões para expor erros

### Mapear para mensagens voltadas ao usuário

Um padrão comum é converter o erro da biblioteca em uma string
plana para exibição:

```rust
fn run() -> Result<String, String> {
    let mut llama = Llama::load(LlamaParams::new("modelo.gguf"))
        .map_err(|e| format!("não foi possível carregar o modelo: {e}"))?;
    let resp = llama.create_completion("Hello", 32)
        .map_err(|e| format!("completion falhou: {e}"))?;
    Ok(resp.text)
}
```

### Usar `anyhow` para código de aplicação

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let mut llama = Llama::load(LlamaParams::new("modelo.gguf"))
        .context("carregando o modelo")?;
    let resp = llama.create_completion("Hello", 32)
        .context("executando a completion")?;
    println!("{}", resp.text);
    Ok(())
}
```

### Usar `thiserror` para código de biblioteca

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Llama(#[from] llama_crab::LlamaError),

    #[error("configuração inválida: {0}")]
    Config(String),
}
```

### Recuperável vs não recuperável

A maioria das variantes de `LlamaError` são *recuperáveis* no
sentido de que o processo pode continuar rodando e responder à
próxima requisição. As duas exceções são:

- `ModelLoad` (o arquivo de modelo está inutilizável; geralmente
  uma configuração errada que você só corrige no startup).
- `ContextCreate` (pressão de memória; geralmente uma falha
  transitória que retries não ajudam).

Para um servidor, trate essas duas como fatais e saia do processo
para que um supervisor possa reiniciá-lo.

## Por onde ir a partir daqui

- [Solução de problemas](../troubleshooting.md) — receitas
  concretas para as mensagens de erro mais comuns.
- [Ciclo de vida](lifecycle.md) — o que acontece com requisições em
  andamento quando um worker atinge um erro irrecuperável.
- [Servidor](../server/index.md) — como o servidor HTTP
  empacotado converte `LlamaError` em códigos de status HTTP no
  estilo OpenAI.

[`LlamaError`]: https://docs.rs/llama-crab/latest/llama_crab/error/enum.LlamaError.html
