import Icons from "unplugin-icons/vite"
import Inspect from "vite-plugin-inspect"
import { sveltekit } from "@sveltejs/kit/vite"
import { visualizer } from "rollup-plugin-visualizer"
import { purgeCss } from "vite-plugin-tailwind-purgecss"
import { nodePolyfills } from "vite-plugin-node-polyfills"
import { partytownVite } from "@builder.io/partytown/utils"
import { defineConfig, loadEnv, type PluginOption } from "vite"

export default defineConfig(config => {
  const {
    INSPECT,
    NODE_ENV,
    VISUALIZE,
    PORT = process.env.PORT || 5173
  } = loadEnv(config.mode, process.cwd(), "") as unknown as EnvironmentVariables

  const plugins = [
    purgeCss(),
    nodePolyfills({
      include: ["stream"],
      globals: { process: true, Buffer: true, global: true }
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

  const dropLogStatements = config.mode === "build" || NODE_ENV === "production"
  return {
    plugins,
    build: { target: "es2020" },
    esbuild: {
      drop: dropLogStatements ? ["console", "debugger"] : []
    },
    optimizeDeps: {
      exclude: ["@tanstack/svelte-query-devtools"]
    },
    server: {
      port: Number(PORT)
    },
    test: { include: ["src/**/*.{test,spec}.{js,ts}"] }
  }
})
