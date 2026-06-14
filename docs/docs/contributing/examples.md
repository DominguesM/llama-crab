---
title: Examples
---

# Examples

Examples are executable documentation and now live in
[`llama-crab-examples`](https://github.com/DominguesM/llama-crab-examples).
When adding or changing one:

1. Add the example crate in the `llama-crab-examples` repository.
2. Add it to that repository's root Cargo workspace.
3. Wire it into `run.sh` if it should be part of the public runner.
4. Document the expected model target and visible output.
5. Update this Docusaurus examples section.

For fast script validation, use the examples repository smoke test:

```bash
bash tests/examples_repo_smoke.sh
```

Functional model runs should use release mode unless the bug specifically
concerns debug behavior.
