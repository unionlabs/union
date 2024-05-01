import Inspect from "vite-plugin-inspect"
import { sveltekit } from "@sveltejs/kit/vite"
import { visualizer } from "rollup-plugin-visualizer"
import { purgeCss } from "vite-plugin-tailwind-purgecss"
import { defineConfig, loadEnv, type PluginOption } from "vite"

export default defineConfig(config => {
  const {
    INSPECT,
    NODE_ENV,
    VISUALIZE,
    VITE_APP_VERSION,
    PORT = process.env.PORT || 5173
  } = loadEnv(config.mode, process.cwd(), "") as unknown as EnvironmentVariables

  const plugins = [purgeCss(), sveltekit()] satisfies Array<PluginOption>

  if (INSPECT === "true") plugins.push(Inspect())
  if (VISUALIZE === "true") plugins.push(visualizer())

  return {
    plugins,
    esbuild: {
      drop: ["console", "debugger"]
    },
    optimizeDeps: {
      include: [
        "clsx",
        "valibot",
        "@urql/svelte",
        "lucide-svelte",
        "@cosmjs/stargate",
        "@cosmjs/tendermint-rpc",
        "@tanstack/svelte-query",
        "@cosmjs/cosmwasm-stargate",
        "@tanstack/svelte-query-devtools"
      ]
    },
    server: {
      port: Number(PORT)
    },
    test: { include: ["src/**/*.{test,spec}.{js,ts}"] },
    define: {
      // Node polyfills
      "process.env": {},
      __APP_VERSION__: JSON.stringify(VITE_APP_VERSION)
    },
    // Node polyfills
    resolve: {
      alias: {
        "node:buffer": "buffer",
        stream: "rollup-plugin-node-polyfills/polyfills/stream"
      }
    }
  }
})
