import { loadEnv } from "vite"
import svelte from "@astrojs/svelte"
import yaml from "@rollup/plugin-yaml"
import sitemap from "@astrojs/sitemap"
import tailwind from "@astrojs/tailwind"
import starlight from "@astrojs/starlight"
import { defineConfig } from "astro/config"
import { markdownConfiguration } from "./markdown.config.ts"
import starlightLinksValidator from "starlight-links-validator"

const SITE_URL = "https://union.build"

const {
  PORT = 4321,
  COMETBLS_STORE_PATH,
  ENABLE_DEV_TOOLBAR = "true"
} = loadEnv(process.env.NODE_ENV, process.cwd(), "")

export default defineConfig({
  site: SITE_URL,
  build: {},
  output: "static",
  trailingSlash: "ignore",
  server: ({ command }) => ({ port: Number(PORT) }),
  redirects: {
    "/feed": "/rss.xml",
    "/logo": "/union-logo.zip"
  },
  markdown: markdownConfiguration,
  devToolbar: { enabled: ENABLE_DEV_TOOLBAR === "true" },
  integrations: [
    starlight({
      title: "Union",
      tagline: "Connecting blockchains trustlessly",
      description:
        "Union is a hyper-efficient, zero-knowledge interoperability layer that connects Appchains, Layer 1, and Layer 2 networks.",
      favicon: "/favicon.svg",
      lastUpdated: true,
      social: {
        github: "https://github.com/unionlabs",
        discord: "https://discord.union.build",
        "x.com": "https://x.com/union_build"
      },
      components: {
        EditLink: "./src/components/EditLink.astro"
      },
      head: [
        {
          tag: "meta",
          attrs: {
            name: "description",
            content: "The Sovereign Interoperability Layer"
          }
        },
        {
          tag: "meta",
          attrs: {
            name: "og:image",
            content: "/og.png"
          }
        },
        {
          tag: "meta",
          attrs: {
            name: "twitter:image",
            content: "/og.png"
          }
        },
        {
          tag: "script",
          attrs: { src: "/scripts/anchor-targets.js" }
        },
        {
          // math rendering breaks without this
          tag: "link",
          attrs: {
            rel: "stylesheet",
            href: "https://www.unpkg.com/katex@0.16.9/dist/katex.min.css"
          }
        }
      ],
      locales: {
        root: { label: "English", lang: "en" }
      },
      defaultLocale: "root",
      logo: {
        alt: "Union Logo",
        dark: "./src/assets/union-logo/union-logo-transparent.svg",
        light: "./src/assets/union-logo/union-logo-white-transparent.svg"
      },
      sidebar: [
        {
          label: "Introduction",
          link: "/docs"
        },
        {
          label: "Architecture",
          autogenerate: {
            directory: "/docs/architecture"
          }
        },
        {
          label: "Concepts",
          autogenerate: {
            directory: "/docs/concepts"
          }
        },
        {
          label: "Infrastructure",
          items: [
            {
              label: "Node Operators",
              collapsed: true,
              autogenerate: {
                directory: "/docs/infrastructure/node-operators"
              }
            }
          ]
        },
        {
          label: "Integration",
          autogenerate: {
            directory: "/docs/integration"
          }
        },
        {
          label: "Demos",
          autogenerate: {
            directory: "/docs/demos"
          }
        },
        {
          label: "Joining the Testnet",
          autogenerate: {
            directory: "/docs/joining-testnet"
          }
        },
        {
          label: "Style Guide",
          autogenerate: {
            directory: "/docs/style-guide"
          }
        }
      ],
      plugins: [starlightLinksValidator()],
      customCss: [
        "./src/styles/fonts.css",
        "./src/styles/tailwind.css",
        "./src/styles/starlight.css"
      ]
    }),
    tailwind({
      applyBaseStyles: false,
      configFile: "tailwind.config.ts"
    }),
    svelte(),
    sitemap()
  ],
  vite: {
    // define: {
    //   COMETBLS_STORE_PATH: JSON.stringify(COMETBLS_STORE_PATH)
    // },
    plugins: [yaml()],
    optimizeDeps: {
      exclude: ["@urql/svelte", "echarts"]
    }
  }
})
