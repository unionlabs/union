import { loadEnv } from "vite"
import react from "@astrojs/react"
import svelte from "@astrojs/svelte"
// @ts-expect-error
import liveCode from "astro-live-code"
import sitemap from "@astrojs/sitemap"
import netlify from "@astrojs/netlify"
import tailwind from "@astrojs/tailwind"
import { defineConfig } from "astro/config"
import starlight from "./starlight.config.ts"
import { markdownConfiguration } from "./markdown.config.ts"

const SITE_URL = "https://union.build"

const { PORT = 4321, ENABLE_DEV_TOOLBAR = "false" } = loadEnv(
  process.env.NODE_ENV,
  process.cwd(),
  ""
)

export default defineConfig({
  site: SITE_URL,
  output: "hybrid",
  experimental: {
    serverIslands: true,
    clientPrerender: true,
    directRenderScript: true
  },
  trailingSlash: "ignore",
  adapter: netlify({
    imageCDN: false,
    edgeMiddleware: false
  }),
  image: {
    domains: [
      "cdn.contentful.com",
      "images.ctfassets.net",
      "raw.githubusercontent.com",
      "avatars.githubusercontent.com"
    ]
  },
  markdown: markdownConfiguration,
  server: _ => ({
    port: Number(PORT),
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp"
    }
  }),
  devToolbar: { enabled: ENABLE_DEV_TOOLBAR === "true" },
  prefetch: { prefetchAll: true, defaultStrategy: "viewport" },
  redirects: { "/feed": "/rss.xml", "/logo": "/union-logo.zip" },
  vite: {
    assetsInclude: ["**/*.splinecode"],
    optimizeDeps: {
      exclude: ["echarts"]
    },
    define: {
      // Node.js polyfills
      global: {
        "process.env": {}
      }
    },
    // Node.js polyfills
    resolve: {
      alias: {
        "node:events": "events",
        "node:process": "process",
        stream: "rollup-plugin-node-polyfills/polyfills/stream"
      }
    }
  },
  integrations: [
    liveCode(),
    react({ experimentalReactChildren: true }),
    starlight,
    svelte(),
    sitemap(),
    tailwind({ applyBaseStyles: false, configFile: "tailwind.config.ts" })
  ]
})
