# Cache & estado de sessão

`llama-crab` expõe dois mecanismos complementares de persistência:

- **Cache de prompt / KV** — reusa os tensores de chave-valor
  computados para um *prefixo* de prompt visto anteriormente.
  Economiza a passada de prefill.
- **Estado de sessão** — serializa o estado KV inteiro (e
  opcionalmente o estado do sampler / optimizer) de uma sequência
  para bytes.

Ambos são **manuais** em v0.1.x. O helper de alto nível
`create_completion` limpa a sequência 0 do KV antes de cada
chamada, então sempre começa na posição 0 e não consulta o cache
de prompt automaticamente. Se você precisa usar esses mecanismos,
dirija a API de baixo nível diretamente — ou use o padrão de
thread worker que o [servidor](../server/index.md) envia.

## Cache de prompt

O armazenamento do cache de prompt vive no módulo [`cache`]. Ambas
as implementações seguem o trait [`Cache`], que armazena e busca
bytes de sessão opacos para prefixos de tokens.

| Implementação | Feature do Cargo | Armazenamento |
| --- | --- | --- |
| [`RamCache`] | _(sempre)_ | `BTreeMap` em processo. |
| [`DiskCache`] | `disk-cache` | Banco de dados `sled` em disco. |

### Usando `RamCache`

```rust
use llama_crab::cache::{Cache, CacheEntry, RamCache};

let cache = RamCache::new();
let tokens: Vec<llama_crab::token::LlamaToken> = Vec::new();

// Depois de computar o estado KV para `tokens` uma vez:
cache.store(&tokens, CacheEntry {
    state: vec![],
    n_past: tokens.len() as i32,
});

// Na próxima chamada com uma sequência mais longa que começa com `tokens`:
if let Some(hit) = cache.lookup(&tokens) {
    println!("cache hit na posição {}", hit.n_past);
}
```

O cache é chaveado na **sequência exata de tokens** e retorna o
prefixo mais longo que corresponde. Se você pedir por `[A, B, C, D]`
e o cache tem `[A, B]`, a busca retorna a entrada para `[A, B]` e
você pode usá-la para pular o prefill dos dois primeiros tokens.

### Usando `DiskCache`

Para persistência entre reinicializações do processo, habilite a
feature `disk-cache` e troque para `DiskCache::open(path)`:

```toml title="Cargo.toml"
[dependencies]
llama-crab = { version = "0.1", features = ["disk-cache"] }
```

```rust
use llama_crab::cache::DiskCache;

let cache = DiskCache::open("./.llama-crab-cache")?;
```

O cache em disco é seguro para compartilhar entre instâncias de
`Llama` no mesmo processo; apenas o último escritor vence em
colisões de chave.

### Quando o cache de prompt ajuda

O cache compensa em três regimes:

- **Loops de chat manuais** — cada turno prefixa o histórico
  completo; se você restaurar o estado em cache você mesmo, um
  hit de prefixo pode pular reavaliar turnos anteriores.
- **RAG** — incorpore o mesmo chunk de documento múltiplas vezes
  entre queries.
- **Prompts templados** — prompt de sistema + exemplos few-shot
  são repetidos entre queries.

O cache **não** ajuda quando:

- Cada chamada usa um prompt fresco sem prefixo comum.
- Você fica no caminho de alto nível `create_completion` sem
  restaurar manualmente o estado em cache.
- O modelo é pequeno o suficiente para que o prefill não seja o
  gargalo de qualquer forma.

## Estado de sessão

As funções `llama_state_get_data` e `llama_state_set_data`
serializam o estado KV completo (e o estado de amostragem aprendido
do modelo, se houver) de uma sequência para um buffer de bytes. É
isso que `RamCache` e `DiskCache` armazenam por baixo dos panos.

O orquestrador de alto nível [`Llama`] ainda não encapsula essas
chamadas atrás de uma API tipada; você pode dirigi-las através de
`llama.context()` se precisar de snapshots de sessão byte-exatos —
por exemplo, para suspender e retomar uma sessão longa de agente.

```rust
// Pseudocódigo — veja llama-crab-sys para a superfície completa.
let bytes = unsafe { llama.context().session_save(/*seq_id*/ 0)? };
std::fs::write("session.bin", bytes)?;

// Depois, em um novo processo:
let bytes = std::fs::read("session.bin")?;
unsafe { llama.context().session_load(/*seq_id*/ 0, &bytes)? };
```

## Reuso manual do cache KV

Se você não quer persistir estado entre processos, ainda pode se
beneficiar do cache KV gerenciando a sequência você mesmo:

```rust
use llama_crab::batch::LlamaBatch;
use llama_crab::token::LlamaToken;

// 1. Carregue um prompt de sistema longo.
let system_tokens = llama.model().tokenize(SYSTEM_PROMPT, true, true)?;
let mut batch = LlamaBatch::new(system_tokens.len(), 1);
batch.add_sequence(&system_tokens, 0, false);
batch.prepare();
llama.context().decode(&batch)?;
let mut n_past = system_tokens.len() as i32;

// 2. A cada turno, decodifique apenas a nova mensagem do usuário.
loop {
    let user_tokens = llama.model().tokenize(/* prompt */, false, true)?;
    let mut batch = LlamaBatch::new(user_tokens.len(), 1);
    batch.add_sequence(&user_tokens, 0, false);
    batch.prepare();
    llama.context().decode(&batch)?;
    n_past += user_tokens.len() as i32;

    // 3. Amostre, anexe, loop.
    // ...
}
```

O cache KV **não** é limpo entre turnos, então o turno `N+1`
paga apenas o custo de prefill da nova mensagem do usuário. Este
é o padrão usado pelo [exemplo `stateful_chat`](../examples/stateful-chat.md).

## Armadilhas de cache

| Armadilha | O que dá errado | Correção |
| --- | --- | --- |
| Dois processos compartilham um `DiskCache` | Escritores concorrentes corrompem o banco de dados. | Limite o cache a um processo ou adicione um lock de arquivo. |
| Chave de cache usa texto bruto | Um typo causa miss. | Tokenize o prompt primeiro, use os ids de token como chave. |
| Cache sobrevive a um upgrade de modelo | Estado KV antigo é incompatível com o modelo novo. | Inclua o hash do GGUF na chave do cache, ou limpe no upgrade. |
| Cache cresce sem limite | Crash de out-of-memory. | Periodicamente faça eviction de entradas antigas ou use `DiskCache`. |

## Por onde ir a partir daqui

- [Chat com estado](../features/stateful-chat.md) — chat
  multi-turno com um histórico crescente que não reproduz o
  contexto inteiro.
- [Servidor](../server/index.md) — a implementação de referência
  do padrão de thread worker.
- [Exemplo de cache](https://github.com/DominguesM/llama-crab/tree/main/examples/stateful_chat) —
  um programa executável que usa reuso manual do cache KV.

[`Cache`]: https://docs.rs/llama-crab/latest/llama_crab/cache/trait.Cache.html
[`RamCache`]: https://docs.rs/llama-crab/latest/llama_crab/cache/struct.RamCache.html
[`DiskCache`]: https://docs.rs/llama-crab/latest/llama_crab/cache/struct.DiskCache.html
[`cache`]: https://docs.rs/llama-crab/latest/llama_crab/cache/index.html
[`Llama`]: https://docs.rs/llama-crab/latest/llama_crab/struct.Llama.html
