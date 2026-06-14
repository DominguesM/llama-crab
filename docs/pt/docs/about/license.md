# Licença

`llama-crab` é distribuído sob a **Licença MIT**. O texto completo
segue.

---

MIT License

Copyright (c) 2024-2026 contribuidores do llama-crab

A permissão é concedida, gratuitamente, a qualquer pessoa que
obtenha uma cópia deste software e arquivos de documentação
associados (o "Software"), a lidar com o Software sem restrições,
incluindo, sem limitação, os direitos de usar, copiar, modificar,
fundir, publicar, distribuir, sublicenciar e/ou vender cópias do
Software, e a permitir pessoas para quem o Software é fornecido
a fazê-lo, sujeito às seguintes condições:

O aviso de copyright acima e este aviso de permissão devem ser
incluídos em todas as cópias ou porções substanciais do Software.

O SOFTWARE É FORNECIDO "COMO ESTÁ", SEM GARANTIA DE QUALQUER
TIPO, EXPRESSA OU IMPLÍCITA, INCLUINDO, MAS NÃO SE LIMITANDO ÀS
GARANTIAS DE COMERCIABILIDADE, ADEQUAÇÃO A UM PROPÓSITO
PARTICULAR E NÃO VIOLAÇÃO. EM NENHUM CASO OS AUTORES OU
TITULARES DOS DIREITOS AUTORAIS SERÃO RESPONSÁVEIS POR
QUALQUER RECLAMAÇÃO, DANOS OU OUTRA RESPONSABILIDADE, SEJA EM
AÇÃO DE CONTRATO, ATO ILÍCITO OU DE OUTRA FORMA, DECORRENTE DE,
FORA DE OU EM CONEXÃO COM O SOFTWARE OU O USO OU OUTRAS
NEGOCIAÇÕES NO SOFTWARE.

---

## Licenças de terceiros

`llama-crab` linka contra e depende dos seguintes projetos. Cada
um carrega sua própria licença; consulte o projeto upstream para
detalhes.

| Projeto | Licença |
| --- | --- |
| [`llama.cpp`](https://github.com/ggml-org/llama.cpp) | MIT. |
| [`ggml`](https://github.com/ggml-org/ggml) | MIT. |
| `serde`, `serde_json`, `anyhow`, `thiserror`, `tokio`, `axum` | MIT ou Apache 2.0 (por crate). |
| `mkdocs-material` (apenas build da documentação) | MIT. |

O crate é licenciado MIT, mas **o modelo que você carrega pode não
ser**. Cada modelo no Hugging Face tem sua própria licença;
verifique o card do modelo antes de distribuir um binário que
embute os pesos do modelo.

## Por onde ir a partir daqui

- [Agradecimentos](acknowledgements.md) — as pessoas e projetos
  que tornam o `llama-crab` possível.
- [Contribuindo](contributing.md) — como enviar uma correção para
  um bug que você encontrou.
