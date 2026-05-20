import { sveltekit } from "@sveltejs/kit/vite"
import tailwindcss from "@tailwindcss/vite"
import { svelteTesting } from "@testing-library/svelte/vite"
import * as Path from "node:path"
import { defineConfig } from "vite"

export default defineConfig({
  define: {
    // TODO: complete me <3
    "import.meta.vitest": "undefined",
  },
  plugins: [sveltekit(), tailwindcss()],
  build: { sourcemap: true },
  assetsInclude: ["**/*.wasm"],
  resolve: {
    alias: {
      /**
       * XXX: Needed probably (hopefully?) only for us given monorepo context
       * with source-based project references and Vite + SvelteKit + tsc
       * shadowing issues.
       */
      "$unionlabs/sdk/internal/wasm": Path.resolve(
        __dirname,
        "../ts-sdk/src/internal/wasm",
      ),
    },
  },
  server: {
    allowedHosts: true,
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
