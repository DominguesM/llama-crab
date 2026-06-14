# `speculative` — Decodificação por rascunho prompt-lookup

Demonstra [`PromptLookupDecoding`]: escaneia o prompt em busca dos
últimos `n` tokens e emite o que veio depois deles como rascunho.
Sem modelo extra necessário.

## Execute

=== "Um comando"

    ```bash
    ./examples/run.sh speculative
    ```

=== "Manual"

    ```bash
    ./scripts/download_models.sh smol
    cargo run --release --bin speculative
    ```

Baixa o `Qwen2.5-0.5B-Instruct-GGUF` (~400 MB).

## O que ele faz

```rust
use llama_crab::speculative::{DraftModel, PromptLookupDecoding};
use llama_crab::{Llama, LlamaParams};

let llama = Llama::load(LlamaParams::new("models/qwen2.5-0.5b-instruct-q4_k_m.gguf")
    .with_n_ctx(1024))?;

let prompt = "Rust is fast and memory safe. Rust is fast";
let prompt_tokens = llama.model().tokenize(prompt, true, true)?;

let draft = PromptLookupDecoding::new(3, 8);
let drafted = draft.draft(&prompt_tokens, 8);

let drafted_text = llama.model().detokenize(&drafted, false)?;
println!("rascunho texto> {}", drafted_text.trim());
```

O prompt contém uma repetição (`"Rust is fast"` aparece duas
vezes), então o rascunho pega `"and memory safe"` e o emite como
os tokens candidatos seguintes.

## Saída esperada

```
prompt> Rust is fast and memory safe. Rust is fast
drafted token ids> [Token(...), Token(...), ...]
drafted text> and memory safe
```

## Knobs de ajuste

| Knob | Descrição | Faixa típica |
| --- | --- | --- |
| `max_ngram_size` | Quantos tokens finais formam a chave de busca. | 2–4 |
| `num_pred_tokens` | Quantos tokens emitir quando uma correspondência é encontrada. | 4–16 |

`max_ngram_size` maior encontra mais correspondências mas é mais
sensível a pequenas edições. `num_pred_tokens` maior reduz a
sobrecarga de verificação por token aceito, mas um rascunho errado
é mais caro de recuperar.

## Quando isso ajuda

- **Prompts repetitivos** — código, listas, RAG que cita o
  contexto.
- **Infill FIM** — o corpo de uma função aparece antes no
  arquivo.
- **Prompts templados longos** — o prompt de sistema se repete.

Para escrita criativa aberta, a aceitação cai e a sobrecarga
pode exceder a economia. Meça antes de adotar.

## Modelos de rascunho customizados

O trait `DraftModel` deixa você plugar qualquer estratégia de
proposição de tokens — um modelo menor, um autômato de regex, uma
máquina de estados finita, um trie de frases comuns:

```rust
use llama_crab::speculative::DraftModel;
use llama_crab::token::LlamaToken;

struct AlwaysHello;
impl DraftModel for AlwaysHello {
    fn draft(&self, _input: &[LlamaToken], n: usize) -> Vec<LlamaToken> {
        // Substitua por: amostre n tokens do seu modelo menor.
        Vec::new()
    }
}
```

Depois dirija o passo especulativo com a função livre
[`speculative_decode`].

## Quando a aceitação é baixa demais

Algumas regras de ouro:

| Sintoma | Causa provável | Correção |
| --- | --- | --- |
| Aceitação < 30% | O prompt não é repetitivo o suficiente. | Tente um modelo de rascunho diferente (um GGUF instruct pequeno). |
| Aceitação > 80% mas speedup é pequeno | O passo de rascunho é lento demais. | Use um rascunho menor, ou `PromptLookupDecoding`. |
| Speedup é negativo | O modelo principal já é pequeno. | Decodificação especulativa raramente ajuda modelos sub-1B. |

## Código-fonte completo

[`examples/speculative/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/speculative/src/main.rs).

## Por onde ir a partir daqui

- [Guia de decodificação especulativa](../features/speculative.md) —
  a referência completa, incluindo modelos de rascunho
  customizados.
- [Receita de ajuste de performance](../recipes/performance.md) —
  meça throughput com e sem decodificação especulativa.

[`PromptLookupDecoding`]: https://docs.rs/llama-crab/latest/llama_crab/speculative/struct.PromptLookupDecoding.html
[`speculative_decode`]: https://docs.rs/llama-crab/latest/llama_crab/speculative/fn.speculative_decode.html
