import { loadEnv } from "vite"
import react from "@astrojs/react"
import svelte from "@astrojs/svelte"
import sitemap from "@astrojs/sitemap"
import Icons from "unplugin-icons/vite"
import tailwind from "@astrojs/tailwind"
import starlight from "@astrojs/starlight"
import { defineConfig } from "astro/config"
import starlightThemeRapide from "starlight-theme-rapide"
import starlightUtils from "@lorenzo_lewis/starlight-utils"
import { markdownConfiguration } from "./markdown.config.ts"
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
  trailingSlash: "ignore",
  markdown: markdownConfiguration,
  vite: {
    resolve: {
      alias: [
        { find: "icons:svelte", replacement: "~icons" },
        { find: "icons:astro", replacement: "~icons" },
        { find: "path", replacement: "rollup-plugin-node-polyfills/polyfills/path" }
      ]
    },
    plugins: [
      Icons({
        compiler: "svelte",
        autoInstall: true
      }),
      Icons({
        compiler: "astro",
        autoInstall: true
      })
    ],
    ssr: {
      noExternal: ["monaco-editor"]
    },
    optimizeDeps: {
      include: ["@xterm/xterm"],
      esbuildOptions: { target: "es2020" }
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
  redirects: { "/logo": "/union-logo.zip" },
  devToolbar: { enabled: ENABLE_DEV_TOOLBAR === "true" },
  prefetch: { prefetchAll: true, defaultStrategy: "viewport" },
  integrations: [
    starlight({
      title: "Union",
      lastUpdated: true,
      defaultLocale: "root",
      favicon: "/favicon.svg",
      description: SITE_DESCRIPTION,
      tagline: "Connecting blockchains trustlessly",
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
              label: "Protocol",
              items: [
                {
                  label: "Overview",
                  link: "/protocol/overview"
                },
                {
                  label: "Chains",
                  autogenerate: {
                    directory: "/protocol/chains"
                  }
                },
                {
                  label: "Channels",
                  autogenerate: {
                    directory: "/protocol/channels"
                  }
                },
                {
                  label: "Connections",
                  autogenerate: {
                    directory: "/protocol/connections"
                  }
                }
              ]
            },
            {
              label: "Architecture",
              autogenerate: {
                directory: "/architecture"
              }
            },
            {
              label: "Connect",
              items: [
                {
                  label: "Apps",
                  items: [
                    {
                      label: "Asset Transfer",
                      items: [
                        {
                          label: "Introduction",
                          link: "/connect/app/asset-transfer"
                        },
                        {
                          label: "Solidity",
                          link: "/connect/app/asset-transfer/solidity"
                        },
                        {
                          label: "CosmWasm",
                          link: "/connect/app/asset-transfer/cosmwasm"
                        }
                      ]
                    }
                    // {
                    //   label: "Custom Data",
                    //   items: [
                    //     {
                    //       label: "Introduction",
                    //       link: "/connect/app/custom-data"
                    //     }
                    //   ]
                    // }
                  ]
                }
                // {
                //   label: "New Chain",
                //   link: "/connect/new-chain"
                // }
              ]
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
                  label: "Testnet 9",
                  link: "/infrastructure/testnet-9"
                },
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
            },
            {
              label: "Ceremony",
              link: "/ceremony"
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
