import sitemap from "@astrojs/sitemap"
import svelte from "@astrojs/svelte"
import tailwind from "@astrojs/tailwind"
import vercel from "@astrojs/vercel/serverless"
import icon from "astro-icon"
import { defineConfig } from "astro/config"
import { loadEnv } from "vite"
import { markdownConfiguration } from "./markdown.config.ts"

const SITE_URL = "https://union.build"

const { PORT = 4321, ENABLE_DEV_TOOLBAR = "false" } = loadEnv(
  process.env.NODE_ENV,
  process.cwd(),
  "",
)

export default defineConfig({
  site: SITE_URL,
  output: "server",
  experimental: {
    clientPrerender: true,
    contentIntellisense: true,
  },
  trailingSlash: "ignore",
  adapter: vercel({
    imageService: true,
  }),
  image: {
    domains: [
      "cdn.contentful.com",
      "images.ctfassets.net",
      "raw.githubusercontent.com",
      "avatars.githubusercontent.com",
    ],
  },
  markdown: markdownConfiguration,
  server: _ => ({ port: Number(PORT) }),
  devToolbar: { enabled: ENABLE_DEV_TOOLBAR === "true" },
  prefetch: { prefetchAll: true, defaultStrategy: "viewport" },
  redirects: {
    "/feed": "/rss.xml",
    "/logo": "/union-logo.zip",
  },
  vite: {
    assetsInclude: ["**/*.splinecode"],
    optimizeDeps: {
      exclude: ["echarts"],
    },
    define: {
      global: {},
    },
  },
  integrations: [
    icon(),
    tailwind({
      applyBaseStyles: false,
      configFile: "tailwind.config.ts",
    }),
    svelte(),
    sitemap(),
  ],
})
