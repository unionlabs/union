import Icons from "unplugin-icons/vite"
import Inspect from "vite-plugin-inspect"
import { sveltekit } from "@sveltejs/kit/vite"
import { purgeCss } from "vite-plugin-tailwind-purgecss"
import { nodePolyfills } from "vite-plugin-node-polyfills"
import { defineConfig, loadEnv, type PluginOption } from "vite"
import { execSync } from "node:child_process"
import pkg from "./package.json"

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
    esbuild: {
      drop: dropLogStatements ? ["console", "debugger"] : []
    },
    optimizeDeps: {
      exclude: ["@tanstack/svelte-query-devtools"]
    },
    ssr: {
      external: [
        //
        "@telegram-apps/bridge"
      ]
    },
    server: {
      port: Number(PORT)
    },
    experimental: {},
    test: { include: ["src/**/*.{test,spec}.{js,ts}"] },
    define: {
      "import.meta.env.VERSION": JSON.stringify(pkg.version),
      "import.meta.env.GIT_HASH": JSON.stringify(getGitHash())
    }
  }
})

const getGitHash = () => {
  if (process.env.CF_PAGES_COMMIT_SHA) {
    return process.env.CF_PAGES_COMMIT_SHA.slice(0, 7);
  }
  try {
    const { execSync } = require('child_process');
    return execSync('git rev-parse --short HEAD').toString().trim();
  } catch {
    return 'unknown';
  }
};
