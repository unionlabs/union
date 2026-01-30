import { sveltekit } from "@sveltejs/kit/vite"
import tailwindcss from "@tailwindcss/vite"
import { svelteTesting } from "@testing-library/svelte/vite"
import { defineConfig } from "vite"

export default defineConfig({
  define: {
    "import.meta.vitest": "undefined",
  },
  plugins: [sveltekit(), tailwindcss()],
  build: { sourcemap: true },
  server: {
    allowedHosts: true,
    watch: {
      ignored: ["**/indexer-v2/**", "**/*.sqlite", "**/*.sqlite-wal", "**/*.sqlite-shm"],
    },
  },
  test: {
    workspace: [
      {
        extends: "./vite.config.ts",
        plugins: [svelteTesting()],
        test: {
          name: "client",
          environment: "happy-dom",
          clearMocks: true,
          include: ["src/**/*.svelte.{test,spec}.{js,ts}"],
          exclude: ["src/lib/server/**"],
          setupFiles: ["./vitest-setup-client.ts"],
        },
      },
      {
        extends: "./vite.config.ts",
        test: {
          name: "server",
          environment: "node",
          include: ["src/**/*.{test,spec}.{js,ts}"],
          exclude: ["src/**/*.svelte.{test,spec}.{js,ts}"],
        },
      },
    ],
  },
})
