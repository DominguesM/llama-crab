---
title: Examples
---

# Examples

Examples are executable documentation. When adding or changing one:

1. Add the example crate under `examples/`.
2. Add it to the root Cargo workspace.
3. Wire it into `examples/run.sh` if it should be part of the public runner.
4. Document the expected model target and visible output.
5. Update the Docusaurus examples section.

For fast script validation, use the repository smoke test when available:

```bash
bash tests/scripts_smoke.sh
```

Functional model runs should use release mode unless the bug specifically
concerns debug behavior.
