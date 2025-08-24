import react from "@astrojs/react"
import sitemap from "@astrojs/sitemap"
import starlight from "@astrojs/starlight"
import svelte from "@astrojs/svelte"
import starlightUtils from "@lorenzo_lewis/starlight-utils"
import tailwindcss from "@tailwindcss/vite"
import { defineConfig } from "astro/config"
import ecTwoSlash from "expressive-code-twoslash"
import starlightHeadingBadges from "starlight-heading-badges"
import starlightLinksValidator from "starlight-links-validator"
import starlightThemeRapide from "starlight-theme-rapide"
import Icons from "unplugin-icons/vite"
import { loadEnv } from "vite"
import examplesToPages from "./integrations/examples-to-pages.js"
import { markdownConfiguration } from "./markdown.config.ts"

const SITE_URL = "https://docs.union.build"
const SITE_DESCRIPTION =
  "Union is a hyper-efficient, zero-knowledge interoperability layer that connects Appchains, Layer 1, and Layer 2 networks."

const { PORT = 4321, ENABLE_DEV_TOOLBAR = "false" } = loadEnv(
  process.env.NODE_ENV,
  process.cwd(),
  "",
)

export default defineConfig({
  site: SITE_URL,
  output: "static",
  experimental: {
    clientPrerender: true,
    contentIntellisense: true,
  },
  trailingSlash: "ignore",
  markdown: markdownConfiguration,
  vite: {
    resolve: {
      alias: [
        { find: "icons:svelte", replacement: "~icons" },
        { find: "icons:astro", replacement: "~icons" },
        { find: "path", replacement: "rollup-plugin-node-polyfills/polyfills/path" },
      ],
    },
    plugins: [
      Icons({
        compiler: "svelte",
        autoInstall: true,
      }),
      Icons({
        compiler: "astro",
        autoInstall: true,
      }),
      tailwindcss(),
    ],
    ssr: {
      noExternal: ["monaco-editor"],
    },
    optimizeDeps: {
      include: ["@xterm/xterm"],
      esbuildOptions: { target: "es2020" },
    },
  },
  server: _ => ({
    port: Number(PORT),
    /**
     * required for webcontainer
     * @see https://webcontainers.io/guides/quickstart
     */
    headers: {
      "Cross-Origin-Embedder-Policy": "require-corp",
      "Cross-Origin-Opener-Policy": "same-origin",
    },
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
        baseUrl: "https://github.com/unionlabs/union/edit/main/docs/",
      },
      expressiveCode: true,
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/unionlabs" },
        { icon: "discord", label: "Discord", href: "https://discord.union.build" },
        { icon: "x.com", label: "X", href: "https://x.com/union_build" },
      ],
      logo: {
        alt: "Union Logo",
        replacesTitle: true,
        dark: "./src/assets/union-logo/union-logo-white.svg",
        light: "./src/assets/union-logo/union-logo-black.svg",
      },
      head: [
        {
          tag: "meta",
          attrs: { property: "og:image", content: "/og.png" },
        },
        {
          tag: "meta",
          attrs: { property: "twitter:image", content: "/og.png" },
        },
        {
          tag: "script",
          attrs: { src: "/scripts/anchor-targets.js" },
        },
        {
          tag: "link",
          attrs: {
            rel: "apple-touch-icon",
            href: "/pwa-192x192.png",
          },
        },
        {
          tag: "link",
          attrs: {
            rel: "mask-icon",
            href: "/favicon.svg",
            color: "#FFFFFF",
          },
        },
        {
          tag: "meta",
          attrs: {
            name: "msapplication-TileColor",
            content: "#131313",
          },
        },
        {
          tag: "meta",
          attrs: {
            name: "theme-color",
            content: "#131313",
          },
        },
      ],
      sidebar: [
        {
          label: "DOCS",
          items: [
            {
              label: "Introduction",
              link: "/",
            },
            {
              label: "Protocol",
              items: [
                {
                  label: "Overview",
                  link: "/protocol/overview",
                },
                {
                  label: "Deployments",
                  link: "/protocol/deployments",
                },
                {
                  label: "Chains",
                  autogenerate: {
                    directory: "/protocol/chains",
                  },
                },
                {
                  label: "Channels",
                  autogenerate: {
                    directory: "/protocol/channels",
                  },
                },
                {
                  label: "Connections",
                  autogenerate: {
                    directory: "/protocol/connections",
                  },
                },
              ],
            },
            {
              label: "Architecture",
              items: [
                {
                  label: "CometBLS",
                  link: "/architecture/cometbls",
                },
                {
                  label: "Galois",
                  link: "/architecture/galois",
                },
                {
                  label: "Voyager",
                  items: [
                    { label: "Overview", link: "/architecture/voyager/overview" },
                    { label: "Concepts", link: "/architecture/voyager/concepts" },
                  ],
                },
              ],
            },
            {
              label: "Connect",
              items: [
                {
                  label: "New Chain",
                  items: [
                    {
                      label: "Overview",
                      link: "/connect/new-chain/overview",
                    },
                    // {
                    //   label: "EVM",
                    //   link: "/connect/new-chain/evm"
                    // },
                    // {
                    //   label: "CosmWasm",
                    //   link: "/connect/new-chain/cosmwasm"
                    // },
                    // {
                    //   label: "Move",
                    //   link: "/connect/new-chain/move"
                    // }
                  ],
                },
                {
                  label: "Apps",
                  items: [
                    {
                      label: "Asset Transfer",
                      items: [
                        {
                          label: "Introduction",
                          link: "/connect/app/asset-transfer",
                        },
                        {
                          label: "Solidity",
                          link: "/connect/app/asset-transfer/solidity",
                        },
                        {
                          label: "CosmWasm",
                          link: "/connect/app/asset-transfer/cosmwasm",
                        },
                      ],
                    },
                    // {
                    //   label: "Custom Data",
                    //   items: [
                    //     {
                    //       label: "Introduction",
                    //       link: "/connect/app/custom-data"
                    //     }
                    //   ]
                    // }
                  ],
                },
              ],
            },
            {
              label: "Concepts",
              autogenerate: {
                directory: "/concepts",
              },
            },
            {
              label: "Standards",
              autogenerate: {
                directory: "/ucs",
              },
            },
            {
              label: "Infrastructure",
              items: [
                {
                  label: "Testnet 9",
                  link: "/infrastructure/testnet-9",
                },
                {
                  label: "Node Operators",
                  collapsed: true,
                  autogenerate: {
                    directory: "/infrastructure/node-operators",
                  },
                },
              ],
            },
            {
              label: "Typescript SDK",
              items: [
                {
                  label: "Getting Started",
                  link: "/integrations/typescript",
                },
                // {
                //   label: "Examples (EVM)",
                //   autogenerate: {
                //     directory: "/integrations/typescript/examples/evm",
                //   },
                // },
              ],
            },
            {
              label: "Integrations",
              items: [
                {
                  label: "Getting Started",
                  link: "/integrations/getting-started",
                },
                {
                  label: "GraphQL",
                  link: "/integrations/api/graphql",
                },
              ],
            },

            {
              label: "Joining the Testnet",
              collapsed: true,
              autogenerate: {
                directory: "/joining-testnet",
              },
            },
            {
              label: "Ceremony",
              link: "/ceremony",
            },
            {
              label: "FAQ",
              collapsed: true,
              items: [
                {
                  label: "How to conduct multisig transfers?",
                  link: "/faq/multisig-transfers-union-btc",
                },
                {
                  label: "How to add new tokens from Union app?",
                  link: "/faq/add-tokens-to-wallet",
                },
                {
                  label: "GraphQL",
                  link: "/integrations/api/graphql",
                },
              ],
            },
          ],
        },
        {
          label: "API",
          items: [
            {
              label: "GraphQL",
              link: "/reference/graphql",
              badge: { text: "new", variant: "success" },
            },
            {
              label: "Typescript SDK",
              autogenerate: { directory: "/reference/@unionlabs/sdk" },
              badge: { text: "evolving", variant: "caution" },
            },
            {
              label: "EVM SDK",
              autogenerate: { directory: "/reference/@unionlabs/sdk-evm" },
              badge: { text: "evolving", variant: "caution" },
            },
            {
              label: "Cosmos SDK",
              autogenerate: { directory: "/reference/@unionlabs/sdk-cosmos" },
              badge: { text: "evolving", variant: "caution" },
            },
          ],
        },
      ],
      plugins: [
        // examplesToPages({
        //   baseDir: "../ts-sdk-evm/examples",
        //   entryPoints: [
        //     // "../ts-sdk/examples/UCS03/*.ts",
        //     // "../ts-sdk/examples/Sui/*.ts",
        //     // "../ts-sdk/examples/EVM/*.ts",
        //     // "../ts-sdk/examples/Cosmos/*.ts",
        //     "../ts-sdk-evm/examples/*.ts",
        //   ],
        //   outDir: "./src/content/docs/integrations/typescript/examples/evm",
        //   clean: true,
        // }),
        starlightThemeRapide(),
        starlightUtils({
          multiSidebar: {
            switcherStyle: "horizontalList",
          },
        }),
        starlightHeadingBadges(),
        starlightLinksValidator(),
      ],
      customCss: [
        "./src/styles/index.css",
        "./src/styles/fonts.css",
        "./src/styles/tailwind.css",
        "./node_modules/katex/dist/katex.min.css",
      ],
    }),
    sitemap(),
    svelte(),
    react({
      include: ["**/react/**"],
      experimentalReactChildren: true,
    }),
  ],
})
