# Sampling guide

`llama-crab` exposes every sampling strategy from `llama.cpp` as an
associated function on [`LlamaSampler`]. Use [`SamplerChain`] to
compose them.

## Strategies

| Strategy                                                 | Purpose                                      |
| -------------------------------------------------------- | -------------------------------------------- |
| `LlamaSampler::greedy()`                                 | Always pick the highest-probability token.   |
| `LlamaSampler::dist(seed)`                               | Uniform random sampling.                     |
| `LlamaSampler::top_k(k)`                                 | Restrict to the top K tokens.                |
| `LlamaSampler::top_p(p, min_keep)`                       | Nucleus sampling.                            |
| `LlamaSampler::min_p(p, min_keep)`                       | Min-P sampling.                              |
| `LlamaSampler::typical(p, min_keep)`                     | Locally-typical sampling.                    |
| `LlamaSampler::temp(t)`                                  | Temperature scaling.                         |
| `LlamaSampler::temp_ext(t, delta, exp)`                  | Dynamic temperature.                         |
| `LlamaSampler::xtc(p, t, min_keep, seed)`                | Exclude top choices.                         |
| `LlamaSampler::top_n_sigma(n)`                           | Top-N-Sigma.                                 |
| `LlamaSampler::mirostat(n_vocab, seed, tau, eta, m)`     | Mirostat v1.                                 |
| `LlamaSampler::mirostat_v2(seed, tau, eta)`              | Mirostat v2.                                 |
| `LlamaSampler::penalties(...)`                           | Repetition / frequency / presence penalties. |
| `LlamaSampler::dry(model, ...)`                          | "Don't Repeat Yourself" sampler.             |
| `LlamaSampler::adaptive_p(target, decay, seed)`          | Adaptive-P probabilistic.                    |
| `LlamaSampler::logit_bias(n_vocab, biases)`              | Manual logit-bias.                           |
| `LlamaSampler::infill(model)`                            | Code-infill sampler (FIM).                   |
| `LlamaSampler::grammar(model, ...)` _(feature `common`)_ | GBNF-constrained sampling.                   |

## Composing a chain

The recommended way is the [`SamplerChain`] builder:

```rust,no_run
use llama_crab::sampling::SamplerChain;

let chain = SamplerChain::new()
    .temp(0.8)
    .top_p(0.95, 1)
    .min_p(0.05, 1)
    .penalties(64, 1.1, 0.0, 0.0)
    .build();
# let _ = chain;
```

The order matters: each stage sees the candidate set as transformed by
the previous one. A typical chain is:

1. **Penalties** (most aggressive — prune bad tokens first)
2. **Temperature / top-k / top-p / min-p** (truncate the tail)
3. **Mirostat / adaptive-p / dist / greedy** (pick one — usually the last)
4. **Grammar** (if any — must be last)

## Low-level

If you need to bypass the builder, the raw API is also available:

```rust,no_run
use llama_crab::sampling::LlamaSampler;
let greedy = LlamaSampler::greedy();
# let _ = greedy;
```

[`LlamaSampler`]: https://docs.rs/llama-crab/latest/llama_crab/sampling/struct.LlamaSampler.html
[`SamplerChain`]: https://docs.rs/llama-crab/latest/llama_crab/sampling/struct.SamplerChain.html
