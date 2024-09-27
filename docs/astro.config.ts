import { loadEnv } from "vite"
import react from "@astrojs/react"
import svelte from "@astrojs/svelte"
import sitemap from "@astrojs/sitemap"
import tailwind from "@astrojs/tailwind"
import starlight from "@astrojs/starlight"
import { defineConfig } from "astro/config"
import starlightThemeRapide from "starlight-theme-rapide"
import starlightUtils from "@lorenzo_lewis/starlight-utils"
// import { markdownConfiguration } from "./markdown.config.ts"
import starlightHeadingBadges from "starlight-heading-badges"
import starlightLinksValidator from "starlight-links-validator"

const SITE_URL = "https://docs.union.build"
const SITE_DESCRIPTION =
  "Union is a hyper-efficient, zero-knowledge interoperability layer that connects Appchains, Layer 1, and Layer 2 networks."

const { PORT = 4321, ENABLE_DEV_TOOLBAR = "false" } = loadEnv(
  process.env.NODE_ENV,
  process.cwd(),
  ""
)

export default defineConfig({
  site: SITE_URL,
  output: "static",
  experimental: {
    serverIslands: true,
    clientPrerender: true,
    directRenderScript: true,
    contentIntellisense: true
  },
\ma
  trailingSlash: "ignore",
  markdown: {},
  // markdown: markdownConfiguration,
  vite: {
    ssr: {
      noExternal: ["monaco-editor"]
    },
    optimizeDeps: {
      include: ["@xterm/xterm"],
      esbuildOptions: { target: "es2022" }
    },
    resolve: {
      alias: {
        path: "rollup-plugin-node-polyfills/polyfills/path"
      }
    }
  },
  server: _ => ({
    port: Number(PORT),
    /**
     * required for webcontainer
     * @see https://webcontainers.io/guides/quickstart
     */
    headers: {
      "Cross-Origin-Embedder-Policy": "require-corp",
      "Cross-Origin-Opener-Policy": "same-origin"
    }
  }),
  devToolbar: { enabled: ENABLE_DEV_TOOLBAR === "true" },
  prefetch: { prefetchAll: true, defaultStrategy: "viewport" },
  redirects: { "/logo": "/union-logo.zip" },
  integrations: [
    starlight({
      title: "Union",
      lastUpdated: true,
      favicon: "/favicon.svg",
      description: SITE_DESCRIPTION,
      tagline: "Connecting blockchains trustlessly",
      defaultLocale: "root",
      locales: { root: { label: "English", lang: "en" } },
      editLink: {
        baseUrl: "https://github.com/unionlabs/union/edit/main/docs/"
      },
      social: {
        github: "https://github.com/unionlabs",
        discord: "https://discord.union.build",
        "x.com": "https://x.com/union_build"
      },
      logo: {
        alt: "Union Logo",
        replacesTitle: true,
        dark: "./src/assets/union-logo/union-logo-white.svg",
        light: "./src/assets/union-logo/union-logo-black.svg"
      },
      head: [
        {
          tag: "meta",
          attrs: { property: "og:image", content: "/og.png" }
        },
        {
          tag: "meta",
          attrs: { property: "twitter:image", content: "/og.png" }
        },
        {
          tag: "script",
          attrs: { src: "/scripts/anchor-targets.js" }
        },
        {
          tag: "link",
          attrs: {
            rel: "apple-touch-icon",
            href: "/pwa-192x192.png"
          }
        },
        {
          tag: "link",
          attrs: {
            rel: "mask-icon",
            href: "/favicon.svg",
            color: "#FFFFFF"
          }
        },
        {
          tag: "meta",
          attrs: {
            name: "msapplication-TileColor",
            content: "#131313"
          }
        },
        {
          tag: "meta",
          attrs: {
            name: "theme-color",
            content: "#131313"
          }
        }
      ],
      sidebar: [
        {
          label: "DOCS",
          items: [
            {
              label: "Introduction",
              link: "/"
            },
            {
              label: "Architecture",
              autogenerate: {
                directory: "/architecture"
              }
            },
            {
              label: "Concepts",
              autogenerate: {
                directory: "/concepts"
              }
            },
            {
              label: "Infrastructure",
              items: [
                {
                  label: "Node Operators",
                  collapsed: true,
                  autogenerate: {
                    directory: "/infrastructure/node-operators"
                  }
                }
              ]
            },
            {
              label: "Integrations",
              items: [
                {
                  label: "Getting Started",
                  link: "/integrations/getting-started"
                },
                {
                  label: "TypeScript SDK",
                  link: "/integrations/typescript",
                  badge: { variant: "success", text: "new" }
                },
                {
                  label: "API",
                  collapsed: true,
                  autogenerate: {
                    directory: "/integrations/api"
                  }
                },
                {
                  label: "CometBFT",
                  collapsed: true,
                  autogenerate: {
                    directory: "/integrations/cometbft"
                  }
                }
              ]
            },
            {
              label: "Joining the Testnet",
              collapsed: true,
              autogenerate: {
                directory: "/joining-testnet"
              }
            }
          ]
        },
        {
          label: "API",
          items: [
            {
              label: "GraphQL",
              link: "/reference/graphql",
              badge: { text: "new", variant: "success" }
            }
          ]
        }
      ],
      plugins: [
        starlightThemeRapide(),
        starlightUtils({
          multiSidebar: {
            switcherStyle: "horizontalList"
          }
        }),
        starlightHeadingBadges(),
        starlightLinksValidator()
      ],
      customCss: [
        "./src/styles/index.css",
        "./src/styles/fonts.css",
        "./src/styles/tailwind.css",
        "./src/styles/starlight.css",
        "./node_modules/katex/dist/katex.min.css"
      ]
    }),
    sitemap(),
    tailwind({
      applyBaseStyles: false,
      configFile: "tailwind.config.ts"
    }),
    svelte(),
    react({
      include: ["**/react/**"],
      experimentalReactChildren: true
    })
  ]
})
