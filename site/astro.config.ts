import "dotenv/config";
Object.assign(process.env, { ASTRO_TELEMETRY_DISABLED: 1 });

import svelte from "@astrojs/svelte";
import tailwind from "@astrojs/tailwind";
import starlight from "@astrojs/starlight";
import { markdownConfiguration } from "./markdown.config.ts";
import { defineConfig, squooshImageService } from "astro/config";

const SITE_URL = "https://union.build";

// https://astro.build/config
export default defineConfig({
  site: SITE_URL,
  prefetch: true,
  output: "static",
  trailingSlash: "ignore",
  server: (_options) => ({
    port: Number(process.env.PORT || import.meta.env.PORT || 4321),
  }),
  markdown: markdownConfiguration,
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
      expressiveCode: {
        frames: {
          extractFileNameFromCode: true,
          showCopyToClipboardButton: true,
          removeCommentsWhenCopyingTerminalFrames: true,
        },
        themes: ["starlight-dark", "starlight-light"],
        useStarlightDarkModeSwitch: true,
        useStarlightUiThemeColors: true,
      },
      head: [
        {
          tag: "link",
          attrs: {
            rel: "stylesheet",
            href: "https://www.unpkg.com/katex@0.16.9/dist/katex.min.css",
          },
        },
      ],
      locales: {
        root: {
          label: "English",
          lang: "en",
        },
      },
      defaultLocale: "en",
      logo: {
        alt: "Union Logo",
        src: "./public/images/logo.png",
      },
      editLink: {
        baseUrl: "https://github.com/unionlabs/union/edit/main/site",
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
      customCss: [
        "./src/styles/index.css",
        "./src/styles/tailwind.css",
        "@fontsource/jetbrains-mono/400.css",
        "@fontsource/jetbrains-mono/600.css",
      ],
    }),
    tailwind({
      configFile: "tailwind.config.ts",
    }),
    svelte(),
  ],
  image: {
    service: squooshImageService(),
  },
  vite: {
    ssr: {
      noExternal: ["smartypants"],
    },
    optimizeDeps: {
      exclude: ["@urql/svelte"],
    },
  },
});
