import svelte from "@astrojs/svelte";
import tailwind from "@astrojs/tailwind";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import { markdownConfiguration } from "./markdown.config.ts";

const SITE_URL = "https://union.build";

const PORT = Number(process.env.PORT || import.meta.env.PORT || 4321);

// https://astro.build/config
export default defineConfig({
  site: SITE_URL,
  output: "static",
  trailingSlash: "ignore",
  server: (_options) => ({
    port: PORT,
  }),
  markdown: markdownConfiguration,
  devToolbar: { enabled: false },
  integrations: [
    starlight({
      title: "Union",
      tagline: "Connecting blockchains trustlessly",
      description:
        "Union is a hyper-efficient, zero-knowledge interoperability layer that connects Appchains, Layer 1, and Layer 2 networks.",
      favicon: "/favicon.ico",
      lastUpdated: true,
      social: {
        github: "https://github.com/unionlabs",
        discord: "https://discord.union.build",
        "x.com": "https://x.com/union_build",
      },
      head: [
        {
          tag: "meta",
          attrs: {
            name: "description",
            content: "The Sovereign Interoperability Layer",
          },
        },
        {
          tag: "meta",
          attrs: {
            name: "og:image",
            content: "/og.png",
          },
        },
        {
          tag: "meta",
          attrs: {
            name: "twitter:image",
            content: "/og.png",
          },
        },
        {
          tag: "script",
          attrs: { src: "/scripts/anchor-targets.js" },
        },
        {
          tag: "script",
          attrs: { src: "/scripts/edit-page-link.js" },
        },
        {
          // math rendering breaks without this
          tag: "link",
          attrs: {
            rel: "stylesheet",
            href: "https://www.unpkg.com/katex@0.16.9/dist/katex.min.css",
          },
        },
      ],
      locales: {
        root: { label: "English", lang: "en" },
      },
      defaultLocale: "root",
      editLink: {
        baseUrl: "https://discord.union.build",
      },
      logo: {
        alt: "Union Logo",
        dark: "./src/assets/union-logo/union-logo-transparent.svg",
        light: "./src/assets/union-logo/union-logo-white-transparent.svg",
      },
      sidebar: [
        {
          label: "Introduction",
          link: "/docs",
        },
        {
          label: "Architecture",
          autogenerate: {
            directory: "/docs/architecture",
          },
        },
        {
          label: "Concepts",
          autogenerate: {
            directory: "/docs/concepts",
          },
        },
        {
          label: "Infrastructure",
          items: [
            {
              label: "Node Operators",
              collapsed: true,
              autogenerate: {
                directory: "/docs/infrastructure/node-operators",
              },
            },
          ],
        },
        {
          label: "Integration",
          autogenerate: {
            directory: "/docs/integration",
          },
        },
        {
          label: "Demos",
          autogenerate: {
            directory: "/docs/demos",
          },
        },
        {
          label: "Joining the Testnet",
          autogenerate: {
            directory: "/docs/joining-testnet",
          },
        },
      ],
      plugins: [],
      customCss: [
        "./src/styles/fonts.css",
        "./src/styles/tailwind.css",
        "./src/styles/starlight.css",
      ],
    }),
    tailwind({
      applyBaseStyles: false,
      configFile: "tailwind.config.ts",
    }),
    svelte(),
  ],
  vite: {
    optimizeDeps: {
      exclude: ["@urql/svelte", "echarts"],
    },
  },
});
