import { sveltekit } from "@sveltejs/kit/vite"
import { defineConfig } from "vite"

export default defineConfig(config => {
  const isProduction = config.mode === "production"

  return {
    plugins: [sveltekit()],
    esbuild: {
      drop: isProduction ? ["console", "debugger"] : []
    }
  }
})
