import starlight from "@astrojs/starlight"
import starlightHeadingBadges from "starlight-heading-badges"
import starlightLinksValidator from "starlight-links-validator"

export const starlightConfig = starlight({
  title: "Union",
  tagline: "Connecting blockchains trustlessly",
  description:
    "Union is a hyper-efficient, zero-knowledge interoperability layer that connects Appchains, Layer 1, and Layer 2 networks.",
  favicon: "/favicon.svg",
  lastUpdated: true,
  editLink: {
    baseUrl: "https://github.com/unionlabs/union/edit/main/docs/"
  },
  social: {
    github: "https://github.com/unionlabs",
    discord: "https://discord.union.build",
    "x.com": "https://x.com/union_build"
  },
  defaultLocale: "root",
  locales: { root: { label: "English", lang: "en" } },
  logo: {
    alt: "Union Logo",
    dark: "./src/assets/union-logo/union-logo-transparent.svg",
    light: "./src/assets/union-logo/union-logo-white-transparent.svg"
  },
  expressiveCode: false,
  head: [
    {
      tag: "meta",
      attrs: {
        name: "description",
        content: "The Modular ZK Interoperability Layer"
      }
    },
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
    }
  ],
  sidebar: [
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
          badge: { variant: "note", text: "new" }
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
  ],
  plugins: [starlightLinksValidator(), starlightHeadingBadges()],
  customCss: [
    "./src/styles/index.css",
    "./src/styles/fonts.css",
    "./src/styles/tailwind.css",
    "./src/styles/twoslash.css",
    "./src/styles/starlight.css",
    "./node_modules/katex/dist/katex.min.css",
    "./node_modules/@shikijs/twoslash/style-rich.css"
  ]
})
