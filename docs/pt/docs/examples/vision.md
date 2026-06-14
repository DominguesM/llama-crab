# `vision` — Multimodal imagem + texto

Um exemplo multimodal de alto nível que emparelha um GGUF de texto
com um projetor `mmproj`, carrega uma imagem, executa uma única
passada de inferência e imprime o turno do assistant. Requer a
feature do Cargo `mtmd`.

## Execute

```bash
./examples/run.sh vision gemma4
# ou
./examples/run.sh vision lfm-vl
```

Baixa ~1–5 GB dependendo do modelo. O alvo `lfm-vl` é menor e mais
rápido; `gemma4` é a opção mais pesada e de maior qualidade.

## O que ele faz

```rust
use llama_crab::multimodal::{MtmdBitmap, MtmdContext, MtmdInputText};
use llama_crab::sampling::LlamaSampler;
use llama_crab::token::LlamaToken;
use llama_crab::{Llama, LlamaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut llama = Llama::load(
        LlamaParams::new("gemma-4-E4B-it-Q4_K_M.gguf").with_n_ctx(4096),
    )?;
    let mtmd = MtmdContext::init_from_file(
        "gemma-4-E4B-it-mmproj.gguf",
        llama.model(),
    )?;
    let bitmap = MtmdBitmap::from_file("image.png")?;
    let chunks = mtmd.tokenize(
        MtmdInputText::new("Describe this image in one sentence."),
        &[&bitmap],
    )?;
    let ctx_ptr = llama.context().raw_handle();
    let new_n_past = unsafe {
        chunks.eval(&mtmd, ctx_ptr, 0, 0, llama.context().n_batch() as i32, true)?
    };
    let mut sampler = LlamaSampler::greedy()?;
    let mut out = String::new();
    let eos = llama.model().token_eos();
    let mut next_pos = new_n_past;
    for _ in 0..128 {
        let tok: LlamaToken = unsafe { sampler.sample(ctx_ptr, -1) };
        sampler.accept(tok);
        if tok == eos { break; }
        if let Ok(piece) = llama.model().detokenize(&[tok], false) {
            out.push_str(&piece);
        }
        let single = llama_crab::batch::LlamaBatch::one(tok, next_pos, 0, true);
        llama.context().decode(&single)?;
        next_pos += 1;
    }
    println!("assistant> {out}");
    Ok(())
}
```

## Saída esperada

```
assistant> A red-and-blue checker pattern with 16x16 squares on a 256x256 canvas.
```

A descrição real depende da imagem de teste e do modelo.

## Imagem de teste

O repositório vem com um `tests/fixtures/test_image.png` sintético
— uma imagem RGB 256×256 com um padrão de xadrez. Use para
verificar o fluxo ponta a ponta sem precisar de uma foto real:

```bash
cargo run --features mtmd --bin vision --release -- \
  models/gemma-4-E4B-it-Q4_K_M.gguf \
  models/mmproj-gemma-4-E4B-it-BF16.gguf \
  tests/fixtures/test_image.png
```

## Variações comuns

=== "Prompt diferente"

    ```rust
    let chunks = mtmd.tokenize(
        MtmdInputText::new("Quais são as cores dominantes nesta imagem?"),
        &[&bitmap],
    )?;
    ```

=== "Múltiplas imagens"

    ```rust
    let bitmap_a = MtmdBitmap::from_file("a.png")?;
    let bitmap_b = MtmdBitmap::from_file("b.png")?;
    let chunks = mtmd.tokenize(
        MtmdInputText::new("Compare as duas imagens."),
        &[&bitmap_a, &bitmap_b],
    )?;
    ```

=== "Sampler diferente"

    ```rust
    use llama_crab::sampling::SamplerChain;
    let mut sampler = SamplerChain::new()
        .temp(0.7)
        .top_p(0.9, 1)
        .build();
    ```

## Armadilhas

- **`mmproj` errado** — Gemma 4 e LFM2.5-VL vêm com projetores
  diferentes. Use o que corresponde ao modelo de texto.
- **Imagem muito grande** — bitmaps grandes desperdiçam memória
  e tornam a avaliação lenta. Use `MtmdBitmap::resize_to` para
  reduzir para a resolução ótima do VLM (geralmente 336×336 a
  896×896).
- **A feature `mtmd` não está habilitada** — o exemplo falha em
  compilar. Adicione `features = ["mtmd"]` à dependência.

## Código-fonte completo

[`examples/vision/src/main.rs`](https://github.com/DominguesM/llama-crab/tree/main/examples/vision/src/main.rs).

## Por onde ir a partir daqui

- [Guia multimodal](../features/multimodal.md) — o fluxo de dados
  e a API de avaliação de chunks.
- [API mtmd bruta](mtmd.md) — quando você precisa de mais
  controle do que os helpers de alto nível expõem.
- [Servidor com visão](../server/api.md#chat-multimodal) — o
  caminho HTTP.
