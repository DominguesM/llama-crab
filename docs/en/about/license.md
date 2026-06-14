# License

`llama-crab` is distributed under the **MIT License**. The full text
follows.

---

MIT License

Copyright (c) 2024-2026 llama-crab contributors

Permission is hereby granted, free of charge, to any person
obtaining a copy of this software and associated documentation
files (the "Software"), to deal in the Software without
restriction, including without limitation the rights to use, copy,
modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.

---

## Third-party licenses

`llama-crab` links against and depends on the following projects.
Each carries its own license; consult the upstream project for
details.

| Project | License |
| --- | --- |
| [`llama.cpp`](https://github.com/ggml-org/llama.cpp) | MIT. |
| [`ggml`](https://github.com/ggml-org/ggml) | MIT. |
| `serde`, `serde_json`, `anyhow`, `thiserror`, `tokio`, `axum` | MIT or Apache 2.0 (per crate). |
| `mkdocs-material` (documentation build only) | MIT. |

The crate is licensed MIT, but **the model you load may not be**.
Each model on Hugging Face has its own license; check the model
card before distributing a binary that embeds the model weights.

## Where to next?

- [Acknowledgements](acknowledgements.md) — the people and projects
  that make `llama-crab` possible.
- [Contributing](contributing.md) — how to send a fix for a bug
  you found.
