import Icons from "unplugin-icons/vite"
import Inspect from "vite-plugin-inspect"
import { sveltekit } from "@sveltejs/kit/vite"
import { purgeCss } from "vite-plugin-tailwind-purgecss"
import { nodePolyfills } from "vite-plugin-node-polyfills"
import { defineConfig, loadEnv, type PluginOption } from "vite"

export default defineConfig(config => {
  const {
    INSPECT,
    NODE_ENV,
    ENVIRONMENT,
    PORT = process.env.PORT || 5173
  } = loadEnv(config.mode, process.cwd(), "") as unknown as EnvironmentVariables

  const plugins = [
    purgeCss(),
    nodePolyfills({
      include: ["stream"],
      globals: { process: true, Buffer: true, global: true }
    }),
    sveltekit(),
    Icons({ compiler: "svelte", autoInstall: true })
  ] satisfies Array<PluginOption>

  if (INSPECT === "true") plugins.push(Inspect())

  // we want logs to show up in preview deployments for debugging
  const dropLogStatements = config.mode === "build" && ENVIRONMENT === "production"
  return {
    plugins,
    build: { target: "es2020" },
    define: {
      "import.meta.env.ENVIRONMENT": JSON.stringify(ENVIRONMENT),
      "import.meta.env.NODE_ENV": JSON.stringify(NODE_ENV)
    },
    esbuild: {
      drop: dropLogStatements ? ["console", "debugger"] : []
    },
    optimizeDeps: {
      exclude: ["@tanstack/svelte-query-devtools"]
    },
    ssr: {
      external: []
    },
    server: {
      port: Number(PORT)
    },
    experimental: {},
    test: { include: ["src/**/*.{test,spec}.{js,ts}"] }
  }
})
