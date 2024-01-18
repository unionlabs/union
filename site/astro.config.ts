import "dotenv/config";
import svelte from "@astrojs/svelte";
import tailwind from "@astrojs/tailwind";
import starlight from "@astrojs/starlight";
import { markdownConfiguration } from "./markdown.config.ts";
import { defineConfig, squooshImageService } from "astro/config";

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
      head: [],
      locales: {
        root: {
          label: "English",
          lang: "en",
        },
      },
      defaultLocale: "en",
      logo: {
        alt: "Union Logo",
        src: "./public/images/union-logo.svg",
      },
      editLink: {
        baseUrl: "https://github.com/unionlabs/union/edit/main/docs",
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
      plugins: [],
      customCss: [
        "./src/styles/index.css",
        "./src/styles/tailwind.css",
        "@fontsource/ibm-plex-mono/400.css",
        "@fontsource/ibm-plex-mono/600.css",
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
