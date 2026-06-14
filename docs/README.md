# `llama-crab` documentation

This folder contains the user guide for the `llama-crab` Rust
bindings. The site is built with
[Material for MkDocs](https://squidfunk.github.io/mkdocs-material/)
and is available in **English** and **Portuguese (Brasil)**.

## Site structure

```
docs/
├── mkdocs.yml              # English (canonical) configuration
├── pt/
│   └── mkdocs.yml          # Portuguese configuration
├── requirements.txt        # Python dependencies for the build
├── assets/
│   └── css/extra.css       # Custom theme styles
├── overrides/              # Theme customisation
│   └── partials/           # (empty by default — extend as needed)
├── en/                     # English content
│   ├── index.md            # English home
│   ├── getting-started/    # English getting started
│   ├── core-concepts/      # English core concepts
│   ├── guides/             # English guides
│   ├── features/           # English features
│   ├── server/             # English server
│   ├── examples/           # English examples
│   ├── recipes/            # English recipes
│   ├── reference/          # English reference
│   ├── troubleshooting.md  # English troubleshooting
│   └── about/              # English about
└── pt/                     # Portuguese translations (one file per
                            # English page, mirror structure)
    ├── docs/               # Portuguese content
    └── mkdocs.yml
```

## Building locally

### One-time setup

```bash
cd docs
python3 -m venv .venv
source .venv/bin/activate           # or `.venv\Scripts\activate` on Windows
pip install -r requirements.txt
```

### Serve with hot-reload

From the `docs/` directory:

```bash
# English site (served at http://127.0.0.1:8000/)
mkdocs serve --config-file mkdocs.yml

# Portuguese site (served at http://127.0.0.1:8000/pt/)
mkdocs serve --config-file pt/mkdocs.yml
```

`mkdocs serve` watches the source files and rebuilds on every
save. Both sites can be served simultaneously on different ports
if you need to check the language selector:

```bash
mkdocs serve --config-file mkdocs.yml --dev-addr 127.0.0.1:8000 &
mkdocs serve --config-file pt/mkdocs.yml --dev-addr 127.0.0.1:8001 &
```

### Build for production

```bash
# English → ./site/en/
mkdocs build --strict --config-file mkdocs.yml --site-dir /tmp/site/en

# Portuguese → ./site/pt/
mkdocs build --strict --config-file pt/mkdocs.yml --site-dir /tmp/site/pt
```

The `--strict` flag turns warnings into errors, which keeps the
docs source clean (no broken links, no missing nav entries).

## Multi-language setup

We use the [mkdocs-material monorepo pattern][monorepo] for
multi-language documentation:

- One `mkdocs.yml` per language, in a subfolder.
- The English `mkdocs.yml` lives at the root of `docs/` and is
  the canonical configuration. It reads from `docs/en/`.
- The Portuguese `mkdocs.yml` lives at `docs/pt/mkdocs.yml` and
  reads from `docs/pt/docs/`.
- Both configs set `site_url` and `extra.alternate` so the
  language selector in the header links between
  `/llama-crab.github.io/en/` and `/llama-crab.github.io/pt/`.
- The English `mkdocs.yml` configures the theme `logo` and
  `favicon` to point at `assets/images/logo.png` (the canarim-crab
  mark).

The GitHub Actions workflow at
`.github/workflows/docs.yml` builds both sites and publishes the
combined static output to `DominguesM/llama-crab.github.io`.

### GitHub Pages deployment

The site is deployed to the separate Pages repository
`DominguesM/llama-crab.github.io`. With the default GitHub Pages URL
for a project repository, the rendered site is available at:

`https://dominguesm.github.io/llama-crab.github.io/`

One-time setup:

1. In `DominguesM/llama-crab.github.io`, enable GitHub Pages:
   `Settings -> Pages -> Source: Deploy from a branch -> Branch: main -> Folder: /`.
2. Create a token that can push to `DominguesM/llama-crab.github.io`.
   A fine-grained personal access token with repository
   `contents:write` permission is sufficient.
3. Add that token to `DominguesM/llama-crab` as the repository
   secret `DOCS_PAGES_TOKEN`.

The workflow then:

1. Builds the English site into `site_root/en/`.
2. Builds the Portuguese site into `site_root/pt/`.
3. Copies the English `index.html` to `site_root/index.html` so
   that visiting the root URL shows the English landing page.
4. Adds `.nojekyll`.
5. Pushes the rendered static files to the `main` branch of
   `DominguesM/llama-crab.github.io`.

The workflow is manual-only by design, so merging docs/deploy-only
changes does not publish a new site or trigger library CI/CD. To
publish the docs from `main`:

```bash
gh workflow run docs.yml --ref main
```

The resulting URLs are:

- `https://dominguesm.github.io/llama-crab.github.io/` (English landing page)
- `https://dominguesm.github.io/llama-crab.github.io/en/` (English)
- `https://dominguesm.github.io/llama-crab.github.io/pt/` (Portuguese)

The mkdocs-material "stay on page" feature works because both
sites share the same page structure: JavaScript in the theme
swaps the language prefix in the URL when the user clicks the
language selector.

[monorepo]: https://squidfunk.github.io/mkdocs-material/setup/changing-the-language/#site-language-selector

## Adding a new page

1. Create the Markdown file in the appropriate `docs/en/<section>/`
   directory.
2. Add the file path to the `nav` section of `docs/mkdocs.yml`.
3. Mirror the page in `docs/pt/docs/<section>/` and add it to
   `docs/pt/mkdocs.yml`.

The `nav:` tree in both `mkdocs.yml` files must stay in sync. If
you add a page in one language but not the other, the build will
warn — and `--strict` turns warnings into errors.

## Conventions

- Use the [Material for MkDocs] reference for content features
  (admonitions, content tabs, code blocks, mermaid diagrams, …).
- Wrap Rust snippets with ` ```rust ` and add `,no_run` if the
  example should not run as part of the doctests.
- Use absolute links to docs.rs / crates.io where possible so
  they keep working if the page is reused elsewhere.
- Keep the English file as the source of truth and translate to
  Portuguese in lockstep. PRs that change one language without
  the other will be asked to update the second language.

## Theme customisation

The `overrides/` folder is empty by default. To add a partial:

```bash
mkdir -p overrides/partials
touch overrides/partials/your-partial.html
```

Then reference it from `mkdocs.yml`:

```yaml
theme:
    name: material
    custom_dir: overrides
```

See the [Material for MkDocs "Extending the theme" guide][extend]
for details.

[Material for MkDocs]: https://squidfunk.github.io/mkdocs-material/
[extend]: https://squidfunk.github.io/mkdocs-material/customization/#extending-the-theme

## License

The documentation is distributed under the same MIT License as the
crate. See [`LICENSE-MIT`](../LICENSE-MIT) for the full text.
