# Completions de texto

O fluxo de trabalho mais simples do `llama-crab`: alimente o modelo
com uma string de prompt, obtenha uma continuação de texto de
volta. Esta página documenta a família de métodos
`create_completion`, os knobs de sequência de parada e
log-probabilidade, streaming, best-of-N e FIM (fill-in-the-middle)
para código.

## A chamada básica

```rust
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("modelo.gguf").with_n_ctx(2048))?;
let resp = llama.create_completion("Era uma vez", 64)?;
println!("{}", resp.text);
```

A struct [`Completion`] retornada pela chamada de alto nível carrega:

| Campo | Tipo | Descrição |
| --- | --- | --- |
| `text` | `String` | O texto gerado, com o prompt removido. |
| `tokens` | `Vec<LlamaToken>` | Os ids de token gerados. |
| `logprobs` | `Option<CompletionLogprobs>` | Log-probabilidades por token, quando requisitadas. |
| `timings` | `CompletionTimings` | Ingestão do prompt, geração e tempo total de parede. |
| `stop_reason` | `StopReason` | Por que a geração parou (`Stop`, `Length`, `TokensLimit`, `Canceled`). |

## Customizando a chamada

`create_completion` é um wrapper fino sobre
`create_completion_with_options`, que recebe um builder
[`CompletionOptions`]. Use-o para expor o resto da cadeia de
sampler, as sequências de parada, as configurações de log-prob e
o knob best-of-N.

```rust
use llama_crab::{CompletionOptions, Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("modelo.gguf").with_n_ctx(2048))?;

let resp = llama.create_completion_with_options(
    "The capital of France is",
    CompletionOptions::new(32)
        .with_temperature(0.7)
        .with_top_p(0.95, 1)
        .with_top_k(40)
        .with_stop_sequence("\n\n")
        .with_logprobs(true, 5)
        .with_echo(false),
)?;
```

### Referência de `CompletionOptions`

| Método | Padrão | Descrição |
| --- | --- | --- |
| `new(max_tokens)` | – | Define o número máximo de tokens a serem gerados. |
| `with_temperature(t)` | `0.8` | Temperatura; `0.0` seleciona decodificação greedy. |
| `with_top_k(k)` | `40` | Restringe aos top K tokens. |
| `with_top_p(p, min_keep)` | `0.95, 1` | Amostragem nucleus. |
| `with_min_p(p, min_keep)` | `0.05, 1` | Amostragem Min-P. |
| `with_typical_p(p, min_keep)` | `1.0, 1` | Amostragem localmente típica. |
| `with_tfs_z(z)` | `1.0` | Amostragem tail-free. |
| `with_repeat_penalty(p)` | `1.0` | Penalidade de repetição. |
| `with_frequency_penalty(p)` | `0.0` | Penalidade de frequência. |
| `with_presence_penalty(p)` | `0.0` | Penalidade de presença. |
| `with_penalty_last_n(n)` | `64` | Tokens a considerar ao aplicar penalidades. |
| `with_mirostat(...)` | `0` | Modo Mirostat (`0` = off, `1`, `2`). |
| `with_seed(seed)` | random | Semente do RNG. |
| `with_stop_sequence(s)` | – | Adiciona uma única sequência de parada. |
| `with_stop_sequences([s])` | – | Adiciona múltiplas sequências de parada. |
| `with_logit_bias(biases)` | `{}` | Bias de logit manual. |
| `with_logprobs(enable, k)` | `false, 0` | Log-probabilidades por token. |
| `with_echo(echo)` | `false` | Ecoa o prompt de volta como parte da resposta. |
| `with_suffix(suffix)` | – | Sufixo anexado após o prompt (para FIM). |
| `with_best_of(n)` | `n` | Número de candidatos internos para `n`. |
| `with_grammar(text)` | – | Gramática GBNF (requer a feature `common`). |

## Streaming

Para UIs em tempo real, use `create_completion_stream`:

```rust
use std::io::{self, Write};
use llama_crab::{CompletionOptions, Llama, LlamaParams, StreamControl};

let mut llama = Llama::load(LlamaParams::new("modelo.gguf").with_n_ctx(512))?;
let prompt = "Escreva uma frase curta sobre Rust.";
let mut stdout = io::stdout().lock();
let mut write_error: Option<io::Error> = None;

let completion = llama.create_completion_stream(
    prompt,
    CompletionOptions::new(64).with_stop_sequence("\n\n"),
    |chunk| {
        if let Err(err) = write!(stdout, "{}", chunk.text).and_then(|_| stdout.flush()) {
            write_error = Some(err);
            return StreamControl::Stop;
        }
        StreamControl::Continue
    },
)?;

if let Some(err) = write_error {
    return Err(err.into());
}
```

O callback não pode retornar um `Result`, então capture erros de
I/O e retorne `StreamControl::Stop`; depois que o stream retorna,
propague o erro capturado.

Veja o [exemplo de streaming](../examples/streaming.md) para um
programa autocontido.

## FIM (fill-in-the-middle) para código

Modelos de completion de código esperam um prompt no estilo
`prefix<SUFFIX_FILL>middle<SUFFIX>`. `llama-crab` expõe
`complete_infill` para isso:

```rust
use llama_crab::{Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("modelo.gguf").with_n_ctx(1024))?;

let prefix = "fn main() {\n    println!(\"";
let suffix = "\");\n}";
let resp = llama.complete_infill(prefix, suffix)?;
println!("{}", resp.text);
```

O helper `complete_infill` renderiza um template
`BuiltinTemplate::CodeFim` ao redor do prefixo e sufixo, depois
roda o loop de geração normal. Certifique-se de que o modelo que
você está usando tenha sido treinado em uma tarefa FIM — os
metadados GGUF geralmente declaram isso.

## Best-of-N

`with_best_of(n)` gera `n` completions internas, pontua elas pela
log-probabilidade média e retorna o top `n` (a contagem pública de
escolhas). Use para trocar compute por qualidade:

```rust
use llama_crab::{CompletionOptions, Llama, LlamaParams};

let mut llama = Llama::load(LlamaParams::new("modelo.gguf").with_n_ctx(1024))?;
let resp = llama.create_completion_with_options(
    "def fibonacci(n):",
    CompletionOptions::new(64).with_best_of(4),
)?;
```

A `Completion` retornada tem a mesma forma de uma completion
única; os candidatos extras são internos.

## Log-probabilidades

`with_logprobs(true, k)` popula `Completion.logprobs` com as top
`k` log-probabilidades de token em cada posição, mais o token
selecionado:

```rust
pub struct CompletionLogprobs {
    pub tokens:           Vec<LlamaToken>,
    pub text_offset:      Vec<usize>,
    pub token_logprobs:   Vec<Option<f32>>,
    pub top_logprobs:     Vec<Vec<TopLogprob>>,
    pub top_logprobs_idx: Vec<usize>,
}
```

Use para estimativas de incerteza, pontuação de perplexidade, ou
para implementar uma UI customizada que mostra alternativas.

## Sequências de parada

Sequências de parada são correspondidas **após** um token ser
decodificado, no chunk detokenizado. Adicione quantas quiser; a
primeira a corresponder termina a geração. São case-sensitive e
sensíveis a whitespace.

```rust
CompletionOptions::new(64)
    .with_stop_sequence("\n\n")
    .with_stop_sequence("</answer>")
    .with_stop_sequence("User:")
```

O comportamento padrão corresponde a qualquer uma delas.

## Ecoando o prompt

`with_echo(true)` retorna o prompt como parte do texto gerado.
Útil para debugging, menos útil em produção.

## Dicas de performance

- **Reuse o modelo.** Cada `Llama::load` é O(segundos). Carregue
  uma vez, chame muitas vezes.
- **Ajuste `n_threads` para os cores físicos.** Um Mac de 16
  cores com hyperthreading deve usar 8–12 threads, não 16.
- **Use `temp = 0.0` para benchmarks.** Escolhe o sampler greedy
  e dá as temporizações mais reproduzíveis.
- **Descarregue para a GPU quando o modelo cabe na VRAM.** Uma
  chamada `with_n_gpu_layers(99)` é usualmente 5–10× mais rápida
  que CPU.
- **Defina `with_seed` com um valor fixo para testes unitários.** A
  semente padrão do RNG é aleatória por chamada, o que torna
  assertions de teste flaky.

## Por onde ir a partir daqui

- [Estratégias de amostragem](../guides/sampling.md) — para
  controle total sobre a cadeia de sampler.
- [Exemplo de streaming](../examples/streaming.md) — um programa
  autocontido que faz streaming de tokens para stdout.
- [Decodificação especulativa](speculative.md) — quando você
  precisa de mais tokens por segundo do que o modelo pode
  produzir nativamente.

[`Completion`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Completion.html
[`CompletionOptions`]: https://docs.rs/llama-crab/latest/llama_crab/struct.CompletionOptions.html
