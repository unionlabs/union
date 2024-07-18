import { loadEnv } from "vite"
import svelte from "@astrojs/svelte"
import sitemap from "@astrojs/sitemap"
import netlify from "@astrojs/netlify"
import tailwind from "@astrojs/tailwind"
import starlight from "@astrojs/starlight"
import basicSsl from "@vitejs/plugin-basic-ssl"
import { markdownConfiguration } from "./markdown.config.ts"
import starlightLinksValidator from "starlight-links-validator"
import { defineConfig, type ViteUserConfig } from "astro/config"

const SITE_URL = "https://union.build"

const {
  PORT = 4321,
  CONTENTFUL_SPACE_ID,
  CONTENTFUL_PREVIEW_TOKEN,
  CONTENTFUL_DELIVERY_TOKEN,
  PUBLIC_ENV = "production",
  ENABLE_DEV_TOOLBAR = "false"
} = loadEnv(process.env.NODE_ENV, process.cwd(), "")

if (!(CONTENTFUL_SPACE_ID || CONTENTFUL_PREVIEW_TOKEN || CONTENTFUL_DELIVERY_TOKEN)) {
  throw new Error("Missing Contentful environment variables")
}

export default defineConfig({
  site: SITE_URL,
  /**
   * hybrid:
   *   - pre-rendered to HTML by default
   *   - use this when most of your site should be static
   *   - any individual page or endpoint can opt-out of pre-rendering
   */
  output: "hybrid",
  experimental: {},
  trailingSlash: "ignore",
  adapter: netlify({
    imageCDN: true, // default: true
    edgeMiddleware: false // default: false
  }),
  vite: viteConfiguration(),
  markdown: markdownConfiguration,
  server: _ => ({ port: Number(PORT) }),
  devToolbar: { enabled: ENABLE_DEV_TOOLBAR === "true" },
  redirects: { "/feed": "/rss.xml", "/logo": "/union-logo.zip" },
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
            content: "The Modular ZK Interoperability Layer"
          }
        },
        {
          tag: "meta",
          attrs: { property: "og:image", content: `${SITE_URL}/og.png` }
        },
        {
          tag: "meta",
          attrs: { property: "twitter:image", content: `${SITE_URL}/og.png` }
        },
        {
          tag: "script",
          attrs: { src: "/scripts/anchor-targets.js" }
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
          label: "Integrations",
          items: [
            {
              label: "Getting Started",
              link: "/docs/integrations/getting-started"
            },
            {
              label: "CometBFT",
              collapsed: true,
              autogenerate: {
                directory: "/docs/integrations/cometbft"
              }
            }
          ]
        },
        {
          label: "Joining the Testnet",
          collapsed: true,
          autogenerate: {
            directory: "/docs/joining-testnet"
          }
        }
      ],
      plugins: [starlightLinksValidator()],
      customCss: [
        "./src/styles/fonts.css",
        "./src/styles/tailwind.css",
        "./src/styles/starlight.css",
        "./node_modules/katex/dist/katex.min.css"
      ]
    }),
    tailwind({
      applyBaseStyles: false,
      configFile: "tailwind.config.ts"
    }),
    svelte(),
    sitemap()
  ]
})

function viteConfiguration(): ViteUserConfig {
  const baseConfiguration = {
    optimizeDeps: {
      exclude: ["echarts"]
    },
    assetsInclude: ["**/*.splinecode"]
  } satisfies ViteUserConfig

  const previewConfiguration = {
    plugins: [basicSsl()],
    server: { https: {} }
  } satisfies ViteUserConfig
  return Object.assign(
    baseConfiguration,
    // don't include 'preview' configuration in development/production
    PUBLIC_ENV === "preview" ? previewConfiguration : {}
  )
}
