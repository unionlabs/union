import Icons from "unplugin-icons/vite"
import Inspect from "vite-plugin-inspect"
import { sveltekit } from "@sveltejs/kit/vite"
import { sentrySvelteKit } from "@sentry/sveltekit"
import { visualizer } from "rollup-plugin-visualizer"
import { purgeCss } from "vite-plugin-tailwind-purgecss"
import { partytownVite } from "@builder.io/partytown/utils"
import { defineConfig, loadEnv, type PluginOption } from "vite"

export default defineConfig(config => {
  const {
    INSPECT,
    NODE_ENV,
    VISUALIZE,
    SENTRY_AUTH_TOKEN,
    PORT = process.env.PORT || 5173
  } = loadEnv(config.mode, process.cwd(), "") as unknown as EnvironmentVariables

  const plugins = [
    purgeCss(),
    sentrySvelteKit({
      sourceMapsUploadOptions: {
        project: "app",
        telemetry: true,
        org: "unionlabs",
        authToken: SENTRY_AUTH_TOKEN
      }
    }),
    sveltekit(),
    partytownVite({
      debug: NODE_ENV === "development",
      dest: `${import.meta.dirname}/static/~partytown`
    }),
    Icons({ compiler: "svelte", autoInstall: true })
  ] satisfies Array<PluginOption>

  if (INSPECT === "true") plugins.push(Inspect())
  if (VISUALIZE === "true") plugins.push(visualizer({ filename: `stats/${Date.now()}_stats.html` }))

  return {
    plugins,
    esbuild: {
      drop: ["console", "debugger"]
    },
    optimizeDeps: {
      exclude: ["@tanstack/svelte-query-devtools"]
    },
    build: {
      sourcemap: true
    },
    server: {
      port: Number(PORT)
    },
    define: {
      // Node polyfills
      "process.env": {}
    },
    // Node polyfills
    resolve: {
      alias: {
        "node:buffer": "buffer",
        "node:events": "events",
        "node:process": "process",
        stream: "rollup-plugin-node-polyfills/polyfills/stream"
      }
    },
    test: { include: ["src/**/*.{test,spec}.{js,ts}"] }
  }
})
