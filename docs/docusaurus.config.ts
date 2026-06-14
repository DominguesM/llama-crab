import type {Config} from '@docusaurus/types';
import type {Options as ClassicPresetOptions} from '@docusaurus/preset-classic';
import {themes as prismThemes} from 'prism-react-renderer';

const config: Config = {
  title: 'llama-crab',
  tagline: 'Rust, server, Tauri, and TypeScript tooling for local llama.cpp applications.',
  favicon: 'img/favicon.png',
  url: 'https://dominguesm.github.io',
  baseUrl: '/llama-crab/',
  organizationName: 'DominguesM',
  projectName: 'llama-crab',
  trailingSlash: true,

  presets: [
    [
      'classic',
      {
        docs: {
          path: 'docs',
          routeBasePath: '/',
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/DominguesM/llama-crab/tree/main/docs/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies ClassicPresetOptions,
    ],
  ],

  themeConfig: {
    image: 'img/social-card.png',
    navbar: {
      title: 'llama-crab',
      logo: {
        alt: 'llama-crab',
        src: 'img/logo.webp',
      },
      items: [
        {to: '/getting-started/', label: 'Start', position: 'left'},
        {to: '/rust/', label: 'Rust', position: 'left'},
        {to: '/server/', label: 'Server', position: 'left'},
        {to: '/tauri/', label: 'Tauri', position: 'left'},
        {to: '/typescript/', label: 'TypeScript', position: 'left'},
        {to: '/examples/', label: 'Examples', position: 'left'},
        {href: 'https://dominguesm.github.io/llama-crab/api/rust/llama_crab/', label: 'Rust API', position: 'right'},
        {href: 'https://github.com/DominguesM/llama-crab', label: 'GitHub', position: 'right'},
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {label: 'Getting started', to: '/getting-started/'},
            {label: 'Examples', to: '/examples/'},
            {label: 'Troubleshooting', to: '/troubleshooting/'},
          ],
        },
        {
          title: 'API',
          items: [
            {label: 'Rust API', href: 'https://dominguesm.github.io/llama-crab/api/rust/llama_crab/'},
            {label: 'TypeScript API', href: 'https://dominguesm.github.io/llama-crab/api/typescript/'},
            {label: 'docs.rs', href: 'https://docs.rs/llama-crab'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'GitHub', href: 'https://github.com/DominguesM/llama-crab'},
            {label: 'crates.io', href: 'https://crates.io/crates/llama-crab'},
            {label: 'npm packages', to: '/typescript/packages/'},
          ],
        },
      ],
      copyright: `Copyright (c) ${new Date().getFullYear()} llama-crab contributors.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['bash', 'json', 'toml', 'rust', 'typescript'],
    },
  },
};

export default config;
