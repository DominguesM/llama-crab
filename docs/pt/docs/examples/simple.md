# `simple` — Completion de texto simples

O menor programa possível: carregue um modelo, gere uma completion,
imprima o resultado. Use como ponto de partida para uma ferramenta
CLI one-shot ou como template quando quiser controle total sobre
a cadeia de sampler.

## Execute

```bash
cargo run --bin simple --release -- modelo.gguf
```

O primeiro argumento posicional é o caminho para um modelo GGUF.

## O que ele faz

```rust
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(
        LlamaParams::new("modelo.gguf")
            .with_n_ctx(2048)
            .with_n_gpu_layers(99),
    )?;
    let resp = llama.create_completion("Era uma vez", 64)?;
    println!("{}", resp.text);
    Ok(())
}
```

## Saída esperada

```
, havia uma menina que adorava ler.
```

O texto real depende do modelo. O ponto do exemplo é a *forma* do
programa: poucas linhas, sem cerimônia, todos os padrões aplicados.

## Customizando a chamada

Passe [`CompletionOptions`] ao helper de alto nível para expor o
resto da cadeia de sampler:

```rust
use llama_crab::{CompletionOptions, Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(LlamaParams::new("modelo.gguf"))?;
    let resp = llama.create_completion_with_options(
        "Era uma vez",
        CompletionOptions::new(64)
            .with_temperature(0.7)
            .with_top_p(0.9, 1)
            .with_stop_sequence("\n\n"),
    )?;
    print!("{}", resp.text);
    Ok(())
}
```

Veja o [guia de completions de texto](../features/text-completion.md)
para o menu completo de opções.

## Usando uma cadeia de sampler customizada

Para controle total, construa uma [`SamplerChain`] e chame
`create_completion_with_sampler`:

```rust
use llama_crab::sampling::SamplerChain;
use llama_crab::{CompletionOptions, Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(LlamaParams::new("modelo.gguf"))?;
    let mut sampler = SamplerChain::new()
        .temp(0.7)
        .top_p(0.9, 1)
        .min_p(0.05, 1)
        .penalties(64, 1.1, 0.0, 0.0)
        .build();
    let resp = llama.create_completion_with_sampler(
        "Era uma vez",
        CompletionOptions::new(64),
        &mut sampler,
    )?;
    print!("{}", resp.text);
    Ok(())
}
```

Veja o [guia de estratégias de amostragem](../guides/sampling.md)
para o menu completo de samplers e cadeias recomendadas.

## Código-fonte completo

[`examples/simple/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/simple/src/main.rs).

## Por onde ir a partir daqui

- [Streaming](streaming.md) — para saída token a token.
- [Chat](chat.md) — quando você quer mensagens baseadas em papel.
- [FIM](#) — fill-in-the-middle para completions de código.

[`CompletionOptions`]: https://docs.rs/llama-crab/latest/llama_crab/struct.CompletionOptions.html
[`SamplerChain`]: https://docs.rs/llama-crab/latest/llama_crab/sampling/struct.SamplerChain.html
