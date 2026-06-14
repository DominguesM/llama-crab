import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docs: [
    'intro',
    {
      type: 'category',
      label: 'Getting Started',
      link: {type: 'doc', id: 'getting-started/index'},
      items: [
        'getting-started/installation',
        'getting-started/first-run',
        'getting-started/models',
        'getting-started/backends',
      ],
    },
    {
      type: 'category',
      label: 'Rust SDK',
      link: {type: 'doc', id: 'rust/index'},
      items: [
        'rust/workspace',
        'rust/lifecycle',
        'rust/completion',
        'rust/chat',
        'rust/embeddings',
        'rust/multimodal',
        'rust/structured-output',
      ],
    },
    {
      type: 'category',
      label: 'Server',
      link: {type: 'doc', id: 'server/index'},
      items: ['server/running', 'server/openai-api', 'server/streaming', 'server/operations'],
    },
    {
      type: 'category',
      label: 'Tauri',
      link: {type: 'doc', id: 'tauri/index'},
      items: ['tauri/plugin', 'tauri/permissions', 'tauri/typescript-client'],
    },
    {
      type: 'category',
      label: 'TypeScript',
      link: {type: 'doc', id: 'typescript/index'},
      items: ['typescript/packages', 'typescript/client-contracts'],
    },
    {
      type: 'category',
      label: 'Guides',
      link: {type: 'doc', id: 'guides/index'},
      items: ['guides/performance', 'guides/mobile', 'guides/troubleshooting-models'],
    },
    {
      type: 'category',
      label: 'Reference',
      link: {type: 'doc', id: 'reference/index'},
      items: ['reference/crates', 'reference/cargo-features', 'reference/versioning'],
    },
    {
      type: 'category',
      label: 'Contributing',
      link: {type: 'doc', id: 'contributing/index'},
      items: ['contributing/development', 'contributing/examples', 'contributing/releases'],
    },
    'troubleshooting',
  ],
};

export default sidebars;
